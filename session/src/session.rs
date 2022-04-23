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

use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::io::Write;

use talker::identifier;
use talker::talker::RTalker;

use crate::band::{Band, Operation};
use crate::mixer::RMixer;
use crate::player::Player;
use crate::state::State;

pub const CONTENT_TYPE: &str = ".gsr";

pub struct Session {
    filename: String,
    band: Band,
    player: Player,
    start_tick: i64,
    end_tick: i64,
}

impl Session {
    pub fn new(band_description: String) -> Result<Session, failure::Error> {
        Ok(Self {
            filename: "NewSession.gsr".to_string(),
            band: Band::make(&band_description)?,
            player: Player::new(band_description)?,
            start_tick: 0,
            end_tick: 0,
        })
    }

    pub fn from_file(filename: &str) -> Result<Session, failure::Error> {
        let mut band_description = String::new();

        let mut f = File::open(filename)?;
        f.read_to_string(&mut band_description)?;

        Ok(Self {
            filename: filename.to_string(),
            band: Band::make(&band_description)?,
            player: Player::new(band_description)?,
            start_tick: 0,
            end_tick: 0,
        })
    }

    pub fn filename<'a>(&'a self) -> &'a str {
        &self.filename
    }

    pub fn talkers<'a>(&'a self) -> &'a HashMap<u32, RTalker> {
        self.band.talkers()
    }

    pub fn add_talker(&mut self, talker_model: &str) -> Result<State, failure::Error> {
        self.modify_band(&Operation::AddTalker(
            identifier::get_next_id(),
            talker_model.to_string(),
        ))
    }

    pub fn mixers<'a>(&'a self) -> &'a HashMap<u32, RMixer> {
        self.band.mixers()
    }

    pub fn state(&mut self) -> State {
        self.player.state()
    }

    pub fn start_tick(&self) -> i64 {
        self.start_tick
    }

    pub fn set_start_tick(&mut self, t: i64) -> Result<State, failure::Error> {
        if self.start_tick == self.end_tick {
            self.start_tick = t;
            self.end_tick = t;
        } else {
            self.start_tick = t;
        }

        self.player.set_time_range(self.start_tick, self.end_tick)
    }

    pub fn end_tick(&self) -> i64 {
        self.end_tick
    }

    pub fn set_end_tick(&mut self, t: i64) -> Result<State, failure::Error> {
        self.end_tick = t;

        self.player.set_time_range(self.start_tick, self.end_tick)
    }

    pub fn player<'a>(&'a mut self) -> &'a Player {
        &self.player
    }
    pub fn new_band(&mut self) -> Result<(), failure::Error> {
        self.band = Band::empty();
        self.player = Player::new("".to_string())?;
        Ok(())
    }

    pub fn init(&mut self, band_description: String) -> Result<(), failure::Error> {
        self.band = Band::make(&band_description)?;
        self.player = Player::new(band_description)?;
        Ok(())
    }

    pub fn load_band(&self) -> Result<State, failure::Error> {
        self.player.load_band(self.band.serialize()?)
    }

    pub fn modify_band(&mut self, operation: &Operation) -> Result<State, failure::Error> {
        self.band.modify(operation)?;
        self.player.modify_band(operation)
    }

    pub fn save(&self) -> Result<(), failure::Error> {
        let mut file = File::create(&self.filename)?;

        writeln!(file, "{}", self.band.serialize()?)?;
        Ok(())
    }
    pub fn save_as(&mut self, filename: &str) -> Result<(), failure::Error> {
        self.filename = filename.to_string();
        self.save()
    }

    pub fn start(&mut self) -> Result<State, failure::Error> {
        self.player.start()
    }

    pub fn play(&mut self) -> Result<State, failure::Error> {
        self.player.play()
    }

    pub fn pause(&mut self) -> Result<State, failure::Error> {
        self.player.pause()
    }

    pub fn stop(&mut self) -> Result<State, failure::Error> {
        self.player.stop()
    }

    pub fn exit(&mut self) -> Result<State, failure::Error> {
        self.player.exit()
    }
}
