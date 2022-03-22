use std::cell::Cell;
use std::collections::HashSet;
use std::f32;

use atom_talker::AtomTalker;
use audio_talker::AudioTalker;
use control_talker::ControlTalker;
use cv_talker::CvTalker;
use horn::{AtomBuf, AudioBuf, ControlBuf, ControlVal, CvBuf, Horn, PortType};
use identifier::Id;
use identifier::Identifiable;
use identifier::Index;
use rtalker;
use talker::{RTalker, TalkerCab};

const DEF_EAR_TAG: &'static str = "In";
const DEF_HUM_TAG: &'static str = "";

pub fn def_audio_talker(value: f32) -> RTalker {
    rtalker!(AudioTalker::new(value, Some(true)))
}
pub fn def_control_talker(value: f32) -> RTalker {
    rtalker!(ControlTalker::new(value, Some(true)))
}
pub fn def_cv_talker(value: f32) -> RTalker {
    rtalker!(CvTalker::new(value, Some(true)))
}
pub fn def_atom_talker() -> RTalker {
    rtalker!(AtomTalker::new(Some(true)))
}

pub fn def_talker(port_type: PortType, value: f32) -> RTalker {
    match port_type {
        PortType::Audio => def_audio_talker(value),
        PortType::Control => def_control_talker(value),
        PortType::Cv => def_cv_talker(value),
        PortType::Atom => def_atom_talker(),
    }
}

pub struct Talk {
    talker: RTalker,
    port: Index,
}

impl Talk {
    pub fn new(talker: &RTalker, port: Index) -> Talk {
        Self {
            talker: talker.clone(),
            port: port,
        }
    }

    pub fn clone(&self) -> Talk {
        Self {
            talker: self.talker.clone(),
            port: self.port,
        }
    }

    pub fn talker<'a>(&'a self) -> &'a RTalker {
        &self.talker
    }
    pub fn port(&self) -> Index {
        self.port
    }
    pub fn value(&self) -> Option<f32> {
        self.talker.voice_value(self.port)
    }
    pub fn visit_horn<F>(&self, mut f: F)
    where
        F: FnMut(&Horn),
    {
        {
            match self.talker.voices().get(self.port) {
                Some(voice) => f(voice.horn()),
                None => (),
            }
        }
    }
    pub fn audio_buffer(&self) -> AudioBuf {
        self.talker.voice(self.port).audio_buffer()
    }
    pub fn control_buffer(&self) -> ControlBuf {
        self.talker.voice(self.port).control_buffer()
    }
    pub fn control_value(&self) -> ControlVal {
        self.talker.voice(self.port).control_value()
    }
    pub fn cv_buffer(&self) -> CvBuf {
        self.talker.voice(self.port).cv_buffer()
    }
    pub fn atom_buffer(&self) -> AtomBuf {
        self.talker.voice(self.port).atom_buffer()
    }

    pub fn listen(&self, tick: i64, len: usize) -> usize {
        let port = self.port;
        {
            let voice = self.talker.voice(port);

            if tick == voice.tick() {
                return usize::min(len, voice.len());
            }
        }

        self.talker.talk(port, tick, len)
    }
}
/*
pub fn def_audio_talk(value: f32) -> Talk {
    Talk::new(&def_audio_talker(value), 0)
}
pub fn def_control_talk(value: f32) -> Talk {
    Talk::new(&def_control_talker(value), 0)
}
pub fn def_cv_talk(value: f32) -> Talk {
    Talk::new(&def_cv_talker(value), 0)
}
*/
pub fn def_talk(port_type: PortType, value: f32) -> Talk {
    match port_type {
        PortType::Audio => Talk::new(&def_audio_talker(value), 0),
        PortType::Control => Talk::new(&def_control_talker(value), 0),
        PortType::Cv => Talk::new(&def_cv_talker(value), 0),
        PortType::Atom => Talk::new(&def_atom_talker(), 0),
    }
}

pub struct Hum {
    tag: String,
    port_type: PortType,
    min_value: f32,
    max_value: f32,
    def_value: f32,
    talks: Vec<Talk>,
    horn: Option<Horn>,
}

pub enum Init<'a> {
    Empty,
    DefValue,
    Value(f32),
    Voice(&'a RTalker, Index),
}

