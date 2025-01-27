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
use talker::lv2_handler;

use crate::band::{Band, Operation};
use crate::feedback::Feedback;
use crate::output::Output;
use crate::state::State;

#[derive(PartialEq, Debug, Clone)]
enum Order {
    Nil,
    Play,
    Record,
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
            Order::Play => "Play",
            Order::Record => "Record",
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
    let mut feedback = Feedback::new(chunk_size)?;

    
    let mut band = Band::make(&band_description, true)?;
    
    let feedback_mixer_id = band.mixers().iter().next().map_or(0, |(k, _)| *k);
    
    let mut state = State::Stopped;
    let mut oorder = order_receiver.recv();

    loop {
        match oorder {
            Ok(order) => match order {
                Order::Pause => {
                    send_state(&order.to_string(), state, State::Paused);

                    if state == State::Playing || state == State::Recording {
                        let len = band.play(tick, feedback.fade_len())?;
 
                        if let Some(mxr) = band.find_mixer(feedback_mixer_id) {
                            feedback.write_fadeout(mxr.borrow().channels_buffers(), len)?;
                        }

                        tick += len as i64;
     
                        band.pause()?;
                        // feedback.pause()?;
                    }
                    state = State::Paused;
                    oorder = order_receiver.recv();
                    continue;
                 }
                Order::Play => {
                    send_state(&order.to_string(), state, State::Playing);

                    if state == State::Stopped {
                        band.open()?;
                        feedback.open()?;
                    }
                    else if state == State::Paused {
                        band.run()?;
                        // feedback.run()?;

                        let len = band.play(tick, feedback.fade_len())?;
 
                        if let Some(mxr) = band.find_mixer(feedback_mixer_id) {
                            feedback.write_fadein(mxr.borrow().channels_buffers(), len)?;
                        }

                        tick += len as i64;
                    }
                    
                    state = State::Playing;
                }
                Order::Record => {
                    send_state(&order.to_string(), state, State::Recording);

                    if state == State::Stopped {
                        band.set_record(true)?;
                        band.open()?;
                        feedback.open()?;
                    }
                    else if state == State::Paused {
                        band.run()?;
                        feedback.run()?;
                    }
                    
                    state = State::Recording;
                }
                Order::Stop => {
                    send_state(&order.to_string(), state, State::Stopped);

                    if state == State::Playing || state == State::Recording {
                        let len = band.play(tick, feedback.fade_len())?;
 
                        if let Some(mxr) = band.find_mixer(feedback_mixer_id) {
                            feedback.write_fadeout(mxr.borrow().channels_buffers(), len)?;
                        }
                    }
                    if state != State::Stopped {
                        band.fadeout(tick)?;
                        band.close()?;
                        feedback.close()?;
                        band.set_record(false)?;
                        tick = start_tick;
                        state = State::Stopped;
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
                   match state {
                       State::Playing | State::Recording => {
                            let len = band.play(tick, feedback.fade_len())?;
                            let _ = band.close();

                            let mut new_band = Band::make(&band_desc, true)?;
                            new_band.open()?;
                            let len = new_band.play(tick, len)?;

                            let mxr = band.find_mixer(feedback_mixer_id).unwrap_or_else(|| band.first_mixer());
                            let new_mxr = new_band.find_mixer(feedback_mixer_id).unwrap_or_else(|| new_band.first_mixer());

                            feedback.write_fade(mxr.borrow().channels_buffers(), new_mxr.borrow().channels_buffers(), len)?;

                            tick += len as i64;
                            band = new_band;
                        }
                        State::Paused => {
                            let _ = band.close();
                            band = Band::make(&band_desc, true)?;
                            band.open()?;
                            oorder = Ok(Order::Pause);
                            continue;
                        }
                        State::Stopped => {
                            band = Band::make(&band_desc, true)?;
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

        // Run LV2 workers
        let _ = match lv2_handler::run_workers() {
            Ok(()) => (),
            Err(e) => {
                let msg = format!("Player::run error : {}", e);
                println!("{}", msg);
                res = Err(failure::err_msg(msg));
                break;
            }
        };

        if tick > end_tick {
            tick = start_tick;
        }

        let mut len = if tick + (chunk_size as i64) < end_tick {
            chunk_size
        } else {
            (end_tick - tick) as usize
        };

        len = band.play(tick, len)?;

        if let Some(mxr) = band.find_mixer(feedback_mixer_id) {
            feedback.write(mxr.borrow().channels_buffers(), len)?;
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
    feedback.close()?;

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
                thread::sleep(Duration::from_millis(20));

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

    pub fn wait(&mut self) -> Result<State, failure::Error> {
        match self.state {
            State::Exited => (),
            _ => {
                thread::sleep(Duration::from_millis(20));

                match self.state_receiver.recv() {
                    Err(e) => {
                        return Err(failure::err_msg(format!("Player::play error : {}", e)));
                    }
                    Ok(state) => {
                        self.state = state;
                    }
                }
            }
        }
        Ok(self.state)
    }

    fn check_not_exited(&self) -> Result<(), failure::Error> {
        match self.state {
            State::Exited => Err(failure::err_msg("Player exited")),
            _ => Ok(()),
        }
    }

    pub fn play(&mut self) -> Result<State, failure::Error> {
        self.check_not_exited()?;

        if self.state != State::Playing {
            self
                .order_sender
                .send(Order::Play)
                .map_err(|e| failure::err_msg(format!("Player::play error : {}", e)))?;
        }
        Ok(self.state())
    }

    pub fn pause(&mut self) -> Result<State, failure::Error> {
        self.check_not_exited()?;

        if self.state != State::Paused {
            self
            .order_sender
            .send(Order::Pause)
            .map_err(|e| failure::err_msg(format!("Player::pause error : {}", e)))?;
        }
        Ok(self.state())
    }

    pub fn stop(&mut self) -> Result<State, failure::Error> {
        self.check_not_exited()?;

        if self.state != State::Stopped {
            self
                .order_sender
                .send(Order::Stop)
                .map_err(|e| failure::err_msg(format!("Player::stop error : {}", e)))?;
        }
        Ok(self.state())
    }

    pub fn record(&mut self) -> Result<State, failure::Error> {
        self.check_not_exited()?;

        if self.state != State::Recording {
            self
                .order_sender
                .send(Order::Record)
                .map_err(|e| failure::err_msg(format!("Player::record error : {}", e)))?;
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

    pub fn modify_band(&mut self, operation: &Operation) -> Result<State, failure::Error> {
        self.check_not_exited()?;

        self.order_sender
            .send(Order::ModifyBand(operation.clone()))
            .map_err(|e| failure::err_msg(format!("Player::modify_band error : {}", e)))?;

        Ok(self.state())
    }

    pub fn exit(&mut self) -> Result<State, failure::Error> {
        if self.state != State::Exited {
            self
                .order_sender
                .send(Order::Exit)
                .map_err(|e| failure::err_msg(format!("Player::exit error : {}", e)))?;
        }
        Ok(self.state())
    }
}
