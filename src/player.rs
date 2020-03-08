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
// use gpplugin::ear::{Ear, Talk};
// use gpplugin::identifier::Identifier;
// use gpplugin::talker::{RTalker, Talker};

use crate::factory::Factory;
// use crate::mixer;
// use crate::mixer::Mixer;
// use crate::output;
// use crate::output::ROutput;
use crate::session::Session;
use crate::state::State;
// use crate::track;
// use crate::track::{RTrack, Track};

enum Order {
    Start,
    Play,
    Pause,
    Stop,
    //    None,
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
            session.add_playback(&factory);
            let mut res = Ok(());
            let mut tick: i64 = 0;
            let mut start_tick: i64 = 0;
            //            let mut end_tick: i64 = 0;
            let len = AudioFormat::chunk_size();

            let mut buf: Vec<f32> = vec![0.; len];

            let mut right: Vec<f32> = vec![0.; len];
            let mut left: Vec<f32> = vec![0.; len];

            let mut channels = Vec::with_capacity(2);
            channels.push(right);
            channels.push(left);

            session.open_outputs();
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
                match session.play_chunk(tick, &mut buf, &mut channels, len) {
                    Ok(ln) => {
                        tick += ln as i64;
                        if ln == 0 {
                            break;
                        }
                    }
                    Err(e) => {
                        res = Err(failure::err_msg(format!("Player::run error : {}", e)));
                        break;
                    }
                }
                order = receiver.try_recv();
            }

            session.deactivate_talkers();
            session.close_outputs();
            res
        });

        // self.sender = Some(sender);
        // self.join_handle = Some(join_handle);
        // self.state = State::Playing;
        // Ok(())
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

    //    pub fn run(&mut self, _factory: &Factory) -> Result<(), failure::Error> {}

    pub fn start(&mut self) -> Result<(), failure::Error> {
        self.sender
            .send(Order::Start)
            .map_err(|e| failure::err_msg(format!("Player::play error : {}", e)))?;

        self.state = State::Playing;
        thread::sleep(Duration::from_millis(1));
        Ok(())
    }
    /*
    pub fn pause(&mut self) -> Result<(), failure::Error> {
        let (state, res) = match self.state {
            State::Playing => (
                State::Playing,
                self.sender
                    .unwrap()
                    .send(Order::Start)
                    .map_err(|e| failure::err_msg(format!("Player::play error : {}", e))),
            ),
            State::Paused => (
                State::Playing,
                self.sender
                    .unwrap()
                    .send(Order::Play)
                    .map_err(|e| failure::err_msg(format!("Player::play error : {}", e))),
            ),
            State::Stopped => (State::Playing, self.run(factory)),
        };
        self.state = state;
        thread::sleep(Duration::from_millis(1));
        res
    }
        method pause (_:int) =
          ignore( match mState with
              | State.Playing -> Mutex.lock mPauseLock; mOrder <- Pause
              | State.Paused -> Mutex.unlock mPauseLock
              | State.Stopped -> ()
            );
          Thread.yield()
    */
    pub fn stop(&mut self) -> Result<(), failure::Error> {
        let res = match self.state {
            State::Playing => self
                .sender
                .send(Order::Stop)
                .map_err(|e| failure::err_msg(format!("Player::play error : {}", e))),
            State::Paused => self
                .sender
                .send(Order::Stop)
                .map_err(|e| failure::err_msg(format!("Player::play error : {}", e))),
            State::Stopped => Ok(()),
        };
        self.state = State::Stopped;
        thread::sleep(Duration::from_millis(1));
        res
    }
    pub fn exit(&mut self) -> Result<(), failure::Error> {
        self.sender
            .send(Order::Exit)
            .map_err(|e| failure::err_msg(format!("Player::play error : {}", e)))?;

        self.state = State::Stopped;
        thread::sleep(Duration::from_millis(1));
        Ok(())
    }
}
