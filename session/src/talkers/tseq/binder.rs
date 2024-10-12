use std::collections::{HashMap, HashSet};
use std::f32;
use std::str::FromStr;

use scale::scale::{self, Scale};
use talker::audio_format::AudioFormat;
use talkers::tseq::parser::{
    PAttack, PBeat, PChord, PChordLine, PDurationLine, PHit, PHitLine,
    PPitchGap, PPitchLineFragment, PPitchLine, PPitchLineTransformation,
    PSeqFragment, PSequence, PScale, PShape, PTime, PVelocity, PVelocityLine,
};
use talkers::tseq::pitch::{self, Pitch};

use super::envelope;

pub const DEFAULT_FREQUENCY: f32 = 0.;
pub const DEFAULT_VELOCITY: f32 = 1.;
pub const DEFAULT_BPM: f32 = 90.;

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
                    None => return Err(failure::err_msg(format!("Envelope {} not found!", id))),
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

fn elements_scheduling<F>(elements_deps: &Vec<HashSet<usize>>, elements_type: &str, element_id: F) -> Result<Vec<usize>, failure::Error>
where F: Fn(usize) -> String,
{
    let elements_count = elements_deps.len();
    let mut elements_to_schedule_indexes = Vec::with_capacity(elements_count);

    for i in 0..elements_count {
        elements_to_schedule_indexes.push(i);
    }

    let mut scheduled_elements_indexes = Vec::with_capacity(elements_count);

    let mut iterations_count = 0;

    while iterations_count < elements_count {
        for i in 0..elements_count {
            let element_idx = elements_to_schedule_indexes[i];

            if element_idx < elements_count {
                let mut deps_found_count = 0;
                
                for dep_idx in &elements_deps[element_idx] {
                    for j in 0..scheduled_elements_indexes.len() {
                        if *dep_idx == scheduled_elements_indexes[j] {
                            deps_found_count += 1;
                            break;
                        }
                    }
                }

                if deps_found_count == elements_deps[element_idx].len() {
                    scheduled_elements_indexes.push(element_idx);
                    elements_to_schedule_indexes[i] = usize::MAX;
                }
            }
        }
        if scheduled_elements_indexes.len() == elements_count {
            break;
        }
        iterations_count += 1;
    }

    // Mutually dependent elements checking
    if scheduled_elements_indexes.len() < elements_count {
        let mut mutually_dependent_elements = String::new();

        for i in elements_to_schedule_indexes {
            if i < elements_count {
                mutually_dependent_elements.push_str(&element_id(i));
                mutually_dependent_elements.push(' ');
            }
        }
        return Err(failure::err_msg(format!("Mutually dependent {}s is forbidden! Concerned {}s : {}.", elements_type, elements_type, mutually_dependent_elements)));
    }
    Ok(scheduled_elements_indexes)
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
    default_durationline: DurationLine,
    pub durationlines: HashMap<&'a str, DurationLine>,
    pub parser_velocitylines: Vec<&'a PVelocityLine<'a>>,
    default_velocityline: Vec<Velocity>,
    pub velocitylines: HashMap<&'a str, Vec<Velocity>>,
    pub parser_hitlines: Vec<&'a PHitLine<'a>>,
    pub hitlines: HashMap<&'a str, HitLine>,
    pub parser_pitchlines: Vec<&'a PPitchLine<'a>>,
    pitchlines: HashMap<&'a str, (&'a Scale, Vec<(f32, PShape)>)>,
    pub parser_sequences: Vec<&'a PSequence<'a>>,
}

