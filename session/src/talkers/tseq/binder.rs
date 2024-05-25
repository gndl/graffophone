use std::collections::HashMap;
use std::f32;
use std::str::FromStr;

use talker::audio_format::AudioFormat;
use talkers::tseq::parser::{
    PAttack, PBeat, PChord, PChordLine, PDurationLine, PHit, PHitLine, PPitchGap, PPitchLine,
    PSequence, PScale, PShape, PTime, PVelocity, PVelocityLine,
};
use talkers::tseq::scale::Scale;

use super::envelope;

pub const DEFAULT_FREQUENCY: f32 = 0.;
pub const DEFAULT_VELOCITY: f32 = 1.;
pub const DEFAULT_BPM: f32 = 90.;
pub const DEFAULT_SCALE: &str = "tempered";

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

pub struct Velocity {
    pub envelope_index: usize,
    pub level: f32,
    pub fadein: bool,
    pub fadeout: bool,
    pub transition: PShape,
}
impl Velocity {
    pub fn new() -> Velocity {
        Self {
            envelope_index: envelope::UNDEFINED,
            level: DEFAULT_VELOCITY,
            fadein: false,
            fadeout: false,
            transition: PShape::None,
        }
    }
    pub fn from(pvelo: &PVelocity, env_idxs: &HashMap<&str, usize>) -> Result<Velocity, failure::Error> {
        let envelop_index = match pvelo.envelope_id {
            Some(id) => {
                match env_idxs.get(id) {
                    Some(idx) => *idx,
                    None => return Err(failure::err_msg(format!("Tseq envelope {} not found!", id))),
                }
            }
            None => envelope::UNDEFINED,
        };

        Ok(Self {
            envelope_index: envelop_index,
            level: pvelo.level,
            fadein: pvelo.fadein,
            fadeout: pvelo.fadeout || envelop_index != envelope::UNDEFINED,
            transition: pvelo.transition,
        })
    }
}

pub struct Harmonic {
    pub pitch_gap: PPitchGap,
    pub delay: Time,
    pub velocity: Velocity,
}
impl Harmonic {
    pub fn new() -> Harmonic {
        Self {
            pitch_gap: PPitchGap::FreqRatio(1.),
            delay: Time::Ticks(0),
            velocity: Velocity::new(),
        }
    }
}

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
    pub default_bpm: f32,
    pub default_scale: &'a str,
    pub parser_beats: HashMap<&'a str, &'a PBeat<'a>>,
    pub parser_scales: HashMap<&'a str, &'a PScale<'a>>,
    pub envelops_indexes: HashMap<&'a str, usize>,
    pub parser_chords: HashMap<&'a str, &'a PChord<'a>>,
    pub parser_attacks: HashMap<&'a str, &'a PAttack<'a>>,
    pub parser_chordlines: Vec<&'a PChordLine<'a>>,
    default_chordline: Vec<Vec<Harmonic>>,
    chordlines: HashMap<&'a str, Vec<Vec<Harmonic>>>,
    pub parser_durationlines: Vec<&'a PDurationLine<'a>>,
    pub durationlines: HashMap<&'a str, DurationLine>,
    pub parser_velocitylines: Vec<&'a PVelocityLine<'a>>,
    default_velocityline: Vec<Velocity>,
    pub velocitylines: HashMap<&'a str, Vec<Velocity>>,
    pub parser_hitlines: Vec<&'a PHitLine<'a>>,
    pub hitlines: HashMap<&'a str, HitLine>,
    pub parser_pitchlines: Vec<&'a PPitchLine<'a>>,
    pitchlines: HashMap<&'a str, (&'a Scale, Vec<(f32, PShape)>)>,
    pub parser_sequences: HashMap<&'a str, &'a PSequence<'a>>,
}

