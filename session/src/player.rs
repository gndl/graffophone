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

use luil::plugin_handle::UiConnector;

use talker::audio_format::AudioFormat;
use talker::lv2_handler;
use talker::identifier::{Id, Index};

use crate::band::{Band, Operation};
use crate::feedback::Feedback;
use crate::output::Output;
use crate::state::State;
use crate::plugin_handle_manager::PluginHandleManager;

const RECEIVE_TIMEOUT: u64 = 10;

//#[derive(PartialEq, Debug, Clone)]
enum Order {
    Nil,
    Play,
    Record,
    Pause,
    Stop,
    SetTimeRange(i64, i64),
    LoadBand(String),
    ModifyBand(Operation),
    AddPluginHandle(Id, UiConnector),
    BandModificationsAndUiCount,
    State,
    Exit,
}

enum Response {
    State(State),
    BandModificationsAndUiCount(Vec<Operation>, usize),
}

fn state_order(state: State) -> Order {
    match state {
        State::Playing => Order::Play,
        State::Recording => Order::Record,
        State::Paused => Order::Pause,
        State::Stopped => Order::Stop,
        State::Exited => Order::Exit,
    }
}

struct Runner {
    order_receiver: Receiver<Order>,
    response_sender: Sender<Response>,
    plugin_handle_manager: PluginHandleManager,
}

impl Runner {
    fn new(
        order_receiver: Receiver<Order>,
        response_sender: Sender<Response>,
    ) -> Runner {
        let plugin_handle_manager = PluginHandleManager::new();

        Runner { order_receiver, response_sender, plugin_handle_manager }
    }

    fn send_state(&self, state: State) -> State {
        match self.response_sender.send(Response::State(state)) {
            Err(e) => eprintln!("Player state sender error : {}", e),
            Ok(()) => (),
        }
        state
    }

    fn start(&mut self, band_description: String) -> Result<(), failure::Error> {

        let res = self.run(band_description);

        let _ = self.response_sender.send(Response::State(State::Exited));

        res.map_err(|e| {
            let msg = format!("Player::run error : {}", e);
            println!("{}", msg);
            failure::err_msg(msg)
        })
    }

    fn run(&mut self, band_description: String) -> Result<(), failure::Error> {
        let mut tick: i64 = 0;
        let mut start_tick: i64 = 0;
        let mut end_tick: i64 = i64::max_value();

        let chunk_size = AudioFormat::chunk_size();
        let mut feedback = Feedback::new(chunk_size)?;

        let mut band = Band::make(&band_description, true)?;
        let feedback_mixer_id = band.mixers().iter().next().map_or(0, |(k, _)| *k);

        let mut state = State::Stopped;
        let mut order = self.wait_order()?;

        loop {
            match order {
                Order::Nil => {}
                Order::BandModificationsAndUiCount => {
                    let (operations, ui_count) = self.plugin_handle_manager.band_modifications_and_ui_count();
                    let _ = self.response_sender.send(Response::BandModificationsAndUiCount(operations, ui_count));

                    order = state_order(state);
                    continue;
                }
                Order::Pause => {
                    self.send_state(State::Paused);
                    
                    if state == State::Playing || state == State::Recording {
                        let len = band.play(tick, feedback.fade_len())?;
                        
                        if let Some(mxr) = band.find_mixer(feedback_mixer_id) {
                            feedback.write_fadeout(mxr.borrow().channels_buffers(), len)?;
                        }
                        
                        tick += len as i64;
                        
                        band.pause()?;
                    }
                    state = State::Paused;
                    order = self.wait_order()?;
                    continue;
                }
                Order::Play => {
                    self.send_state(State::Playing);
                    
                    if state == State::Stopped {
                        band.open()?;
                        feedback.open()?;
                    }
                    else if state == State::Paused {
                        band.run()?;
                        
                        let len = band.play(tick, feedback.fade_len())?;
                        
                        if let Some(mxr) = band.find_mixer(feedback_mixer_id) {
                            feedback.write_fadein(mxr.borrow().channels_buffers(), len)?;
                        }

                        tick += len as i64;
                    }

                    state = State::Playing;
                }
                Order::Record => {
                    self.send_state(State::Recording);

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
                    self.send_state(State::Stopped);

                    if state != State::Stopped {
                        let len = band.fadeout(tick)?;

                        if state == State::Playing || state == State::Recording {
                            if let Some(mxr) = band.find_mixer(feedback_mixer_id) {
                                feedback.write(mxr.borrow().channels_buffers(), len)?;
                            }
                        }
                        band.close()?;
                        feedback.close()?;
                        band.set_record(false)?;
                        tick = start_tick;
                        state = State::Stopped;
                    }
                    order = self.wait_order()?;
                    continue;
                }
                Order::SetTimeRange(start, end) => {
                    start_tick = start;
                    end_tick = end;
                    
                    order = state_order(state);
                    continue;
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
                            band = Band::make(&band_desc, true)?;
                            band.open()?;
                        }
                        State::Stopped => {
                            band = Band::make(&band_desc, true)?;
                        }
                        State::Exited => (),
                    }
                    order = state_order(state);
                    continue;
                }
                Order::ModifyBand(operation) => {
                    band.modify(&operation)?;

                    order = state_order(state);
                    continue;
                }
                Order::AddPluginHandle(talker_id, ui_connector) => {
                    self.plugin_handle_manager.add_plugin_handle(talker_id, ui_connector, &mut band)?;
                    
                    order = state_order(state);
                    continue;
                }
                Order::State => {
                    order = state_order(state);
                    continue;
                }
                Order::Exit => break,
            }
            