impl Hum {
    pub fn new(
        tag: Option<&str>,
        port_type: PortType,
        min_value: f32,
        max_value: f32,
        def_value: f32,
        talks: Vec<Talk>,
    ) -> Hum {
        let horn = if talks.len() > 1 {
            Some(port_type.to_horn())
        } else {
            None
        };

        Self {
            tag: tag.unwrap_or(DEF_HUM_TAG).to_string(),
            port_type,
            min_value,
            max_value,
            def_value,
            talks,
            horn,
        }
    }

    fn initialized(
        tag: Option<&str>,
        port_type: PortType,
        min_value: f32,
        max_value: f32,
        def_value: f32,
        init: &Init,
    ) -> Result<Hum, failure::Error> {
        let talks = match init {
            Init::Empty => Vec::new(),
            Init::DefValue => vec![def_talk(port_type, def_value)],
            Init::Value(v) => vec![def_talk(port_type, *v)],
            Init::Voice(tkr, port) => vec![Talk::new(tkr, *port)],
        };
        Ok(Hum::new(
            tag, port_type, min_value, max_value, def_value, talks,
        ))
    }

    pub fn talks_with(&self, talk_idx: Index, otalk: Option<Talk>) -> Vec<Talk> {
        let mut talks: Vec<Talk> = Vec::with_capacity(self.talks.len());

        for i in 0..Index::min(self.talks.len(), talk_idx) {
            talks.push(self.talks[i].clone());
        }
        otalk.map(|talk| talks.push(talk));

        for i in (talk_idx + 1)..self.talks.len() {
            talks.push(self.talks[i].clone());
        }

        talks
    }

    pub fn with_talks(&self, talks: Vec<Talk>) -> Hum {
        Hum::new(
            Some(&self.tag),
            self.port_type,
            self.min_value,
            self.max_value,
            self.def_value,
            talks,
        )
    }

    pub fn clone(&self) -> Hum {
        self.with_talks(self.talks_with(self.talks.len(), None))
    }

    pub fn visit_horn<F>(&self, mut f: F)
    where
        F: FnMut(&Horn),
    {
        if self.talks.len() == 1 {
            self.talks[0].visit_horn(f)
        } else {
            match &self.horn {
                Some(horn) => f(&horn),
                None => (),
            }
        }
    }

