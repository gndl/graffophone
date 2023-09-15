use std::collections::HashMap;
use std::f32;

use talker::audio_format::AudioFormat;
use talkers::tseq::audio_event;
use talkers::tseq::parser::{
    PBeat, PChord, PChordLine, PDurationLine, PHitLine, PPitchLine, PSequence, PTransition,
    PVelocityLine,
};
use talkers::tseq::scale::Scale;

pub struct Harmonic {
    pub freq_ratio: f32,
    pub delay_ticks: i64,
    pub velocity: f32,
    pub velocity_transition: PTransition,
}
const DEFAULT_CHORD: Harmonic = Harmonic {
    freq_ratio: 1.,
    delay_ticks: 0,
    velocity: 1.,
    velocity_transition: PTransition::None,
};

pub struct Binder<'a> {
    pub beats: HashMap<&'a str, &'a PBeat<'a>>,
    pub chords: HashMap<&'a str, &'a PChord<'a>>,
    pub chordlines: Vec<&'a PChordLine<'a>>,
    default_chordline: Vec<Vec<Harmonic>>,
    deserialized_chordlines: HashMap<&'a str, Vec<Vec<Harmonic>>>,
    pub durationlines: HashMap<&'a str, &'a PDurationLine<'a>>,
    pub velocitylines: HashMap<&'a str, &'a PVelocityLine<'a>>,
    pub hitlines: HashMap<&'a str, &'a PHitLine<'a>>,
    pub pitchlines: Vec<&'a PPitchLine<'a>>,
    deserialized_pitchlines: HashMap<&'a str, Vec<(f32, PTransition)>>,
    pub sequences: HashMap<&'a str, &'a PSequence<'a>>,
}

impl<'a> Binder<'a> {
    pub fn new() -> Binder<'a> {
        Self {
            beats: HashMap::new(),
            chords: HashMap::new(),
            chordlines: Vec::new(),
            default_chordline: vec![vec![DEFAULT_CHORD]],
            deserialized_chordlines: HashMap::new(),
            durationlines: HashMap::new(),
            velocitylines: HashMap::new(),
            hitlines: HashMap::new(),
            pitchlines: Vec::new(),
            deserialized_pitchlines: HashMap::new(),
            sequences: HashMap::new(),
        }
    }

    pub fn deserialize(&mut self) -> Result<(), failure::Error> {
        let scale = Scale::tempered();

        for pitchline in &self.pitchlines {
            let mut pitchs = Vec::new();
            for pitch in &pitchline.pitchs {
                pitchs.push((scale.fetch_frequency(pitch.id)?, pitch.transition));
            }
            self.deserialized_pitchlines.insert(pitchline.id, pitchs);
        }

        let sample_rate = AudioFormat::sample_rate() as f32;

        for chordline in &self.chordlines {
            let mut deserialized_chordline = Vec::new();

            for chord_id in &chordline.chords {
                match self.chords.get(chord_id) {
                    Some(chord) => {
                        let mut deserialized_chord = Vec::new();

                        for pharmonic in &chord.harmonics {
                            let mut velocity = audio_event::DEFAULT_VELOCITY;
                            let mut velocity_transition = PTransition::None;

                            if let Some(pvelocity) = &pharmonic.velocity {
                                velocity = pvelocity.value;
                                velocity_transition = pvelocity.transition;
                            }

                            let harmonic = Harmonic {
                                freq_ratio: pharmonic.freq_ratio.num / pharmonic.freq_ratio.den,
                                delay_ticks: (pharmonic.delay.unwrap_or(0.) * sample_rate) as i64,
                                velocity,
                                velocity_transition,
                            };
                            deserialized_chord.push(harmonic);
                        }
                        deserialized_chordline.push(deserialized_chord);
                    }
                    None => {
                        return Err(failure::err_msg(format!(
                            "Tseq chord {} not found!",
                            chord_id
                        )))
                    }
                }
            }
            self.deserialized_chordlines
                .insert(chordline.id, deserialized_chordline);
        }
        Ok(())
    }

    pub fn fetch_beat(&'a self, id: &str) -> Result<&'a PBeat, failure::Error> {
        match self.beats.get(id) {
            Some(e) => Ok(e),
            None => Err(failure::err_msg(format!("Tseq beat {} not found!", id))),
        }
    }
    pub fn fetch_durationline(&'a self, id: &str) -> Result<&'a PDurationLine, failure::Error> {
        match self.durationlines.get(id) {
            Some(e) => Ok(e),
            None => Err(failure::err_msg(format!(
                "Tseq durations {} not found!",
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
    pub fn fetch_chord(&'a self, id: &str) -> Result<&'a PChord, failure::Error> {
        match self.chords.get(id) {
            Some(e) => Ok(e),
            None => Err(failure::err_msg(format!("Tseq chord {} not found!", id))),
        }
    }

    pub fn fetch_deserialized_chordline(
        &'a self,
        oid: &'a Option<&str>,
    ) -> Result<&'a Vec<Vec<Harmonic>>, failure::Error> {
        match oid {
            Some(id) => match self.deserialized_chordlines.get(id) {
                Some(deserialized_chordline) => Ok(deserialized_chordline),
                None => Err(failure::err_msg(format!("Tseq chords {} not found!", id))),
            },
            None => Ok(&self.default_chordline),
        }
    }

    pub fn fetch_hitline(&'a self, id: &str) -> Result<&'a PHitLine, failure::Error> {
        match self.hitlines.get(id) {
            Some(e) => Ok(e),
            None => Err(failure::err_msg(format!("Tseq hits {} not found!", id))),
        }
    }
    pub fn fetch_deserialized_pitchline(
        &'a self,
        id: &str,
    ) -> Result<&'a Vec<(f32, PTransition)>, failure::Error> {
        match self.deserialized_pitchlines.get(id) {
            Some(e) => Ok(e),
            None => Err(failure::err_msg(format!("Tseq pitchs {} not found!", id))),
        }
    }
    pub fn fetch_sequence(&'a self, id: &str) -> Result<&'a PSequence, failure::Error> {
        match self.sequences.get(id) {
            Some(e) => Ok(e),
            None => Err(failure::err_msg(format!("Tseq seq {} not found!", id))),
        }
    }
}
