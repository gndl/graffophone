use std::collections::{HashMap, HashSet};
use std::f32;
use std::str::FromStr;

use scale::scale::{self, Scale};
use talker::audio_format::AudioFormat;
use talkers::tseq::parser::{
    PAttack, PBeat, PChord, PChordLineFragment, PChordLine, PDurationLine, PHit, PHitLine,
    PPitchGap, PPitchLineFragment, PPitchLine, PPitchLineTransformation,
    PSeqFragment, PSequence, PScale, PShape, PTime, PVelocity, PVelocityLine,
};
use talkers::tseq::pitch::{self, Pitch};

use super::envelope;
use super::parser::PVelocityLineFragment;

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

#[derive(Clone, Copy)]
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

#[derive(Clone, Copy)]
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

    fn chordline_dependencies(&self, fragments: &Vec<PChordLineFragment>, line_deps: &mut HashSet<usize>) -> Result<(), failure::Error> {
        let lines_count = self.parser_chordlines.len();

        for fragment in fragments {
            match fragment {
                PChordLineFragment::Part(_) => (),
                PChordLineFragment::Ref(line_ref) => {
                    let mut dep_idx = 0;

                    while dep_idx < lines_count {
                        if line_ref.id == self.parser_chordlines[dep_idx].id {
                            line_deps.insert(dep_idx);
                            break;
                        }
                        dep_idx += 1;
                    }
                    if dep_idx == lines_count {
                        return Err(failure::err_msg(format!("Chordline {} unknown!", line_ref.id)));
                    }
                },
                PChordLineFragment::Fragments((frags, _)) => {
                    self.chordline_dependencies(frags, line_deps)?;
                },
            }
        }
        Ok(())
    }

    fn chordlines_scheduling(&self) -> Result<Vec<usize>, failure::Error> {
        let lines_count = self.parser_chordlines.len();

        // chordlines dependencies indexes
        let mut lines_deps = vec![HashSet::new(); lines_count];

        for (line_idx, line) in self.parser_chordlines.iter().enumerate() {
            self.chordline_dependencies(&line.fragments, &mut lines_deps[line_idx])?;
        }

        // chordlines scheduling
        elements_scheduling(&lines_deps, "chordline", |i| self.parser_chordlines[i].id.to_string())
    }

    fn chordline_deserialize(&self, fragments: &Vec<PChordLineFragment>) -> Result<Vec<Vec<Harmonic>>, failure::Error> {
        let mut line = Vec::new();
        let no_accents = Vec::with_capacity(0);

        for fragment in fragments {
            match fragment {
                PChordLineFragment::Part((part, mul)) => {
                    match self.parser_chords.get(part.chord_id) {
                        Some(pchord) => {
                            let paccents = part
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

                            line.reserve(*mul);

                            for _ in 0..(mul - 1) {
                                line.push(chord.clone());
                            }
                            line.push(chord);
                        }
                        None => {
                            return Err(failure::err_msg(format!(
                                "Chord {} not found!",
                                part.chord_id
                            )))
                        }
                    }
                },
                PChordLineFragment::Ref(line_ref) => {
                    let ref_line = match self.chordlines.get(line_ref.id) {
                        Some(ref_line) => ref_line,
                        None => return Err(failure::err_msg(format!("Chordline {} not found!", line_ref.id))),
                    };

                    line.reserve(ref_line.len() * line_ref.mul);
                    
                    for _ in 0..line_ref.mul {
                        for elem in ref_line {
                            line.push(elem.clone());
                        }
                    }
                },
                PChordLineFragment::Fragments((frags, mul)) => {
                    let frags_line = self.chordline_deserialize(frags)?;

                    line.reserve(frags_line.len() * mul);

                    for _ in 0..(*mul - 1) {
                        for elem in &frags_line {
                            line.push(elem.clone());
                        }
                    }
                    line.extend(frags_line);
                }
            }
        }
        Ok(line)
    }

    fn pitchline_dependencies(&self, fragments: &Vec<PPitchLineFragment>, line_deps: &mut HashSet<usize>) -> Result<(), failure::Error> {
        let lines_count = self.parser_pitchlines.len();

        for fragment in fragments {
            match fragment {
                PPitchLineFragment::Part(_) => (),
                PPitchLineFragment::Ref((line_ref, _)) => {
                    let mut dep_idx = 0;

                    while dep_idx < lines_count {
                        if line_ref.id == self.parser_pitchlines[dep_idx].id {
                            line_deps.insert(dep_idx);
                            break;
                        }
                        dep_idx += 1;
                    }
                    if dep_idx == lines_count {
                        return Err(failure::err_msg(format!("Pitchline {} unknown!", line_ref.id)));
                    }
                },
                PPitchLineFragment::Fragments((frags, _)) => {
                    self.pitchline_dependencies(frags, line_deps)?;
                },
            }
        }
        Ok(())
    }

    fn pitchlines_scheduling(&self) -> Result<Vec<usize>, failure::Error> {
        let pitchlines_count = self.parser_pitchlines.len();

        // pitchlines dependencies indexes
        let mut pitchlines_deps = vec![HashSet::new(); pitchlines_count];

        for (pitchline_idx, pitchline) in self.parser_pitchlines.iter().enumerate() {
            self.pitchline_dependencies(&pitchline.fragments, &mut pitchlines_deps[pitchline_idx])?;
        }

        // pitchlines scheduling
        elements_scheduling(&pitchlines_deps, "pitchline", |i| self.parser_pitchlines[i].id.to_string())
    }

    fn pitchline_development(&self, fragments: &Vec<PPitchLineFragment>, pitchs_map: &HashMap<&'a str, Vec<Pitch>>, scale: &Scale) -> Result<Vec<Pitch>, failure::Error> {
        let mut line = Vec::new();

        for fragment in fragments {
            match fragment {
                PPitchLineFragment::Part((part, mul)) => {
                    let elem = Pitch::from_parser_pitch(part);

                    line.reserve(*mul);

                    for _ in 0..(mul - 1) {
                        line.push(elem.clone());
                    }
                    line.push(elem);
                },
                PPitchLineFragment::Ref((line_ref, transformations)) => {

                    let ref_line = match pitchs_map.get(line_ref.id) {
                        Some(ref_line) => ref_line,
                        None => return Err(failure::err_msg(format!("Pitchline {} not found!", line_ref.id))),
                    };

                    line.reserve(ref_line.len() * line_ref.mul);

                    // dependence pitchs transformation
                    if let Some(transformations) = &transformations {

                        let mut work_pitchs = ref_line.clone();

                        for transfo in transformations {
                            match transfo {
                                PPitchLineTransformation::NoteShift(n) => pitch::notes_shift(&mut work_pitchs, *n)?,
                                PPitchLineTransformation::BackwardNoteShift(n) => pitch::backward_notes_shift(&mut work_pitchs, *n)?,
                                PPitchLineTransformation::PitchTranspo(ip, fp) => pitch::pitchs_transposition(&mut work_pitchs, scale, ip, fp)?,
                                PPitchLineTransformation::PitchInv => pitch::pitchs_inversion(&mut work_pitchs, scale)?,
                            }
                        }

                        for _ in 0..line_ref.mul {
                            for elem in &work_pitchs {
                                line.push(elem.clone());
                            }
                        }
                    }
                    else {
                        for _ in 0..line_ref.mul {
                            for elem in ref_line {
                                line.push(elem.clone());
                            }
                        }
                    }
                },
                PPitchLineFragment::Fragments((frags, mul)) => {
                    let frags_line = self.pitchline_development(frags, pitchs_map, scale)?;

                    line.reserve(frags_line.len() * mul);

                    for _ in 0..(*mul - 1) {
                        for elem in &frags_line {
                            line.push(elem.clone());
                        }
                    }
                    line.extend(frags_line);
                },
            }
        }
        Ok(line)
    }

    fn velocityline_dependencies(&self, fragments: &Vec<PVelocityLineFragment>, line_deps: &mut HashSet<usize>) -> Result<(), failure::Error> {
        let lines_count = self.parser_velocitylines.len();

        for fragment in fragments {
            match fragment {
                PVelocityLineFragment::Part(_) => (),
                PVelocityLineFragment::Ref(line_ref) => {
                    let mut dep_idx = 0;

                    while dep_idx < lines_count {
                        if line_ref.id == self.parser_velocitylines[dep_idx].id {
                            line_deps.insert(dep_idx);
                            break;
                        }
                        dep_idx += 1;
                    }
                    if dep_idx == lines_count {
                        return Err(failure::err_msg(format!("Velocityline {} unknown!", line_ref.id)));
                    }
                },
                PVelocityLineFragment::Fragments((frags, _)) => {
                    self.velocityline_dependencies(frags, line_deps)?;
                },
            }
        }
        Ok(())
    }

    fn velocitylines_scheduling(&self) -> Result<Vec<usize>, failure::Error> {
        let lines_count = self.parser_velocitylines.len();

        // velocitylines dependencies indexes
        let mut lines_deps = vec![HashSet::new(); lines_count];

        for (line_idx, line) in self.parser_velocitylines.iter().enumerate() {
            self.velocityline_dependencies(&line.fragments, &mut lines_deps[line_idx])?;
        }

        // velocitylines scheduling
        elements_scheduling(&lines_deps, "velocityline", |i| self.parser_velocitylines[i].id.to_string())
    }

    fn velocityline_deserialize(&self, fragments: &Vec<PVelocityLineFragment>) -> Result<Vec<Velocity>, failure::Error> {
        let mut line = Vec::new();

        for fragment in fragments {
            match fragment {
                PVelocityLineFragment::Part((part, mul)) => {
                    let elem = Velocity::from(part, &self.envelops_indexes)?;
                    
                    line.reserve(*mul);

                    for _ in 0..(mul - 1) {
                        line.push(elem.clone());
                    }
                    line.push(elem);
                },
                PVelocityLineFragment::Ref(line_ref) => {
                    let ref_line = match self.velocitylines.get(line_ref.id) {
                        Some(ref_line) => ref_line,
                        None => return Err(failure::err_msg(format!("Velocityline {} not found!", line_ref.id))),
                    };

                    line.reserve(ref_line.len() * line_ref.mul);
                    
                    for _ in 0..line_ref.mul {
                        for elem in ref_line {
                            line.push(elem.clone());
                        }
                    }
                },
                PVelocityLineFragment::Fragments((frags, mul)) => {
                    let frags_line = self.velocityline_deserialize(frags)?;

                    line.reserve(frags_line.len() * mul);

                    for _ in 0..(*mul - 1) {
                        for elem in &frags_line {
                            line.push(elem.clone());
                        }
                    }
                    line.extend(frags_line);
                }
            }
        }
        Ok(line)
    }

    fn sequence_dependencies(&self, fragments: &Vec<PSeqFragment>, sequence_deps: &mut HashSet<usize>) -> Result<(), failure::Error> {
        let sequences_count = self.parser_sequences.len();

        for fragment in fragments {
            match fragment {
                PSeqFragment::Part(_) => (),
                PSeqFragment::Ref(line_ref) => {
                    let mut dep_idx = 0;

                    while dep_idx < sequences_count {
                        if line_ref.id == self.parser_sequences[dep_idx].id {
                            sequence_deps.insert(dep_idx);
                            break;
                        }
                        dep_idx += 1;
                    }
                    if dep_idx == sequences_count {
                        return Err(failure::err_msg(format!("Sequence {} unknown!", line_ref.id)));
                    }
                },
                PSeqFragment::Fragments((frags, _)) => self.sequence_dependencies(frags, sequence_deps)?,
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

    pub fn deserialize(&mut self, scales: &'a scale::Collection) -> Result<(), failure::Error> {

        self.default_bpm = self.parser_beats
                                   .iter()
                                   .last()
                                   .map_or(DEFAULT_BPM, |(_, b)| b.bpm);


        // Deserialize chordlines
        let scheduled_chordlines_indexes = self.chordlines_scheduling()?;

        // chordlines harmonics development and deserialization
        for i in scheduled_chordlines_indexes {
            let pchordline = self.parser_chordlines[i];

            let chordline = self.chordline_deserialize(&pchordline.fragments)?;

            self.chordlines.insert(pchordline.id, chordline);
        }

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
            
            let scale = match pitchline.scale {
                Some(scale_id) => match self.parser_scales.get(scale_id) {
                        Some(pscale) => scales.fetch(&pscale.name)?,
                        None => scales.fetch(scale_id)?,
                    },
                None => default_scale,
            };

            // pitch development and transformation
            let pitchs = self.pitchline_development(&pitchline.fragments, &pitchs_map, &scale)?;

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
        let scheduled_velocitylines_indexes = self.velocitylines_scheduling()?;

        // velocitylines deserialization
        for i in scheduled_velocitylines_indexes {
            let pvelocityline = self.parser_velocitylines[i];

            let velocityline = self.velocityline_deserialize(&pvelocityline.fragments)?;

            self.velocitylines.insert(pvelocityline.id, velocityline);
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
