use std::collections::HashMap;
use std::f32;
use std::str::FromStr;

use talker::audio_format::AudioFormat;
use talkers::tseq::audio_event;
use talkers::tseq::parser::{
    PAttack, PBeat, PChord, PChordLine, PDurationLine, PHit, PHitLine, PPitchGap, PPitchLine,
    PSequence, PShape, PTime, PVelocity, PVelocityLine,
};
use talkers::tseq::scale;
use talkers::tseq::scale::RScale;

pub const UNDEFINED_TICKS: i64 = i64::MAX;

#[derive(Clone, Copy)]
pub enum Time {
    Rate(f32),
    Ticks(i64),
}

fn to_time(ptime: &PTime, ticks_per_second: f32) -> Time {
    match ptime {
        PTime::Rate(v) => Time::Rate(v.num / v.den),
        PTime::Second(v) => Time::Ticks((ticks_per_second * v.num / v.den) as i64),
    }
}

pub fn to_ticks(time: &Time, rate_uq: f32) -> i64 {
    match time {
        Time::Rate(r) => (rate_uq * r) as i64,
        Time::Ticks(t) => *t,
    }
}

pub fn option_to_ticks(otime: &Option<Time>, ofset: i64, rate_uq: f32) -> i64 {
    match otime {
        None => UNDEFINED_TICKS,
        Some(time) => ofset + to_ticks(time, rate_uq),
    }
}

pub struct Harmonic {
    pub freq_ratio: f32,
    pub delay: Time,
    pub velocity: PVelocity,
}
const DEFAULT_CHORD: Harmonic = Harmonic {
    freq_ratio: 1.,
    delay: Time::Ticks(0),
    velocity: PVelocity {
        level: 1.,
        fadein: false,
        fadeout: false,
        transition: PShape::None,
    },
};

pub struct Hit {
    pub position: Time,
    pub duration: Option<Time>,
}
fn to_hit(phit: &PHit, ticks_per_second: f32) -> Hit {
    Hit {
        position: to_time(&phit.position, ticks_per_second),
        duration: phit.duration.as_ref().map(|d| to_time(d, ticks_per_second)),
    }
}

pub struct HitLine {
    pub hits: Vec<Hit>,
    pub duration: Time,
}
fn to_hitline(phitline: &PHitLine, ticks_per_second: f32) -> HitLine {
    HitLine {
        hits: phitline
            .hits
            .iter()
            .map(|ph| to_hit(ph, ticks_per_second))
            .collect(),
        duration: to_time(&phitline.duration, ticks_per_second),
    }
}

fn to_freq_ratio(pitch_gap: &PPitchGap, scale: &RScale) -> f32 {
    match pitch_gap {
        PPitchGap::FreqRatio(r) => r.num / r.den,
        PPitchGap::Interval(i) => scale.frequency_ratio(*i),
    }
}

pub struct DurationLine {
    pub durations: Vec<Time>,
}
fn to_durationline(pdurationline: &PDurationLine, ticks_per_second: f32) -> DurationLine {
    DurationLine {
        durations: pdurationline
            .durations
            .iter()
            .map(|pt| to_time(pt, ticks_per_second))
            .collect(),
    }
}

pub struct Binder<'a> {
    pub ticks_per_second: f32,
    pub ticks_per_minute: f32,
    pub parser_beats: HashMap<&'a str, &'a PBeat<'a>>,
    pub envelops_indexes: HashMap<&'a str, usize>,
    pub no_envelop: usize,
    pub parser_chords: HashMap<&'a str, &'a PChord<'a>>,
    pub parser_attacks: HashMap<&'a str, &'a PAttack<'a>>,
    pub parser_chordlines: Vec<&'a PChordLine<'a>>,
    default_chordline: Vec<Vec<Harmonic>>,
    chordlines: HashMap<&'a str, Vec<Vec<Harmonic>>>,
    pub parser_durationlines: Vec<&'a PDurationLine<'a>>,
    pub durationlines: HashMap<&'a str, DurationLine>,
    pub parser_velocitylines: HashMap<&'a str, &'a PVelocityLine<'a>>,
    default_velocityline: PVelocityLine<'a>,
    pub parser_hitlines: Vec<&'a PHitLine<'a>>,
    pub hitlines: HashMap<&'a str, HitLine>,
    pub parser_pitchlines: Vec<&'a PPitchLine<'a>>,
    pitchlines: HashMap<&'a str, Vec<(f32, PShape)>>,
    pub parser_sequences: HashMap<&'a str, &'a PSequence<'a>>,
}

