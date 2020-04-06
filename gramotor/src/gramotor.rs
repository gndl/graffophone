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

use crate::event_bus::{EventBus, Notification, REventBus};
use crate::player::Player;
use crate::session::{RSession, Session};
use crate::state::State;

pub struct Gramotor {
    session: RSession,
    player: Player,
    player_synchronized: bool,
    start_tick: i64,
    end_tick: i64,
    bus: REventBus,
}

impl Gramotor {
    pub fn new() -> Gramotor {
Self {
            session: Session::new_ref(None, None, None, None, None),
            player: Player::new("".to_string()).unwrap(),
            player_synchronized: false,
            start_tick: 0,
            end_tick: 0,
            bus: EventBus::new_ref(),
        }
    }

    pub fn state<'a>(&'a self) -> &'a State {
        self.player.state()
    }

    pub fn start_tick(&self) -> i64 {
        self.start_tick
    }

    pub fn set_start_tick(&mut self, t: i64) -> Result<(), failure::Error> {
        if self.start_tick == self.end_tick {
            self.start_tick = t;
            self.end_tick = t;
            self.bus.notify(Notification::Tick(t));
        } else {
            self.start_tick = t;
            self.bus.notify(Notification::TimeRange(t, self.end_tick));
        }
        self.synchronize_player()?;
        self.player.set_time_range(self.start_tick, self.end_tick)
    }

    pub fn end_tick(&self) -> i64 {
        self.end_tick
    }

    pub fn set_end_tick(&mut self, t: i64) -> Result<(), failure::Error> {
        self.end_tick = t;
        self.bus.notify(Notification::TimeRange(self.start_tick, t));
        self.synchronize_player()?;
        self.player.set_time_range(self.start_tick, self.end_tick)
    }

    pub fn player<'a>(&'a mut self) -> &'a Player {
        &self.player
    }
    pub fn new_session(&mut self) -> Result<(), failure::Error> {
        self.session = Session::new_ref(None, None, None, None, None);
        self.player = Player::new("".to_string())?;
        Ok(())
    }

    pub fn init_session(&mut self, session_description: String) -> Result<(), failure::Error> {
        self.session = Session::make(session_description.as_ref(), false)?.to_ref();
        self.player = Player::new(session_description)?;
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
        self.player.start()
    }

    pub fn play(&mut self) -> Result<(), failure::Error> {
        self.synchronize_player()?;
        self.player.play()
    }

    pub fn pause(&mut self) -> Result<(), failure::Error> {
        self.synchronize_player()?;
        self.player.pause()
    }

    pub fn stop(&mut self) -> Result<(), failure::Error> {
        self.synchronize_player()?;
        self.player.stop()
    }
}
/*
#[no_mangle]
pub extern "C" fn gramotor_gramotor_new() -> Box<Gramotor> {
    Box::new(Gramotor::new().unwrap())
}

#[no_mangle]
pub extern "C" fn gramotor_gramotor_play(motor: &mut Gramotor) {
    let _ = motor.play();
}

#[no_mangle]
pub extern "C" fn gramotor_gramotor_drop(_motor: Box<Gramotor>) {}
*/
