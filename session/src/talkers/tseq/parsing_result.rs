use std::collections::HashMap;
use std::f32;

use talkers::tseq::parser::{
    PBeat, PDurationLine, PHitLine, PSequence, PTransition, PVelocityLine,
};

pub struct ParsingResult<'a> {
    pub beats: HashMap<&'a str, &'a PBeat<'a>>,
    pub pitchlines: HashMap<&'a str, Vec<(f32, PTransition)>>,
    pub hitlines: HashMap<&'a str, &'a PHitLine<'a>>,
    pub durationlines: HashMap<&'a str, &'a PDurationLine<'a>>,
    pub velocitylines: HashMap<&'a str, &'a PVelocityLine<'a>>,
    pub sequences: HashMap<&'a str, &'a PSequence<'a>>,
}

impl<'a> ParsingResult<'a> {
    pub fn new() -> ParsingResult<'a> {
        Self {
            beats: HashMap::new(),
            pitchlines: HashMap::new(),
            hitlines: HashMap::new(),
            durationlines: HashMap::new(),
            velocitylines: HashMap::new(),
            sequences: HashMap::new(),
        }
    }
    pub fn fetch_beat(&'a self, id: &str) -> Result<&'a PBeat, failure::Error> {
        match self.beats.get(id) {
            Some(e) => Ok(e),
            None => Err(failure::err_msg(format!("Tseq beat {} not found!", id))),
        }
    }
    pub fn fetch_pitchline(
        &'a self,
        id: &str,
    ) -> Result<&'a Vec<(f32, PTransition)>, failure::Error> {
        match self.pitchlines.get(id) {
            Some(e) => Ok(e),
            None => Err(failure::err_msg(format!(
                "Tseq pitchline {} not found!",
                id
            ))),
        }
    }
    pub fn fetch_hitline(&'a self, id: &str) -> Result<&'a PHitLine, failure::Error> {
        match self.hitlines.get(id) {
            Some(e) => Ok(e),
            None => Err(failure::err_msg(format!("Tseq pattern {} not found!", id))),
        }
    }
    pub fn fetch_durationline(&'a self, id: &str) -> Result<&'a PDurationLine, failure::Error> {
        match self.durationlines.get(id) {
            Some(e) => Ok(e),
            None => Err(failure::err_msg(format!(
                "Tseq durationline {} not found!",
                id
            ))),
        }
    }
    pub fn fetch_velocityline(&'a self, id: &str) -> Result<&'a PVelocityLine, failure::Error> {
        match self.velocitylines.get(id) {
            Some(e) => Ok(e),
            None => Err(failure::err_msg(format!(
                "Tseq velocityline {} not found!",
                id
            ))),
        }
    }
    pub fn fetch_sequence(&'a self, id: &str) -> Result<&'a PSequence, failure::Error> {
        match self.sequences.get(id) {
            Some(e) => Ok(e),
            None => Err(failure::err_msg(format!("Tseq sequence {} not found!", id))),
        }
    }
}
