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
// use std::fs;
use std::fs::File;
use std::io::Read;
use std::io::Write;

use talker::identifier;
use talker::talker::RTalker;
// use talker::talker::Talker;

// use crate::band::RBand;
use crate::band::{Band, Operation};
// use crate::mixer;
use crate::mixer::RMixer;
use crate::player::Player;
use crate::state::State;
// use crate::track;
// use crate::track::RTrack;
// use crate::track::Track;

pub const CONTENT_TYPE: &str = ".gsr";

pub struct Session {
    filename: String,
    band: Band,
    player: Player,
    player_synchronized: bool,
    start_tick: i64,
    end_tick: i64,
}

impl Session {
    pub fn new(band_description: String) -> Result<Session, failure::Error> {
        Ok(Self {
            filename: "NewBand.gsr".to_string(),
            band: Band::make(&band_description)?,
            player: Player::new(band_description)?,
            player_synchronized: false,
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
            player_synchronized: false,
            start_tick: 0,
            end_tick: 0,
        })
    }
    /*
        pub fn load_file(filename: &str) -> Result<Band, failure::Error> {
            let description_buffer = fs::read(filename)?;
            Band::make(&description_buffer)
        }
    */
    pub fn filename<'a>(&'a self) -> &'a str {
        &self.filename
    }

    pub fn talkers<'a>(&'a self) -> &'a HashMap<u32, RTalker> {
        self.band.talkers()
    }

    pub fn add_talker(&mut self, talker_model: &str) -> Result<(), failure::Error> {
        self.modify_band(&Operation::AddTalker(
            identifier::get_next_id(),
            talker_model.to_string(),
        ))
    }
    /*
        pub fn sup_talker(&mut self, talker: &RTalker) -> Result<(), failure::Error> {
            self.band.sup_talker(talker)
        }
    */
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
        self.synchronize_player()?;

        let state = self.player.set_time_range(self.start_tick, self.end_tick)?;
        Ok(state)
    }

    pub fn end_tick(&self) -> i64 {
        self.end_tick
    }

    pub fn set_end_tick(&mut self, t: i64) -> Result<State, failure::Error> {
        self.end_tick = t;
        self.synchronize_player()?;

        let state = self.player.set_time_range(self.start_tick, self.end_tick)?;
        Ok(state)
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
        let state = self.player.load_band(self.band.serialize()?)?;
        Ok(state)
    }

    pub fn modify_band(&mut self, operation: &Operation) -> Result<(), failure::Error> {
        self.player.modify_band(operation)?;
        self.band.modify(operation)?;
        Ok(())
    }

    pub fn save(&self) -> Result<(), failure::Error> {
        let mut file = File::create(&self.filename)?;

        writeln!(file, "{}", self.band.serialize()?)?;
        Ok(())
    }
    pub fn save_as(&mut self, filename: &str) -> Result<(), failure::Error> {
        self.filename = filename.to_string();
        self.save()?;
        Ok(())
    }

    fn synchronize_player(&mut self) -> Result<(), failure::Error> {
        if !self.player_synchronized {
            // TODO : synchronize player
            self.player_synchronized = true;
        }
        Ok(())
    }
    pub fn start(&mut self) -> Result<State, failure::Error> {
        self.synchronize_player()?;
        let state = self.player.start()?;
        Ok(state)
    }

    pub fn play(&mut self) -> Result<State, failure::Error> {
        self.synchronize_player()?;
        let state = self.player.play()?;
        Ok(state)
    }

    pub fn pause(&mut self) -> Result<State, failure::Error> {
        self.synchronize_player()?;
        let state = self.player.pause()?;
        Ok(state)
    }

    pub fn stop(&mut self) -> Result<State, failure::Error> {
        self.synchronize_player()?;
        let state = self.player.stop()?;
        Ok(state)
    }
}