            // Run LV2 workers
            lv2_handler::run_workers()?;
            
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
            
            if self.plugin_handle_manager.has_handle() {
                self.plugin_handle_manager.run()?;
            }

            tick += len as i64;

            match self.order_receiver.try_recv() {
                Err(_) => {
                    order = Order::Nil;
                }
                Ok(o) => {
                    order = o;
                }
            }
        }
        Ok(())
    }

    fn wait_order(&mut self) -> Result<Order, failure::Error> {

        if self.plugin_handle_manager.has_handle() {
            loop {
                self.plugin_handle_manager.run_idle()?;

                if self.plugin_handle_manager.has_handle() {
                    match self.order_receiver.try_recv() {
                        Ok(o) => return Ok(o),
                        Err(_) => {}
                    }
                }
                else {
                    return self.wait_order();
                }
            }
        }
        else {
            self.order_receiver.recv().map_err(|e| failure::err_msg(format!("Player::wait_order error : {}", e)))
        }
    }
}

pub struct Player {
    order_sender: Sender<Order>,
    response_receiver: Receiver<Response>,
    state: State,
}
pub type RPlayer = Rc<RefCell<Player>>;

impl Player {
    pub fn new(band_description: String) -> Result<Player, failure::Error> {
        let (order_sender, order_receiver): (Sender<Order>, Receiver<Order>) =
            std::sync::mpsc::channel();
        let (response_sender, response_receiver): (Sender<Response>, Receiver<Response>) =
            std::sync::mpsc::channel();

        let state = if band_description.is_empty() {
            State::Exited
        } else {
            let _join_handle = thread::spawn(move || {
                let mut runner = Runner::new(order_receiver, response_sender);

                runner.start(band_description)
            });
            State::Stopped
        };
        Ok(Self {
            order_sender,
            response_receiver,
            state,
        })
    }