impl<'a> Binder<'a> {
    pub fn new() -> Binder<'a> {
        let ticks_per_second = AudioFormat::sample_rate() as f32;
        let ticks_per_minute = ticks_per_second * 60.;
        Self {
            ticks_per_second,
            ticks_per_minute,
            default_bpm: DEFAULT_BPM,
            default_scale: DEFAULT_SCALE,
            parser_beats: HashMap::new(),
            parser_scales: HashMap::new(),
            envelops_indexes: HashMap::new(),
            parser_chords: HashMap::new(),
            parser_attacks: HashMap::new(),
            parser_chordlines: Vec::new(),
            default_chordline: vec![vec![Harmonic::new()]],
            chordlines: HashMap::new(),
            parser_durationlines: Vec::new(),
            durationlines: HashMap::new(),
            parser_velocitylines: Vec::new(),
            default_velocityline: vec![Velocity::new()],
            velocitylines: HashMap::new(),
            parser_hitlines: Vec::new(),
            hitlines: HashMap::new(),
            parser_pitchlines: Vec::new(),
            pitchlines: HashMap::new(),
            parser_sequences: HashMap::new(),
        }
    }

    pub fn deserialize(&mut self, scales: &'a HashMap<&str, Scale>) -> Result<(), failure::Error> {

        let mut scales_freqs =  HashMap::new();

        self.default_bpm = self.parser_beats
                                   .iter()
                                   .last()
                                   .map_or(DEFAULT_BPM, |(_, b)| b.bpm);

        self.default_scale = self.parser_scales
                                     .iter()
                                     .last()
                                     .map_or(DEFAULT_SCALE, |(_, s)| s.name);

        // Deserialize pitchlines
        let default_scale = match scales.get(self.default_scale) {
                        Some(scale) => scale,
                        None => return Err(failure::err_msg(format!("Tseq scale {} not found!", self.default_scale))),
                    };

        for ppitchline in &self.parser_pitchlines {
            let mut pitchs = Vec::new();

            let scale = match ppitchline.scale {
                Some(scale_name) => {
                    match scales.get(scale_name) {
                        Some(scale) => scale,
                        None => return Err(failure::err_msg(format!("Tseq scale {} not found!", scale_name))),
                    }
                },
                None => default_scale,
            };

            let pitch_freq_map = match scales_freqs.get_mut(scale.name) {
                Some(m) => m,
                None => {
                    let pitch_freq_map = HashMap::new();
                    scales_freqs.insert(scale.name, pitch_freq_map);
                    scales_freqs.get_mut(scale.name).unwrap()
                }
            };

            for pitch in &ppitchline.pitchs {
                let freq = match pitch_freq_map.get(&pitch.id) {
                    Some(f) => *f,
                    None => {
                        let f = scale.fetch_frequency(pitch.id)?;
                        pitch_freq_map.insert(pitch.id, f);
                        f
                    }
                };
                pitchs.push((freq, pitch.transition));
            }
            self.pitchlines.insert(ppitchline.id, (scale, pitchs));
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

                            let opvelocity = if harmonic_idx < paccents.len() {
                                delay = to_time(&paccents[harmonic_idx].delay, self.ticks_per_second);
                                &paccents[harmonic_idx].velocity
                            } else {
                                &pharmonic.velocity
                            };

                            let velocity = match opvelocity {
                                Some(pvelo) => Velocity::from(pvelo, &self.envelops_indexes)?,
                                None => Velocity::new(),
                            };

                            let harmonic = Harmonic {
                                pitch_gap: pharmonic.pitch_gap,
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

        // Deserialize velocitylines
        for pvelocityline in &self.parser_velocitylines {
            let mut velocities = Vec::with_capacity(pvelocityline.velocities.len());

            for pvelocity in &pvelocityline.velocities {
                let velocity = Velocity::from(pvelocity, &self.envelops_indexes)?;
                velocities.push(velocity);
            }
            self.velocitylines.insert(pvelocityline.id, velocities);
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
            None => Err(failure::err_msg(format!("Tseq envelope {} not found!", id))),
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
    ) -> Result<&Vec<Velocity>, failure::Error> {
        match oid {
            Some(id) => match self.velocitylines.get(id) {
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
    pub fn fetch_pitchline(&'a self, id: &str) -> Result<&(&Scale, Vec<(f32, PShape)>), failure::Error> {
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
