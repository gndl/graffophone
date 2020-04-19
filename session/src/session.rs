/*
 * Copyright (C) 2020 Gaëtan Dubreil
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

use crate::band::{Band, RBand};
use crate::event_bus::{EventBus, Notification, REventBus};
use crate::player::Player;
use crate::state::State;

pub struct Session {
    band: RBand,
    player: Player,
    player_synchronized: bool,
    start_tick: i64,
    end_tick: i64,
    event_bus: REventBus,
}

impl Session {
    pub fn new() -> Session {
        Self {
            band: Band::new_ref(None, None, None, None, None),
            player: Player::new("".to_string()).unwrap(),
            player_synchronized: false,
            start_tick: 0,
            end_tick: 0,
            event_bus: EventBus::new_ref(),
        }
    }

    pub fn state(&mut self) -> State {
        self.player.state()
    }

    pub fn event_bus<'a>(&'a self) -> &'a REventBus {
        &self.event_bus
    }

    pub fn start_tick(&self) -> i64 {
        self.start_tick
    }

    pub fn set_start_tick(&mut self, t: i64) -> Result<(), failure::Error> {
        if self.start_tick == self.end_tick {
            self.start_tick = t;
            self.end_tick = t;
            self.event_bus.borrow().notify(Notification::Tick(t));
        } else {
            self.start_tick = t;
            self.event_bus
                .borrow()
                .notify(Notification::TimeRange(t, self.end_tick));
        }
        self.synchronize_player()?;

        let state = self.player.set_time_range(self.start_tick, self.end_tick)?;
        self.event_bus.borrow().notify(Notification::State(state));
        Ok(())
    }

    pub fn end_tick(&self) -> i64 {
        self.end_tick
    }

    pub fn set_end_tick(&mut self, t: i64) -> Result<(), failure::Error> {
        self.end_tick = t;
        self.event_bus
            .borrow()
            .notify(Notification::TimeRange(self.start_tick, t));
        self.synchronize_player()?;

        let state = self.player.set_time_range(self.start_tick, self.end_tick)?;
        self.event_bus.borrow().notify(Notification::State(state));
        Ok(())
    }

    pub fn player<'a>(&'a mut self) -> &'a Player {
        &self.player
    }
    pub fn new_band(&mut self) -> Result<(), failure::Error> {
        self.band = Band::new_ref(None, None, None, None, None);
        self.player = Player::new("".to_string())?;
        Ok(())
    }

    pub fn init(&mut self, band_description: String) -> Result<(), failure::Error> {
        self.band = Band::make(band_description.as_ref(), false)?.to_ref();
        self.player = Player::new(band_description)?;
        Ok(())
    }

    fn synchronize_player(&mut self) -> Result<(), failure::Error> {
        if !self.player_synchronized {
            // TODO : synchronize player
            self.player_synchronized = true;
        }
        Ok(())
    }
    pub fn start(&mut self) -> Result<(), failure::Error> {
        self.synchronize_player()?;
        let state = self.player.start()?;
        self.event_bus.borrow().notify(Notification::State(state));
        Ok(())
    }

    pub fn play(&mut self) -> Result<(), failure::Error> {
        self.synchronize_player()?;
        let state = self.player.play()?;
        self.event_bus.borrow().notify(Notification::State(state));
        Ok(())
    }

    pub fn pause(&mut self) -> Result<(), failure::Error> {
        self.synchronize_player()?;
        let state = self.player.pause()?;
        self.event_bus.borrow().notify(Notification::State(state));
        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), failure::Error> {
        self.synchronize_player()?;
        let state = self.player.stop()?;
        self.event_bus.borrow().notify(Notification::State(state));
        Ok(())
    }
}
/*
#[no_mangle]
pub extern "C" fn gramotor_gramotor_new() -> Box<Session> {
    Box::new(Session::new().unwrap())
}

#[no_mangle]
pub extern "C" fn gramotor_gramotor_play(motor: &mut Session) {
    let _ = motor.play();
}

#[no_mangle]
pub extern "C" fn gramotor_gramotor_drop(_motor: Box<Session>) {}
*/