    pub fn port_type(&self) -> PortType {
        self.port_type
    }
    pub fn tag<'a>(&'a self) -> &'a String {
        &self.tag
    }
    pub fn min_value(&self) -> f32 {
        self.min_value
    }
    pub fn max_value(&self) -> f32 {
        self.max_value
    }
    pub fn range(&self) -> (f32, f32, f32) {
        (self.min_value, self.max_value, self.def_value)
    }
    pub fn def_value(&self) -> f32 {
        self.def_value
    }
    pub fn can_have_a_value(&self) -> bool {
        self.port_type != PortType::Atom
    }
    pub fn value(&self) -> Option<f32> {
        if self.can_have_a_value() {
            for talk in &self.talks {
                match talk.value() {
                    Some(v) => return Some(v),
                    _ => (),
                }
            }
        }
        None
    }
    pub fn value_or_default(&self) -> f32 {
        self.value().unwrap_or(self.def_value)
    }
    pub fn audio_buffer(&self) -> AudioBuf {
        if self.talks.len() > 1 {
            self.horn.as_ref().unwrap().audio_buffer()
        } else {
            self.talks[0].audio_buffer()
        }
    }
    pub fn control_buffer(&self) -> ControlBuf {
        if self.talks.len() > 1 {
            self.horn.as_ref().unwrap().control_buffer()
        } else {
            self.talks[0].control_buffer()
        }
    }
    pub fn cv_buffer(&self) -> CvBuf {
        if self.talks.len() > 1 {
            self.horn.as_ref().unwrap().cv_buffer()
        } else {
            self.talks[0].cv_buffer()
        }
    }
    pub fn atom_buffer(&self) -> AtomBuf {
        if self.talks.len() > 1 {
            self.horn.as_ref().unwrap().atom_buffer()
        } else {
            self.talks[0].atom_buffer()
        }
    }
    pub fn talks<'a>(&'a self) -> &'a Vec<Talk> {
        &self.talks
    }
    pub fn add_talk(&self, talk: Talk) -> Result<Hum, failure::Error> {
        Ok(self.with_talks(self.talks_with(self.talks.len(), Some(talk))))
    }
    pub fn add_value(&self, value: f32) -> Result<Hum, failure::Error> {
        self.add_talk(def_talk(self.port_type, value))
    }
    pub fn check_voice(&self, talker: &RTalker, port: Index) -> Result<(), failure::Error> {
        if self.port_type().can_hear(talker.voice_port_type(port)) {
            Ok(())
        } else {
            Err(failure::err_msg(format!(
                "Talker {} voice {} type {} is not compatible with {} type {}!",
                talker.name(),
                port,
                talker.voice_port_type(port).to_string(),
                self.tag(),
                self.port_type.to_string()
            )))
        }
    }
    pub fn add_voice(&self, talker: &RTalker, port: Index) -> Result<Hum, failure::Error> {
        self.check_voice(talker, port)?;
        self.add_talk(Talk::new(talker, port))
    }
    pub fn set_value(&self, value: f32) -> Result<Hum, failure::Error> {
        Ok(self.with_talks(vec![def_talk(self.port_type, value)]))
    }
    pub fn set_voice(&self, talker: &RTalker, port: Index) -> Result<Hum, failure::Error> {
        self.check_voice(talker, port)?;
        Ok(self.with_talks(vec![Talk::new(talker, port)]))
    }
    pub fn sup_talk(&self, talk_idx: Index) -> Result<Hum, failure::Error> {
        Ok(self.with_talks(self.talks_with(talk_idx, None)))
    }

    pub fn set_talk_value(&self, talk_idx: Index, value: f32) -> Result<Hum, failure::Error> {
        Ok(self.with_talks(self.talks_with(
            talk_idx,
            Some(Talk::new(&def_talker(self.port_type, value), 0)),
        )))
    }

    pub fn set_talk_voice(
        &self,
        talk_idx: Index,
        talker: &RTalker,
        port: Index,
    ) -> Result<Hum, failure::Error> {
        self.check_voice(talker, port)?;
        Ok(self.with_talks(self.talks_with(talk_idx, Some(Talk::new(talker, port)))))
    }

    pub fn listen(&self, tick: i64, len: usize) -> usize {
        let mut ln = len;

        for talk in &self.talks {
            ln = talk.listen(tick, ln);
        }

        if self.talks.len() > 1 {
            let horn = &self.horn.as_ref().unwrap();
            match self.port_type {
                PortType::Audio => {
                    let out_buf = horn.audio_buffer();
                    let first_buf = self.talks[0].audio_buffer();

                    for i in 0..ln {
                        out_buf[i] = first_buf[i];
                    }
                    for ti in 1..self.talks.len() {
                        let in_buf = self.talks[ti].audio_buffer();

                        for i in 0..ln {
                            out_buf[i] = out_buf[i] + in_buf[i];
                        }
                    }
                    let c = 1. / (self.talks.len() as f32);

                    for i in 0..ln {
                        out_buf[i] = out_buf[i] * c;
                    }
                }
                PortType::Control => {
                    let mut v = self.talks[0].control_value();

                    for ti in 1..self.talks.len() {
                        v += self.talks[ti].control_value();
                    }
                    horn.set_control_value(v / (self.talks.len() as f32));
                }
                PortType::Cv => {
                    let out_buf = horn.cv_buffer();
                    let first_buf = self.talks[0].cv_buffer();

                    for i in 0..ln {
                        out_buf[i] = first_buf[i];
                    }
                    for ti in 1..self.talks.len() {
                        let in_buf = self.talks[ti].cv_buffer();

                        for i in 0..ln {
                            out_buf[i] = out_buf[i] + in_buf[i];
                        }
                    }
                    let c = 1. / (self.talks.len() as f32);

                    for i in 0..ln {
                        out_buf[i] = out_buf[i] * c;
                    }
                }
                PortType::Atom => {}
            }
        }
        ln
    }
}

pub struct Set {
    hums: Vec<Hum>,
}

impl Set {
    pub fn new(hums: Vec<Hum>) -> Set {
        Self { hums }
    }

    pub fn from_attributs(
        hums_attributs: &Vec<(&str, PortType, f32, f32, f32, Init)>,
    ) -> Result<Set, failure::Error> {
        let mut hums = Vec::with_capacity(hums_attributs.len());

        for (tag, port_type, min_value, max_value, def_value, init) in hums_attributs {
            let hum_tag = if tag.len() > 0 { tag } else { DEF_EAR_TAG };

            hums.push(Hum::initialized(
                Some(hum_tag),
                *port_type,
                *min_value,
                *max_value,
                *def_value,
                init,
            )?);
        }
        Ok(Self { hums })
    }