    fn receive_state(&mut self) -> State {
        match self.state {
            State::Exited => {}
            _ => {
                match self.response_receiver.recv_timeout(Duration::from_secs(RECEIVE_TIMEOUT)) {
                    Err(e) => println!("Player state error : {}", e),
                    Ok(response) => {
                        match response {
                            Response::State(state) => self.state = state,
                            _ => println!("Unexpected player response"),
                        }
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
                match self.response_receiver.recv_timeout(Duration::from_secs(RECEIVE_TIMEOUT)) {
                    Err(e) => {
                        return Err(failure::err_msg(format!("Player::play error : {}", e)));
                    }
                    Ok(response) => {
                        match response {
                            Response::State(state) => self.state = state,
                            _ => println!("Unexpected player response"),
                        }
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

    pub fn state(&mut self) -> State {
        let _ = self.order_sender.send(Order::State);

        self.receive_state()
    }

    pub fn play(&mut self) -> Result<State, failure::Error> {
        self.check_not_exited()?;

        if self.state != State::Playing {
            self
                .order_sender
                .send(Order::Play)
                .map_err(|e| failure::err_msg(format!("Player::play error : {}", e)))?;
        }
        Ok(self.receive_state())
    }

    pub fn pause(&mut self) -> Result<State, failure::Error> {
        self.check_not_exited()?;

        if self.state != State::Paused {
            self
            .order_sender
            .send(Order::Pause)
            .map_err(|e| failure::err_msg(format!("Player::pause error : {}", e)))?;
        }
        Ok(self.receive_state())
    }

    pub fn stop(&mut self) -> Result<State, failure::Error> {
        self.check_not_exited()?;

        if self.state != State::Stopped {
            self
                .order_sender
                .send(Order::Stop)
                .map_err(|e| failure::err_msg(format!("Player::stop error : {}", e)))?;
        }
        Ok(self.receive_state())
    }

    pub fn record(&mut self) -> Result<State, failure::Error> {
        self.check_not_exited()?;

        if self.state != State::Recording {
            self
                .order_sender
                .send(Order::Record)
                .map_err(|e| failure::err_msg(format!("Player::record error : {}", e)))?;
        }
        Ok(self.receive_state())
    }

    pub fn set_time_range(&mut self, start_tick: i64, end_tick: i64) -> Result<State, failure::Error> {
        self.check_not_exited()?;

        self.order_sender
            .send(Order::SetTimeRange(start_tick, end_tick))
            .map_err(|e| failure::err_msg(format!("Player::set_time_range error : {}", e)))?;

        Ok(self.receive_state())
    }

    pub fn load_band(&mut self, band_description: String) -> Result<State, failure::Error> {
        self.check_not_exited()?;

        self.order_sender
            .send(Order::LoadBand(band_description))
            .map_err(|e| failure::err_msg(format!("Player::load_band error : {}", e)))?;

        Ok(self.receive_state())
    }

    pub fn modify_band(&mut self, operation: &Operation) -> Result<State, failure::Error> {
        self.check_not_exited()?;

        self.order_sender
            .send(Order::ModifyBand(operation.clone()))
            .map_err(|e| failure::err_msg(format!("Player::modify_band error : {}", e)))?;

        Ok(self.receive_state())
    }

    pub fn add_plugin_handle(&mut self, talker_id: Id, ui_connector: UiConnector) -> Result<State, failure::Error> {
        self.check_not_exited()?;

        self.order_sender
            .send(Order::AddPluginHandle(talker_id, ui_connector))
            .map_err(|e| failure::err_msg(format!("Player::set_time_range error : {}", e)))?;

        Ok(self.receive_state())
    }

    pub fn band_modifications_and_ui_count(&mut self) -> Result<(Vec<Operation>, usize), failure::Error> {
        self.check_not_exited()?;

        self.order_sender
            .send(Order::BandModificationsAndUiCount)
            .map_err(|e| failure::err_msg(format!("Player::band_modifications_and_ui_count order error : {}", e)))?;
        
        let res = self.response_receiver.recv_timeout(Duration::from_secs(RECEIVE_TIMEOUT));
        let _ = self.receive_state();
                
        let response = res.map_err(|e| failure::err_msg(format!("Player::band_modifications_and_ui_count response error : {}", e)))?;

        match response {
            Response::BandModificationsAndUiCount(operations, ui_count) => Ok((operations, ui_count)),
            _ => Err(failure::err_msg("Player::band_modifications_and_ui_count response error : Unexpected player state response")),
        }
    }

    pub fn exit(&mut self) -> Result<State, failure::Error> {
        if self.state != State::Exited {
            self
                .order_sender
                .send(Order::Exit)
                .map_err(|e| failure::err_msg(format!("Player::exit error : {}", e)))?;
        }
        Ok(self.receive_state())
    }
}