impl<'a> Binder<'a> {
    pub fn new() -> Binder<'a> {
        let ticks_per_second = AudioFormat::sample_rate() as f32;
        let ticks_per_minute = ticks_per_second * 60.;
        Self {
            ticks_per_second,
            ticks_per_minute,
            parser_beats: HashMap::new(),
            envelops_indexes: HashMap::new(),
            no_envelop: usize::MAX,
            parser_chords: HashMap::new(),
            parser_attacks: HashMap::new(),
            parser_chordlines: Vec::new(),
            default_chordline: vec![vec![DEFAULT_CHORD]],
            chordlines: HashMap::new(),
            parser_durationlines: Vec::new(),
            durationlines: HashMap::new(),
            parser_velocitylines: HashMap::new(),
            default_velocityline: PVelocityLine {
                id: "",
                velocities: vec![PVelocity {
                    level: 1.,
                    fadein: false,
                    fadeout: false,
                    transition: PShape::None,
                }],
            },
            parser_hitlines: Vec::new(),
            hitlines: HashMap::new(),
            parser_pitchlines: Vec::new(),
            pitchlines: HashMap::new(),
            parser_sequences: HashMap::new(),
        }
    }

    pub fn deserialize(&mut self) -> Result<(), failure::Error> {
        // Deserialize pitchlines
        let scale = scale::create("tempered")?;

        for ppitchline in &self.parser_pitchlines {
            let mut pitchs = Vec::new();
            for pitch in &ppitchline.pitchs {
                let freq = match scale.fetch_frequency(pitch.id) {
                    Some(f) => *f,
                    None => match f32::from_str(pitch.id) {
                        Ok(f) => f,
                        Err(_) => {
                            return Err(failure::err_msg(format!(
                                "Tseq pitch {} not found!",
                                pitch.id
                            )))
                        }
                    },
                };
                pitchs.push((freq, pitch.transition));
            }
            self.pitchlines.insert(ppitchline.id, pitchs);
        }

        // Deserialize chordlines
        let no_accents = Vec::new();

        for pchordline in &self.parser_chordlines {
            let mut chordline = Vec::new();

            for pchord_and_attack in &pchordline.chords {
                match self.parser_chords.get(pchord_and_attack.chord_id) {
                    Some(pchord) => {
                        let paccents = pchord_and_attack
                            .attack_id
                            .and_then(|id| self.parser_attacks.get(id))
                            .map_or(&no_accents, |a| &a.accents);
                        let mut chord = Vec::new();

                        for (harmonic_idx, pharmonic) in pchord.harmonics.iter().enumerate() {
                            let mut delay = pharmonic
                                .delay
                                .as_ref()
                                .map_or(Time::Ticks(0), |d| to_time(&d, self.ticks_per_second));
                            let mut velocity = PVelocity {
                                level: audio_event::DEFAULT_VELOCITY,
                                fadein: false,
                                fadeout: false,
                                transition: PShape::None,
                            };

                            let ovelocity = if harmonic_idx < paccents.len() {
                                delay =
                                    to_time(&paccents[harmonic_idx].delay, self.ticks_per_second);
                                &paccents[harmonic_idx].velocity
                            } else {
                                &pharmonic.velocity
                            };
                            if let Some(pvelocity) = ovelocity {
                                velocity = *pvelocity;
                            }

                            let harmonic = Harmonic {
                                freq_ratio: to_freq_ratio(&pharmonic.pitch_gap, &scale),
                                delay,
                                velocity,
                            };
                            chord.push(harmonic);
                        }
                        chordline.push(chord);
                    }
                    None => {
                        return Err(failure::err_msg(format!(
                            "Tseq chord {} not found!",
                            pchord_and_attack.chord_id
                        )))
                    }
                }
            }
            self.chordlines.insert(pchordline.id, chordline);
        }

        // Deserialize hitlines
        for phitline in &self.parser_hitlines {
            self.hitlines
                .insert(phitline.id, to_hitline(phitline, self.ticks_per_second));
        }

        // Deserialize durationlines
        for pdurationline in &self.parser_durationlines {
            self.durationlines.insert(
                pdurationline.id,
                to_durationline(pdurationline, self.ticks_per_second),
            );
        }

        Ok(())
    }

    pub fn fetch_beat(&'a self, id: &str) -> Result<f32, failure::Error> {
        match self.parser_beats.get(id) {
            Some(e) => Ok(e.bpm as f32),
            None => match f32::from_str(id) {
                Ok(f) => Ok(f),
                Err(_) => Err(failure::err_msg(format!("Tseq beat {} not found!", id))),
            },
        }
    }
    pub fn fetch_envelop_index(&'a self, id: &str) -> Result<usize, failure::Error> {
        match self.envelops_indexes.get(id) {
            Some(ei) => Ok(*ei),
            None => Err(failure::err_msg(format!("Tseq envelop {} not found!", id))),
        }
    }
    pub fn fetch_durationline(&'a self, id: &str) -> Result<&'a DurationLine, failure::Error> {
        match self.durationlines.get(id) {
            Some(e) => Ok(e),
            None => Err(failure::err_msg(format!(
                "Tseq durations {} not found!",
                id
            ))),
        }
    }
    pub fn fetch_velocityline(
        &'a self,
        oid: &'a Option<&str>,
    ) -> Result<&'a PVelocityLine, failure::Error> {
        match oid {
            Some(id) => match self.parser_velocitylines.get(id) {
                Some(e) => Ok(e),
                None => Err(failure::err_msg(format!(
                    "Tseq velocityline {} not found!",
                    id
                ))),
            },
            None => Ok(&self.default_velocityline),
        }
    }

    pub fn fetch_chord(&'a self, id: &str) -> Result<&'a PChord, failure::Error> {
        match self.parser_chords.get(id) {
            Some(e) => Ok(e),
            None => Err(failure::err_msg(format!("Tseq chord {} not found!", id))),
        }
    }

    pub fn fetch_chordline(
        &'a self,
        oid: &'a Option<&str>,
    ) -> Result<&'a Vec<Vec<Harmonic>>, failure::Error> {
        match oid {
            Some(id) => match self.chordlines.get(id) {
                Some(chordline) => Ok(chordline),
                None => Err(failure::err_msg(format!("Tseq chords {} not found!", id))),
            },
            None => Ok(&self.default_chordline),
        }
    }

    pub fn fetch_hitline(&'a self, id: &str) -> Result<&'a HitLine, failure::Error> {
        match self.hitlines.get(id) {
            Some(e) => Ok(e),
            None => Err(failure::err_msg(format!("Tseq hits {} not found!", id))),
        }
    }
    pub fn fetch_pitchline(&'a self, id: &str) -> Result<&'a Vec<(f32, PShape)>, failure::Error> {
        match self.pitchlines.get(id) {
            Some(e) => Ok(e),
            None => Err(failure::err_msg(format!("Tseq pitchs {} not found!", id))),
        }
    }
    pub fn fetch_sequence(&'a self, id: &str) -> Result<&'a PSequence, failure::Error> {
        match self.parser_sequences.get(id) {
            Some(e) => Ok(e),
            None => Err(failure::err_msg(format!("Tseq seq {} not found!", id))),
        }
    }
}
