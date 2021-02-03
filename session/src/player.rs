/*
 * Copyright (C) 2015 Gaetan Dubreil
 *
 *  All rights reserved.This file is distributed under the terms of the
 *  GNU General Public License version 3.0.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Lesser General Public License for more details.
 *
 * You should have received a copy of the GNU Lesser General Public License
 * along with this program; if not, write to the Free Software
 * Foundation, Inc., 59 Temple Place - Suite 330, Boston, MA 02111-1307, USA.
 */

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::time::Duration;

use talker::audio_format::AudioFormat;

use crate::band::{Band, Operation};
use crate::feedback;
use crate::feedback::Feedback;
use crate::state::State;

#[derive(PartialEq, Debug, Clone)]
enum Order {
    Nil,
    Start,
    Play,
    Pause,
    Stop,
    SetTimeRange(i64, i64),
    LoadBand(String),
    ModifyBand(Operation),
    Exit,
}

impl Order {
    pub fn to_string(&self) -> String {
        (match self {
            Order::Nil => "Nil",
            Order::Start => "Start",
            Order::Play => "Play",
            Order::Pause => "Pause",
            Order::Stop => "Stop",
            Order::SetTimeRange(_, _) => "SetTimeRange",
            Order::LoadBand(_) => "LoadBand",
            Order::ModifyBand(_) => "ModifyBand",
            Order::Exit => "Exit",
        })
        .to_string()
    }
}

fn create_band(
    band_description: &String,
    channels: &mut Vec<Vec<f32>>,
    min_channels: usize,
    chunk_size: usize,
) -> Result<Band, failure::Error> {
    let band = Band::make(band_description)?;

    let nb_channels = usize::max(min_channels, band.nb_channels());

    if channels.len() < nb_channels {
        for _ in channels.len()..nb_channels {
            channels.push(vec![0.; chunk_size]);
        }
    } else if channels.len() > nb_channels {
        let nb_over = channels.len() - nb_channels;

        for _ in 0..nb_over {
            let _ = channels.pop();
        }
    }
    Ok(band)
}

pub struct Player {
    order_sender: Sender<Order>,
    state_receiver: Receiver<State>,
    state: State,
}

pub type RPlayer = Rc<RefCell<Player>>;

fn run(
    order_receiver: &Receiver<Order>,
    state_sender: &Sender<State>,
    band_description: String,
) -> Result<(), failure::Error> {
    let send_state = |order: &String, prev_state: State, state: State| {
        match state_sender.send(state) {
            Err(e) => eprintln!("Player state sender error : {}", e),
            Ok(()) => (),
        }

        println!(
            "Player received order {} : {} -> {}",
            order,
            prev_state.to_string(),
            state.to_string()
        );

        state
    };

    let mut res = Ok(());
    let mut tick: i64 = 0;
    let mut start_tick: i64 = 0;
    let mut end_tick: i64 = i64::max_value();
    let chunk_size = AudioFormat::chunk_size();

    let mut buf: Vec<f32> = vec![0.; chunk_size];
    let mut channels: Vec<Vec<f32>> = Vec::new();

    let feedback = Feedback::new_ref(chunk_size)?;
    let min_channels = feedback.borrow().nb_channels();
    let mut extra_outputs = Vec::new();
    extra_outputs.push(feedback.clone());

    let mut band = create_band(&band_description, &mut channels, min_channels, chunk_size)?;

    let mut state = State::Stopped;
    let mut oorder = order_receiver.recv();

    loop {
        match oorder {
            Ok(order) => match order {
                Order::Start => {
                    band.open()?;
                    feedback.borrow_mut().open()?;

                    tick = start_tick;
                    state = send_state(&order.to_string(), state, State::Playing);
                }
                Order::Pause => {
                    if state != State::Paused {
                        band.pause()?;
                        feedback.borrow_mut().pause()?;

                        state = send_state(&order.to_string(), state, State::Paused);
                    }
                    oorder = order_receiver.recv();
                    continue;
                }
                Order::Play => {
                    band.run()?;
                    feedback.borrow_mut().run()?;
                    state = send_state(&order.to_string(), state, State::Playing);
                }
                Order::Stop => {
                    if state != State::Stopped {
                        band.close()?;
                        feedback.borrow_mut().close()?;
                        tick = start_tick;
                        state = send_state(&order.to_string(), state, State::Stopped);
                    }
                    oorder = order_receiver.recv();
                    continue;
                }
                Order::SetTimeRange(start, end) => {
                    start_tick = start;
                    end_tick = end;

                    if state != State::Playing {
                        oorder = Ok(Order::Pause);
                        continue;
                    }
                }
                Order::LoadBand(band_desc) => {
                    //                                band.close()?;
                    band = create_band(&band_desc, &mut channels, min_channels, chunk_size)?;

                    match state {
                        State::Playing => band.open()?,
                        State::Paused => {
                            band.open()?;
                            oorder = Ok(Order::Pause);
                            continue;
                        }
                        State::Stopped => {
                            oorder = Ok(Order::Stop);
                            continue;
                        }
                        State::Exited => (),
                    }
                }
                Order::ModifyBand(operation) => {
                    band.modify(&operation)?;

                    match state {
                        State::Paused => {
                            oorder = Ok(Order::Pause);
                            continue;
                        }
                        State::Stopped => {
                            oorder = Ok(Order::Stop);
                            continue;
                        }
                        _ => (),
                    }
                }
                Order::Exit => {
                    send_state(&order.to_string(), state, State::Exited);
                    break;
                }
                Order::Nil => {}
            },
            Err(e) => {
                let msg = format!("Player::run error : {}", e);
                println!("{}", msg);
                res = Err(failure::err_msg(msg));
                break;
            }
        }

        if tick > end_tick {
            tick = start_tick;
        }

        let mut len = if tick + (chunk_size as i64) < end_tick {
            chunk_size
        } else {
            (end_tick - tick) as usize
        };

        for rmixer in band.mixers().values() {
            match rmixer
                .borrow_mut()
                .come_out(tick, &mut buf, &mut channels, len, &extra_outputs)
            {
                Ok(ln) => {
                    len = ln;

                    if ln == 0 {
                        break;
                    }
                }
                Err(e) => {
                    res = Err(failure::err_msg(format!("Player::run error : {}", e)));
                    break;
                }
            }
        }
        tick += len as i64;

        match order_receiver.try_recv() {
            Err(_) => {
                oorder = Ok(Order::Nil);
            }
            Ok(o) => {
                oorder = Ok(o);
            }
        }
    }

    band.close()?;
    feedback.borrow_mut().close()?;
    let _ = state_sender.send(State::Exited)?;
    res
}

