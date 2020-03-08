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

use gpplugin::audio_format::AudioFormat;

use crate::factory::Factory;
use crate::session::Session;
use crate::state::State;

enum Order {
    Start,
    Play,
    Pause,
    Stop,
    Exit,
}

pub struct Player {
    filename: String,
    sender: Sender<Order>,
    join_handle: JoinHandle<Result<(), failure::Error>>,
    state: State,
}

pub type RPlayer = Rc<RefCell<Player>>;

impl Player {
    pub fn new(filename: &str) -> Player {
        let (sender, receiver): (Sender<Order>, Receiver<Order>) = std::sync::mpsc::channel();
        let fname = filename.to_string();

        let join_handle = thread::spawn(move || {
            let factory = Factory::new();
            let mut session = Session::load(&factory, &fname)?;
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

            session.open_outputs()?;
            session.activate_talkers();
            let mut order = Ok(Order::Stop);

            loop {
                match order {
                    Ok(Order::Start) => {
                        tick = start_tick;
                    }
                    Ok(Order::Pause) => match receiver.recv() {
                        Err(e) => {
                            res = Err(failure::err_msg(format!("Player::run error : {}", e)));
                            break;
                        }
                        Ok(Order::Exit) => break,
                        _ => (),
                    },
                    Ok(Order::Stop) => {
                        tick = start_tick;
                        match receiver.recv() {
                            Err(e) => {
                                res = Err(failure::err_msg(format!("Player::run error : {}", e)));
                                break;
                            }
                            Ok(Order::Exit) => break,
                            _ => (),
                        }
                    }
                    Ok(Order::Exit) => break,
                    _ => (),
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

                order = receiver.try_recv();
            }

            session.deactivate_talkers();
            session.close_outputs()?;
            res
        });

        Self {
            filename: filename.to_string(),
            sender,
            join_handle,
            state: State::Stopped,
        }
    }

    pub fn new_ref(filename: &str) -> RPlayer {
        Rc::new(RefCell::new(Player::new(filename)))
    }

    pub fn start(&mut self) -> Result<(), failure::Error> {
        self.sender
            .send(Order::Start)
            .map_err(|e| failure::err_msg(format!("Player::play error : {}", e)))?;

        self.state = State::Playing;
        thread::sleep(Duration::from_millis(1));
        Ok(())
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