impl<'a> Binder<'a> {
    pub fn new() -> Binder<'a> {
        let ticks_per_second = AudioFormat::sample_rate() as f32;
        let ticks_per_minute = ticks_per_second * 60.;
        Self {
            ticks_per_second,
            ticks_per_minute,
            default_bpm: DEFAULT_BPM,
            default_scale: scale::DEFAULT,
            parser_beats: HashMap::new(),
            parser_scales: HashMap::new(),
            envelops_indexes: HashMap::new(),
            parser_chords: HashMap::new(),
            parser_attacks: HashMap::new(),
            parser_chordlines: Vec::new(),
            default_chordline: vec![vec![Harmonic::new()]],
            chordlines: HashMap::new(),
            parser_durationlines: Vec::new(),
            default_durationline: DurationLine{durations: Vec::new()},
            durationlines: HashMap::new(),
            parser_velocitylines: Vec::new(),
            default_velocityline: vec![Velocity::new()],
            velocitylines: HashMap::new(),
            parser_hitlines: Vec::new(),
            hitlines: HashMap::new(),
            parser_pitchlines: Vec::new(),
            pitchlines: HashMap::new(),
            parser_sequences: Vec::new(),
        }
    }

    fn sequence_dependencies(&self, fragments: &Vec<PSeqFragment>, sequence_deps: &mut HashSet<usize>) -> Result<(), failure::Error> {
        let sequences_count = self.parser_sequences.len();

        for fragment in fragments {
            match fragment {
                PSeqFragment::SeqRef(pl_ref) => {
                    let mut dep_idx = 0;

                    while dep_idx < sequences_count {
                        if pl_ref.id == self.parser_sequences[dep_idx].id {
                            sequence_deps.insert(dep_idx);
                            break;
                        }
                        dep_idx += 1;
                    }
                    if dep_idx == sequences_count {
                        return Err(failure::err_msg(format!("Sequence {} unknown!", pl_ref.id)));
                    }
                },
                PSeqFragment::Fragments((frags, _)) => self.sequence_dependencies(frags, sequence_deps)?,
                PSeqFragment::Part(_) => (),
            }
        }
        Ok(())
    }

    pub fn check_sequences(&self) -> Result<(), failure::Error> {
        let sequences_count = self.parser_sequences.len();

        // sequences dependencies indexes
        let mut sequences_deps = vec![HashSet::new(); sequences_count];

        for (seq_idx, seq) in self.parser_sequences.iter().enumerate() {
            self.sequence_dependencies(&seq.fragments, &mut sequences_deps[seq_idx])?;
        }

        // sequences scheduling
        let _ = elements_scheduling(&sequences_deps, "sequence", |i| self.parser_sequences[i].id.to_string())?;
        Ok(())
    }

    fn pitchlines_scheduling(&self) -> Result<Vec<usize>, failure::Error> {
        let pitchlines_count = self.parser_pitchlines.len();

        // pitchlines dependencies indexes
        let mut pitchlines_deps = vec![HashSet::new(); pitchlines_count];

        for (pitchline_idx, pitchline) in self.parser_pitchlines.iter().enumerate() {
            for fragment in &pitchline.fragments {
                match fragment {
                    PPitchLineFragment::PitchLineRef(pl_ref) => {
                        let mut dep_idx = 0;

                        while dep_idx < pitchlines_count {
                            if pl_ref.id == self.parser_pitchlines[dep_idx].id {
                                pitchlines_deps[pitchline_idx].insert(dep_idx);
                                break;
                            }
                            dep_idx += 1;
                        }
                        if dep_idx == pitchlines_count {
                            return Err(failure::err_msg(format!("Pitchline {} unknown!", pl_ref.id)));
                        }
                    },
                    PPitchLineFragment::Pitch(_) => (),
                }
            }
        }

        // pitchlines scheduling
        elements_scheduling(&pitchlines_deps, "pitchline", |i| self.parser_pitchlines[i].id.to_string())
    }

    pub fn deserialize(&mut self, scales: &'a scale::Collection) -> Result<(), failure::Error> {

        self.default_bpm = self.parser_beats
                                   .iter()
                                   .last()
                                   .map_or(DEFAULT_BPM, |(_, b)| b.bpm);

        // Deserialize pitchlines
        let scheduled_pitchlines_indexes = self.pitchlines_scheduling()?;

        // pitchlines pitchs development, transformation and frequencies
        let mut pitchs_map: HashMap<&'a str, Vec<Pitch>> = HashMap::new();

        self.default_scale = self.parser_scales
                                     .iter()
                                     .last()
                                     .map_or(scale::DEFAULT, |(_, s)| s.name);

        let default_scale = scales.fetch(self.default_scale)?;
        let mut scales_freqs =  HashMap::new();

        for i in scheduled_pitchlines_indexes {
            let pitchline = self.parser_pitchlines[i];
            
            let mut pitchs = Vec::new();

            let scale = match pitchline.scale {
                Some(scale_id) => match self.parser_scales.get(scale_id) {
                        Some(pscale) => scales.fetch(&pscale.name)?,
                        None => scales.fetch(scale_id)?,
                    },
                None => default_scale,
            };

            // pitch development and transformation
            for fragment in &pitchline.fragments {
                match fragment {
                    PPitchLineFragment::Pitch(p) => {
                        for _ in 0..p.mul {
                            pitchs.push(Pitch::from_parser_pitch(p));
                        }
                    },
                    PPitchLineFragment::PitchLineRef(pl_ref) => {

                        let ref_pitchs = match pitchs_map.get(pl_ref.id) {
                            Some(ps) => ps,
                            None => return Err(failure::err_msg(format!("Pitchline {} not found!", pl_ref.id))),
                        };

                        pitchs.reserve(ref_pitchs.len() * pl_ref.mul);

                        // dependence pitchs transformation
                        if let Some(transformations) = &pl_ref.transformations {

                            let mut work_pitchs = ref_pitchs.clone();

                            for transfo in transformations {
                                match transfo {
                                    PPitchLineTransformation::NoteShift(n) => pitch::notes_shift(&mut work_pitchs, *n)?,
                                    PPitchLineTransformation::BackwardNoteShift(n) => pitch::backward_notes_shift(&mut work_pitchs, *n)?,
                                    PPitchLineTransformation::PitchTranspo(ip, fp) => pitch::pitchs_transposition(&mut work_pitchs, scale, ip, fp)?,
                                    PPitchLineTransformation::PitchInv => pitch::pitchs_inversion(&mut work_pitchs, scale)?,
                                }
                            }

                            for _ in 0..pl_ref.mul {
                                for p in &work_pitchs {
                                    pitchs.push(Pitch::new(p));
                                }
                            }
                        }
                        else {
                            for _ in 0..pl_ref.mul {
                                for p in ref_pitchs {
                                    pitchs.push(Pitch::new(p));
                                }
                            }
                        }
                    },
                }
            }

            // pitchs to frequencies
            let mut frequencies = Vec::new();

            let pitch_freq_map = match scales_freqs.get_mut(scale.name) {
                Some(m) => m,
                None => {
                    let pitch_freq_map = HashMap::new();
                    scales_freqs.insert(scale.name, pitch_freq_map);
                    scales_freqs.get_mut(scale.name).unwrap()
                }
            };

            for pitch in &pitchs {
                let freq = match pitch_freq_map.get(&pitch.id) {
                    Some(f) => *f,
                    None => {
                        let f = scale.pitch_name_to_frequency(&pitch.id)?;
                        pitch_freq_map.insert(pitch.id.clone(), f);
                        f
                    }
                };
                frequencies.push((freq, pitch.transition));
            }
            self.pitchlines.insert(pitchline.id, (scale, frequencies));
                
            pitchs_map.insert(pitchline.id, pitchs);
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
                            "Chord {} not found!",
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
                Err(_) => Err(failure::err_msg(format!("Beat {} not found!", id))),
            },
        }
    }
    pub fn fetch_envelop_index(&'a self, id: &str) -> Result<usize, failure::Error> {
        match self.envelops_indexes.get(id) {
            Some(ei) => Ok(*ei),
            None => Err(failure::err_msg(format!("Envelope {} not found!", id))),
        }
    }
    pub fn fetch_durationline(&'a self, oid: &'a Option<&str>,) -> Result<&'a DurationLine, failure::Error> {
        match oid {
            Some(id) => match self.durationlines.get(id) {
                Some(e) => Ok(e),
                None => Err(failure::err_msg(format!("Durations {} not found!", id))),
            },
            None => Ok(&self.default_durationline),
        }
    }
    pub fn fetch_velocityline(
        &'a self,
        oid: &'a Option<&str>,
    ) -> Result<&Vec<Velocity>, failure::Error> {
        match oid {
            Some(id) => match self.velocitylines.get(id) {
                Some(e) => Ok(e),
                None => Err(failure::err_msg(format!("Velocityline {} not found!", id))),
            },
            None => Ok(&self.default_velocityline),
        }
    }

    pub fn fetch_chord(&'a self, id: &str) -> Result<&'a PChord, failure::Error> {
        match self.parser_chords.get(id) {
            Some(e) => Ok(e),
            None => Err(failure::err_msg(format!("Chord {} not found!", id))),
        }
    }

    pub fn fetch_chordline(
        &'a self,
        oid: &'a Option<&str>,
    ) -> Result<&'a Vec<Vec<Harmonic>>, failure::Error> {
        match oid {
            Some(id) => match self.chordlines.get(id) {
                Some(chordline) => Ok(chordline),
                None => Err(failure::err_msg(format!("Chords {} not found!", id))),
            },
            None => Ok(&self.default_chordline),
        }
    }

    pub fn fetch_hitline(&'a self, id: &str) -> Result<&'a HitLine, failure::Error> {
        match self.hitlines.get(id) {
            Some(e) => Ok(e),
            None => Err(failure::err_msg(format!("Hits {} not found!", id))),
        }
    }
    pub fn fetch_pitchline(&'a self, id: &str) -> Result<&(&Scale, Vec<(f32, PShape)>), failure::Error> {
        match self.pitchlines.get(id) {
            Some(e) => Ok(e),
            None => Err(failure::err_msg(format!("Pitchs {} not found!", id))),
        }
    }
    pub fn fetch_sequence(&'a self, id: &str) -> Result<&'a PSequence, failure::Error> {
        for seq in &self.parser_sequences {
            if seq.id == id {
                return Ok(seq);
            }
        }
        Err(failure::err_msg(format!("Sequence {} not found!", id)))
    }
}