    pub fn hums<'a>(&'a self) -> &'a Vec<Hum> {
        &self.hums
    }

    pub fn hums_len(&self) -> usize {
        self.hums.len()
    }

    pub fn find_hum_index(&self, hum_tag: &str) -> Result<Index, failure::Error> {
        for i in 0..self.hums.len() {
            if self.hums[i].tag() == hum_tag {
                return Ok(i);
            }
        }
        Err(failure::err_msg(format!("hum {} not found!", hum_tag)))
    }

    pub fn with_hum<F>(&self, hum_idx: Index, mut map: F) -> Result<Set, failure::Error>
    where
        F: FnMut(&Hum) -> Result<Hum, failure::Error>,
    {
        let hums_len = self.hums.len();
        let mut hums: Vec<Hum> = Vec::with_capacity(hums_len);

        for i in 0..Index::min(hum_idx, hums_len) {
            hums.push(self.hums[i].clone());
        }
        hums.push(map(&self.hums[hum_idx])?);

        for i in (hum_idx + 1)..hums_len {
            hums.push(self.hums[i].clone());
        }
        Ok(Set { hums })
    }

    pub fn clone(&self) -> Set {
        let mut hums: Vec<Hum> = Vec::with_capacity(self.hums.len());

        for hum in &self.hums {
            hums.push(hum.clone());
        }
        Set { hums }
    }

    pub fn get_hum_audio_buffer(&self, hum_idx: Index) -> AudioBuf {
        self.hums[hum_idx].audio_buffer()
    }

    pub fn get_hum_control_buffer(&self, hum_idx: Index) -> ControlBuf {
        self.hums[hum_idx].control_buffer()
    }

    pub fn get_hum_cv_buffer(&self, hum_idx: Index) -> CvBuf {
        self.hums[hum_idx].cv_buffer()
    }
}

pub struct Ear {
    tag: String,
    multi_hum: bool,
    stem_set: Option<Set>,
    sets: Cell<Vec<Set>>,
}

