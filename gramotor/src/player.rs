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
use std::thread::JoinHandle;
use std::time::Duration;

use granode::audio_format::AudioFormat;

use crate::factory::Factory;
use crate::session::Session;
use crate::state::State;

enum Order {
    Start,
    Play,
    Pause,
    Stop,
    SetTimeRange(i64, i64),
    Exit,
}

pub struct Player {
    sender: Sender<Order>,
    join_handle: JoinHandle<Result<(), failure::Error>>,
    state: State,
}

pub type RPlayer = Rc<RefCell<Player>>;

impl Player {
    pub fn new(filename: &str) -> Result<Player, failure::Error> {
        let (sender, receiver): (Sender<Order>, Receiver<Order>) = std::sync::mpsc::channel();
        let fname = filename.to_string();

        let join_handle = thread::spawn(move || {
            let factory = Factory::new();
            let mut session = Session::load_file(&factory, &fname)?;
            session.add_playback(&factory)?;
            let mut res = Ok(());
            let mut tick: i64 = 0;
            let mut start_tick: i64 = 0;
            let mut end_tick: i64 = i64::max_value();
            let chunk_size = AudioFormat::chunk_size();

            let mut buf: Vec<f32> = vec![0.; chunk_size];

            let nb_channels = session.nb_channels();
            let mut channels: Vec<Vec<f32>> = Vec::with_capacity(nb_channels);

            for _ in 0..nb_channels {
                channels.push(vec![0.; chunk_size]);
            }

            session.open()?;

            let mut state = State::Stopped;
            let mut order = Ok(Order::Stop);

            loop {
                match order {
                    Ok(Order::Start) => {
                        state = State::Playing;
                        tick = start_tick;
                    }
                    Ok(Order::Pause) => {
                        state = State::Paused;
                        session.pause()?;
                        order = receiver.recv();
                        continue;
                    }
                    Ok(Order::Play) => {
                        state = State::Playing;
                        session.run()?;
                    }
                    Ok(Order::Stop) => {
                        state = State::Stopped;
                        tick = start_tick;
                        order = receiver.recv();
                        continue;
                    }
                    Ok(Order::SetTimeRange(start, end)) => {
                        start_tick = start;
                        end_tick = end;

                        if state != State::Playing {
                            order = Ok(Order::Pause);
                            continue;
                        }
                    }
                    Ok(Order::Exit) => break,
                    Err(e) => {
                        res = Err(failure::err_msg(format!("Player::run error : {}", e)));
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

                for rmixer in session.mixers().values() {
                    match rmixer
                        .borrow_mut()
                        .come_out(tick, &mut buf, &mut channels, len)
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

                match receiver.try_recv() {
                    Err(_) => {
                        order = Ok(Order::Play);
                    }
                    Ok(o) => {
                        order = Ok(o);
                    }
                }
            }

            session.close()?;
            res
        });

        Ok(Self {
            sender,
            join_handle,
            state: State::Stopped,
        })
    }

    pub fn new_ref(filename: &str) -> Result<RPlayer, failure::Error> {
        Ok(Rc::new(RefCell::new(Player::new(filename)?)))
    }

    pub fn state<'a>(&'a self) -> &'a State {
        &self.state
    }

    pub fn start(&mut self) -> Result<(), failure::Error> {
        self.sender
            .send(Order::Start)
            .map_err(|e| failure::err_msg(format!("Player::play error : {}", e)))?;

        self.state = State::Playing;
        thread::sleep(Duration::from_millis(1));
        Ok(())
    }

    pub fn play(&mut self) -> Result<(), failure::Error> {
        let (state, res) = match self.state {
            State::Playing => (State::Playing, Ok(())),
            _ => (
                State::Playing,
                self.sender
                    .send(Order::Play)
                    .map_err(|e| failure::err_msg(format!("Player::play error : {}", e))),
            ),
        };
        self.state = state;
        thread::sleep(Duration::from_millis(1));
        res
    }

    pub fn pause(&mut self) -> Result<(), failure::Error> {
        let (state, res) = match self.state {
            State::Playing => (
                State::Paused,
                self.sender
                    .send(Order::Pause)
                    .map_err(|e| failure::err_msg(format!("Player::play error : {}", e))),
            ),
            State::Paused => (
                State::Playing,
                self.sender
                    .send(Order::Play)
                    .map_err(|e| failure::err_msg(format!("Player::play error : {}", e))),
            ),
            State::Stopped => (State::Stopped, Ok(())),
        };
        self.state = state;
        thread::sleep(Duration::from_millis(1));
        res
    }

    pub fn stop(&mut self) -> Result<(), failure::Error> {
        match self.state {
            State::Stopped => {}
            _ => self
                .sender
                .send(Order::Stop)
                .map_err(|e| failure::err_msg(format!("Player::play error : {}", e)))?,
        }
        self.state = State::Stopped;
        thread::sleep(Duration::from_millis(1));
        Ok(())
    }

    pub fn set_time_range(&mut self, start_tick: i64, end_tick: i64) -> Result<(), failure::Error> {
        self.sender
            .send(Order::SetTimeRange(start_tick, end_tick))
            .map_err(|e| failure::err_msg(format!("Player::set_time_range error : {}", e)))?;

        thread::sleep(Duration::from_millis(1));
        Ok(())
    }
    /*
        pub fn send_order(&mut self, order: Order) -> Result<(), failure::Error> {
            self.sender
                .send(order)
                .map_err(|e| failure::err_msg(format!("Player::send_order error : {}", e)))?;
            thread::sleep(Duration::from_millis(1));
            Ok(())
        }
    */
    pub fn exit(&mut self) -> Result<(), failure::Error> {
        self.sender
            .send(Order::Exit)
            .map_err(|e| failure::err_msg(format!("Player::exit error : {}", e)))?;

        // self.join_handle
        //     .join()
        //     .map_err(|e| failure::err_msg(format!("Player::join error : {:?}", e)))?;

        thread::sleep(Duration::from_millis(50));

        self.state = State::Stopped;
        Ok(())
    }
}