impl Player {
    pub fn new(band_description: String) -> Result<Player, failure::Error> {
        let (order_sender, order_receiver): (Sender<Order>, Receiver<Order>) =
            std::sync::mpsc::channel();
        let (state_sender, state_receiver): (Sender<State>, Receiver<State>) =
            std::sync::mpsc::channel();

        let state = if band_description.is_empty() {
            State::Exited
        } else {
            let _join_handle = thread::spawn(move || {
                match run(&order_receiver, &state_sender, band_description) {
                    Ok(()) => Ok(()),
                    Err(e) => {
                        eprintln!("Player state sender error : {}", e);
                        let _ = state_sender.send(State::Exited);
                        Err(e)
                    }
                }
            });
            State::Stopped
        };
        Ok(Self {
            order_sender,
            state_receiver,
            state,
        })
    }

    pub fn state(&mut self) -> State {
        match self.state {
            State::Exited => {}
            _ => {
                thread::sleep(Duration::from_millis(60));
                match self.state_receiver.try_recv() {
                    Err(_) => {}
                    Ok(state) => {
                        self.state = state;
                    }
                }
            }
        }
        self.state
    }

    fn check_not_exited(&self) -> Result<(), failure::Error> {
        match self.state {
            State::Exited => Err(failure::err_msg("Player exited")),
            _ => Ok(()),
        }
    }

    pub fn start(&mut self) -> Result<State, failure::Error> {
        self.check_not_exited()?;
        self.order_sender
            .send(Order::Start)
            .map_err(|e| failure::err_msg(format!("Player::play error : {}", e)))?;

        Ok(self.state())
    }

    pub fn play(&mut self) -> Result<State, failure::Error> {
        self.check_not_exited()?;

        match self.state {
            State::Playing => self
                .order_sender
                .send(Order::Pause)
                .map_err(|e| failure::err_msg(format!("Player::play error : {}", e)))?,
            _ => self
                .order_sender
                .send(Order::Play)
                .map_err(|e| failure::err_msg(format!("Player::play error : {}", e)))?,
        }
        Ok(self.state())
    }

    pub fn pause(&mut self) -> Result<State, failure::Error> {
        self.check_not_exited()?;

        match self.state {
            State::Playing => self
                .order_sender
                .send(Order::Pause)
                .map_err(|e| failure::err_msg(format!("Player::pause error : {}", e)))?,
            State::Paused => self
                .order_sender
                .send(Order::Play)
                .map_err(|e| failure::err_msg(format!("Player::pause error : {}", e)))?,
            _ => (),
        };
        Ok(self.state())
    }

    pub fn stop(&mut self) -> Result<State, failure::Error> {
        self.check_not_exited()?;
        match self.state {
            State::Stopped => {}
            _ => self
                .order_sender
                .send(Order::Stop)
                .map_err(|e| failure::err_msg(format!("Player::stop error : {}", e)))?,
        }
        Ok(self.state())
    }

    pub fn set_time_range(&self, start_tick: i64, end_tick: i64) -> Result<State, failure::Error> {
        self.check_not_exited()?;
        self.order_sender
            .send(Order::SetTimeRange(start_tick, end_tick))
            .map_err(|e| failure::err_msg(format!("Player::set_time_range error : {}", e)))?;

        Ok(self.state)
    }

    pub fn load_band(&self, band_description: String) -> Result<State, failure::Error> {
        self.check_not_exited()?;
        self.order_sender
            .send(Order::LoadBand(band_description))
            .map_err(|e| failure::err_msg(format!("Player::load_band error : {}", e)))?;

        Ok(self.state)
    }

    pub fn modify_band(&self, operation: &Operation) -> Result<State, failure::Error> {
        self.check_not_exited()?;
        self.order_sender
            .send(Order::ModifyBand(operation.clone()))
            .map_err(|e| failure::err_msg(format!("Player::load_band error : {}", e)))?;

        Ok(self.state)
    }

    pub fn exit(&mut self) -> Result<State, failure::Error> {
        match self.state {
            State::Exited => {}
            _ => {
                self.order_sender
                    .send(Order::Exit)
                    .map_err(|e| failure::err_msg(format!("Player::exit error : {}", e)))?;
            }
        }
        Ok(self.state())
    }
}