impl Ear {
    pub fn new(
        tag: Option<&str>,
        multi_hum: bool,
        stem_set: Option<Set>,
        sets: Option<Vec<Set>>,
    ) -> Ear {
        Self {
            tag: tag.unwrap_or(DEF_EAR_TAG).to_string(),
            multi_hum,
            stem_set,
            sets: Cell::new(sets.unwrap_or(Vec::new())),
        }
    }
    pub fn new_mono_hum(tag: Option<&str>, multi_set: bool, hum: Hum) -> Ear {
        let stem_set = if multi_set {
            Some(Set::new(vec![hum.clone()]))
        } else {
            None
        };

        Self {
            tag: tag.unwrap_or(DEF_EAR_TAG).to_string(),
            multi_hum: false,
            stem_set,
            sets: Cell::new(vec![Set::new(vec![hum])]),
        }
    }
    pub fn tag<'a>(&'a self) -> &'a String {
        &self.tag
    }
    pub fn sets<'a>(&'a self) -> &'a Vec<Set> {
        unsafe { self.sets.as_ptr().as_mut().unwrap() }
    }
    pub fn is_multi_set(&self) -> bool {
        self.stem_set.is_some()
    }
    pub fn is_multi_hum(&self) -> bool {
        self.multi_hum
    }
    pub fn is_listening_talker(&self, id: Id) -> bool {
        for set in self.sets().iter() {
            for hum in &set.hums {
                for talk in &hum.talks {
                    if talk.talker().id() == id {
                        return true;
                    }
                }
            }
        }
        false
    }
    /*
        pub fn is_listening_talker_ports(&self, id: Id) -> HashSet<Index> {
            let mut talker_ports = HashSet::new();

            for set in self.sets().iter() {
                for hum in &set.hums {
                    for talk in &hum.talks {
                        if talk.talker().id() == id {
                            talker_ports.insert(talk.port);
                        }
                    }
                }
            }
            talker_ports
        }
    */
    pub fn sets_len(&self) -> usize {
        self.sets().len()
    }
    pub fn hums_len(&self) -> usize {
        if let Some(stem_set) = &self.stem_set {
            return stem_set.hums.len();
        } else if self.sets().len() > 0 {
            return self.sets()[0].hums.len();
        }
        return 0;
    }

    pub fn hum_range(&self, hum_idx: Index) -> (f32, f32, f32) {
        if let Some(stem_set) = &self.stem_set {
            if stem_set.hums.len() > hum_idx {
                return stem_set.hums[hum_idx].range();
            }
        } else if self.sets().len() > 0 && self.sets()[0].hums.len() > hum_idx {
            return self.sets()[0].hums[hum_idx].range();
        }
        return (0., 0., 0.);
    }

    pub fn talk_def_value(&self, hum_idx: Index) -> f32 {
        if let Some(stem_set) = &self.stem_set {
            if stem_set.hums.len() > hum_idx {
                return stem_set.hums[hum_idx].def_value();
            }
        } else if self.sets().len() > 0 && self.sets()[0].hums.len() > hum_idx {
            return self.sets()[0].hums[hum_idx].def_value();
        }
        return 0.;
    }

    pub fn talk_value_or_default(&self, set_idx: Index, hum_idx: Index) -> f32 {
        if let Some(stem_set) = &self.stem_set {
            if stem_set.hums.len() > hum_idx {
                return stem_set.hums[hum_idx].value_or_default();
            }
        } else if self.sets().len() > 0 && self.sets()[set_idx].hums.len() > hum_idx {
            return self.sets()[set_idx].hums[hum_idx].value_or_default();
        }
        return 0.;
    }

    pub fn get_set_hum_audio_buffer(&self, set_idx: Index, hum_idx: Index) -> AudioBuf {
        self.sets()[set_idx].get_hum_audio_buffer(hum_idx)
    }
    pub fn get_set_audio_buffer(&self, set_idx: Index) -> AudioBuf {
        self.get_set_hum_audio_buffer(set_idx, 0)
    }
    pub fn get_audio_buffer(&self) -> AudioBuf {
        self.get_set_audio_buffer(0)
    }

    pub fn get_set_hum_control_buffer(&self, set_idx: Index, hum_idx: Index) -> ControlBuf {
        self.sets()[set_idx].get_hum_control_buffer(hum_idx)
    }
    pub fn get_set_control_buffer(&self, set_idx: Index) -> ControlBuf {
        self.get_set_hum_control_buffer(set_idx, 0)
    }
    pub fn get_control_buffer(&self) -> ControlBuf {
        self.get_set_control_buffer(0)
    }
    pub fn get_control_value(&self) -> ControlVal {
        self.get_set_control_buffer(0)[0]
    }

    pub fn get_set_hum_cv_buffer(&self, set_idx: Index, hum_idx: Index) -> CvBuf {
        self.sets()[set_idx].get_hum_cv_buffer(hum_idx)
    }
    pub fn get_set_cv_buffer(&self, set_idx: Index) -> CvBuf {
        self.get_set_hum_cv_buffer(set_idx, 0)
    }
    pub fn get_cv_buffer(&self) -> CvBuf {
        self.get_set_cv_buffer(0)
    }

    pub fn get_atom_buffer(&self) -> AtomBuf {
        self.sets()[0].hums[0].atom_buffer()
    }

    pub fn listen(&self, tick: i64, len: usize) -> usize {
        let mut ln = len;

        for set in self.sets().iter() {
            for hum in &set.hums {
                ln = hum.listen(tick, ln);
            }
        }
        ln
    }

    pub fn iter_talks<F, P>(&self, mut f: F, p: &mut P) -> Result<(), failure::Error>
    where
        F: FnMut(&Talk, &mut P) -> Result<(), failure::Error>,
    {
        for set in self.sets().iter() {
            for hum in &set.hums {
                for talk in &hum.talks {
                    f(&talk, p)?;
                }
            }
        }
        Ok(())
    }

    pub fn fold_talks<F, P>(&self, mut f: F, p: P) -> Result<P, failure::Error>
    where
        F: FnMut(&str, Index, &str, &Talk, P) -> Result<P, failure::Error>,
    {
        let mut acc = p;
        for (set_idx, set) in self.sets().iter().enumerate() {
            for hum in &set.hums {
                for talk in &hum.talks {
                    acc = f(&self.tag, set_idx, hum.tag(), &talk, acc)?;
                }
            }
        }
        Ok(acc)
    }

    pub fn iter_talkers<F, P>(&self, mut f: F, p: &mut P) -> Result<(), failure::Error>
    where
        F: FnMut(&RTalker, &mut P) -> Result<(), failure::Error>,
    {
        self.iter_talks(|tlk, p| f(&tlk.talker, p), p)
    }

    pub fn visit_horn<F>(&self, f: F)
    where
        F: FnMut(&Horn),
    {
        if self.sets().len() == 1 && self.sets()[0].hums.len() == 1 {
            self.sets()[0].hums[0].visit_horn(f);
        }
    }

    fn add_set(&self) -> Result<Index, failure::Error> {
        if let Some(stem_set) = &self.stem_set {
            let set_idx = self.sets().len();
            let mut sets: Vec<Set> = Vec::with_capacity(set_idx + 1);

            for set in self.sets().iter() {
                sets.push(set.clone());
            }
            sets.push(stem_set.clone());

            self.sets.set(sets);

            Ok(set_idx)
        } else {
            return Err(failure::err_msg(format!(
                "Ear {} stem set not found!",
                self.tag()
            )));
        }
    }
    pub fn sup_set(&self, set_idx: Index) -> Result<(), failure::Error> {
        let mut sets: Vec<Set> = Vec::with_capacity(self.sets().len() - 1);

        for (i, set) in self.sets().iter().enumerate() {
            if i != set_idx {
                sets.push(set.clone());
            }
        }
        self.sets.set(sets);
        Ok(())
    }

    pub fn visit_set<F, P>(&self, set_idx: Index, mut f: F, p: P) -> Result<P, failure::Error>
    where
        F: FnMut(&Set, P) -> Result<P, failure::Error>,
    {
        if let Some(set) = self.sets().get(set_idx) {
            f(set, p)
        } else {
            Err(failure::err_msg(format!("Ear set {} not found!", set_idx)))
        }
    }

    pub fn clone_hum(&self, set_idx: Index, hum_idx: Index) -> Result<Hum, failure::Error> {
        if set_idx < self.sets().len() && hum_idx < self.sets()[set_idx].hums.len() {
            Ok(self.sets()[set_idx].hums[hum_idx].clone())
        } else {
            Err(failure::err_msg(format!(
                "Ear set {} hum {} not found!",
                set_idx, hum_idx
            )))
        }
    }

    pub fn set_hum<F>(&self, set_idx: Index, hum_idx: Index, map: F) -> Result<(), failure::Error>
    where
        F: FnMut(&Hum) -> Result<Hum, failure::Error>,
    {
        let old_sets = self.sets();
        let sets_len = old_sets.len();
        let mut new_sets: Vec<Set> = Vec::with_capacity(sets_len);

        for i in 0..Index::min(set_idx, sets_len) {
            new_sets.push(old_sets[i].clone());
        }
        if set_idx < sets_len {
            new_sets.push(old_sets[set_idx].with_hum(hum_idx, map)?);

            for i in (set_idx + 1)..sets_len {
                new_sets.push(old_sets[i].clone());
            }
        } else {
            if let Some(stem_set) = &self.stem_set {
                for _ in sets_len..(set_idx) {
                    new_sets.push(stem_set.clone());
                }
                new_sets.push(stem_set.with_hum(hum_idx, map)?);
            } else {
                return Err(failure::err_msg(format!(
                    "Ear {} stem set not found!",
                    self.tag()
                )));
            }
        }

        self.sets.set(new_sets);
        Ok(())
    }

    pub fn find_hum_index(&self, hum_tag: &str) -> Result<Index, failure::Error> {
        if let Some(set) = &self.stem_set {
            set.find_hum_index(hum_tag)
        } else if self.sets().len() > 0 {
            self.sets()[0].find_hum_index(hum_tag)
        } else {
            Err(failure::err_msg(format!("hum {} not found!", hum_tag)))
        }
    }

    pub fn set_hum_value_by_tag(
        &self,
        set_idx: Index,
        hum_tag: &str,
        value: f32,
    ) -> Result<(), failure::Error> {
        let hum_idx = self.find_hum_index(hum_tag)?;
        self.set_hum(set_idx, hum_idx, |hum| hum.set_value(value))
    }
    pub fn set_hum_voice_by_tag(
        &self,
        set_idx: Index,
        hum_tag: &str,
        talker: &RTalker,
        port: usize,
    ) -> Result<(), failure::Error> {
        let hum_idx = self.find_hum_index(hum_tag)?;
        self.set_hum(set_idx, hum_idx, |hum| hum.set_voice(talker, port))
    }
    pub fn set_hum_value(
        &self,
        set_idx: Index,
        hum_idx: Index,
        value: f32,
    ) -> Result<(), failure::Error> {
        self.set_hum(set_idx, hum_idx, |hum| hum.set_value(value))
    }
    pub fn set_hum_voice(
        &self,
        set_idx: Index,
        hum_idx: Index,
        talker: &RTalker,
        port: usize,
    ) -> Result<(), failure::Error> {
        self.set_hum(set_idx, hum_idx, |hum| hum.set_voice(talker, port))
    }
    pub fn set_talk_value(
        &self,
        set_idx: Index,
        hum_idx: Index,
        talk_idx: Index,
        value: f32,
    ) -> Result<(), failure::Error> {
        self.set_hum(set_idx, hum_idx, |hum| hum.set_talk_value(talk_idx, value))
    }
    pub fn set_talk_voice(
        &self,
        set_idx: Index,
        hum_idx: Index,
        talk_idx: Index,
        talker: &RTalker,
        port: usize,
    ) -> Result<(), failure::Error> {
        self.set_hum(set_idx, hum_idx, |hum| {
            hum.set_talk_voice(talk_idx, talker, port)
        })
    }
    pub fn add_value_to_hum(
        &self,
        set_idx: Index,
        hum_idx: Index,
        value: f32,
    ) -> Result<(), failure::Error> {
        self.set_hum(set_idx, hum_idx, |hum| hum.add_value(value))
    }

    pub fn add_voice_to_hum(
        &self,
        set_idx: Index,
        hum_idx: Index,
        voice_talker: &RTalker,
        port: usize,
    ) -> Result<(), failure::Error> {
        self.set_hum(set_idx, hum_idx, |hum| hum.add_voice(voice_talker, port))
    }

    pub fn sup_talk(
        &self,
        set_idx: Index,
        hum_idx: Index,
        talk_idx: Index,
    ) -> Result<(), failure::Error> {
        self.set_hum(set_idx, hum_idx, |hum| hum.sup_talk(talk_idx))
    }

    pub fn replace_talker(
        &self,
        talker_id: Id,
        new_talker: &RTalker,
    ) -> Result<(), failure::Error> {
        let old_sets = self.sets();
        let mut new_sets: Vec<Set> = Vec::with_capacity(old_sets.len());

        for set in old_sets.iter() {
            let mut hums = Vec::with_capacity(set.hums.len());

            for hum in &set.hums {
                let mut talks = Vec::with_capacity(hum.talks.len());

                for talk in &hum.talks {
                    if talk.talker().id() == talker_id {
                        talks.push(Talk::new(new_talker, talk.port));
                    } else {
                        talks.push(talk.clone());
                    }
                }
                hums.push(hum.with_talks(talks));
            }
            new_sets.push(Set::new(hums));
        }
        self.sets.set(new_sets);
        Ok(())
    }

    pub fn sup_talker(&self, talker_id: Id) -> Result<(), failure::Error> {
        let old_sets = self.sets();
        let mut new_sets: Vec<Set> = Vec::with_capacity(old_sets.len());

        for set in old_sets.iter() {
            let mut hums = Vec::with_capacity(set.hums.len());

            for hum in &set.hums {
                let mut talks = Vec::with_capacity(hum.talks.len());

                for talk in &hum.talks {
                    if talk.talker().id() != talker_id {
                        talks.push(talk.clone());
                    }
                }
                if talks.is_empty() {
                    talks.push(def_talk(hum.port_type, 0.));
                }
                hums.push(hum.with_talks(talks));
            }
            new_sets.push(Set::new(hums));
        }
        self.sets.set(new_sets);
        Ok(())
    }

    pub fn sup_talker_ports(
        &self,
        talker_id: Id,
        talker_ports: &HashSet<Index>,
    ) -> Result<(), failure::Error> {
        let mut ports_to_suppress = HashSet::new();

        for set in self.sets().iter() {
            for hum in &set.hums {
                for talk in &hum.talks {
                    if talk.talker().id() == talker_id && talker_ports.contains(&talk.port) {
                        ports_to_suppress.insert(talk.port);
                    }
                }
            }
        }

        if !ports_to_suppress.is_empty() {
            let old_sets = self.sets();
            let mut new_sets: Vec<Set> = Vec::with_capacity(old_sets.len());

            for set in old_sets.iter() {
                let mut hums = Vec::with_capacity(set.hums.len());

                for hum in &set.hums {
                    let mut talks = Vec::with_capacity(hum.talks.len());

                    for talk in &hum.talks {
                        if talk.talker().id() != talker_id
                            || !ports_to_suppress.contains(&talk.port)
                        {
                            talks.push(talk.clone());
                        }
                    }
                    if talks.is_empty() {
                        talks.push(def_talk(hum.port_type, 0.));
                    }
                    hums.push(hum.with_talks(talks));
                }
                new_sets.push(Set::new(hums));
            }
            self.sets.set(new_sets);
        }
        Ok(())
    }

    pub fn add_set_value(&self, hum_idx: Index, value: f32) -> Result<(), failure::Error> {
        let set_idx = self.add_set()?;
        self.set_hum(set_idx, hum_idx, |hum| hum.add_value(value))
    }

    pub fn add_set_voice(
        &self,
        hum_idx: Index,
        voice_talker: &RTalker,
        port: usize,
    ) -> Result<(), failure::Error> {
        let set_idx = self.add_set()?;
        self.set_hum(set_idx, hum_idx, |hum| hum.add_voice(voice_talker, port))
    }
}

fn mono_hum(
    tag: Option<&str>,
    port_type: PortType,
    multi_set: bool,
    min_value: f32,
    max_value: f32,
    def_value: f32,
    init: &Init,
) -> Result<Ear, failure::Error> {
    Ok(Ear::new_mono_hum(
        tag,
        multi_set,
        Hum::initialized(None, port_type, min_value, max_value, def_value, init)?,
    ))
}

pub fn audio(
    tag: Option<&str>,
    min_value: f32,
    max_value: f32,
    def_value: f32,
    init: &Init,
) -> Result<Ear, failure::Error> {
    mono_hum(
        tag,
        PortType::Audio,
        false,
        min_value,
        max_value,
        def_value,
        init,
    )
}

pub fn control(
    tag: Option<&str>,
    min_value: f32,
    max_value: f32,
    def_value: f32,
) -> Result<Ear, failure::Error> {
    let hum = Hum::initialized(
        None,
        PortType::Control,
        min_value,
        max_value,
        def_value,
        &Init::DefValue,
    )?;
    Ok(Ear::new_mono_hum(tag, false, hum))
}

pub fn cv(
    tag: Option<&str>,
    min_value: f32,
    max_value: f32,
    def_value: f32,
    init: &Init,
) -> Result<Ear, failure::Error> {
    mono_hum(
        tag,
        PortType::Cv,
        false,
        min_value,
        max_value,
        def_value,
        init,
    )
}

pub fn atom(tag: Option<&str>) -> Result<Ear, failure::Error> {
    mono_hum(tag, PortType::Atom, false, 0., 0., 0., &Init::DefValue)
}

pub fn multi_set(
    tag: Option<&str>,
    port_type: PortType,
    min_value: f32,
    max_value: f32,
    def_value: f32,
    init: &Init,
) -> Result<Ear, failure::Error> {
    mono_hum(tag, port_type, true, min_value, max_value, def_value, init)
}

pub fn controls(
    tag: Option<&str>,
    min_value: f32,
    max_value: f32,
    def_value: f32,
) -> Result<Ear, failure::Error> {
    multi_set(
        tag,
        PortType::Control,
        min_value,
        max_value,
        def_value,
        &Init::DefValue,
    )
}

pub fn audios(
    tag: Option<&str>,
    min_value: f32,
    max_value: f32,
    def_value: f32,
    init: &Init,
) -> Result<Ear, failure::Error> {
    multi_set(tag, PortType::Audio, min_value, max_value, def_value, init)
}

pub fn cvs(
    tag: Option<&str>,
    min_value: f32,
    max_value: f32,
    def_value: f32,
    init: &Init,
) -> Result<Ear, failure::Error> {
    multi_set(tag, PortType::Cv, min_value, max_value, def_value, init)
}

pub fn set(
    tag: Option<&str>,
    multi_set: bool,
    hums_attributs: &Vec<(&str, PortType, f32, f32, f32, Init)>,
) -> Result<Ear, failure::Error> {
    let multi_hum = hums_attributs.len() > 1;
    let initial_set = Set::from_attributs(hums_attributs)?;
    let stem_set = if multi_set {
        Some(initial_set.clone())
    } else {
        None
    };

    Ok(Ear::new(tag, multi_hum, stem_set, Some(vec![initial_set])))
}
