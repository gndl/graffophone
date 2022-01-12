use std::cell::RefCell;
use std::f32;
use std::rc::Rc;

use audio_talker::AudioTalker;
use control_talker::ControlTalker;
use cv_talker::CvTalker;
use horn::{AudioBuf, ControlBuf, CvBuf, Horn};
use identifier::Index;
use talker::RTalker;
use voice::PortType;

const DEF_EAR_TAG: &'static str = "In";
const DEF_HUM_TAG: &'static str = "";

pub fn def_audio_talker(value: f32) -> RTalker {
    Rc::new(RefCell::new(AudioTalker::new(value, Some(true))))
}
pub fn def_control_talker(value: f32) -> RTalker {
    Rc::new(RefCell::new(ControlTalker::new(value, Some(true))))
}
pub fn def_cv_talker(value: f32) -> RTalker {
    Rc::new(RefCell::new(CvTalker::new(value, Some(true))))
}
pub fn def_talker(port_type: PortType, value: f32) -> RTalker {
    match port_type {
        PortType::Audio => def_audio_talker(value),
        PortType::Control => def_control_talker(value),
        PortType::Cv => def_cv_talker(value),
    }
}

pub struct Talk {
    tkr: RTalker,
    port: Index,
}

pub type RTalk = RefCell<Talk>;

impl Talk {
    pub fn new(talker: RTalker, port: Index) -> RTalk {
        RefCell::new(Self {
            tkr: talker,
            port: port,
        })
    }

    pub fn clone(&self) -> Talk {
        Self {
            tkr: self.tkr.clone(),
            port: self.port,
        }
    }

    pub fn talker<'a>(&'a self) -> &'a RTalker {
        &self.tkr
    }
    pub fn port(&self) -> Index {
        self.port
    }
    pub fn value(&self) -> Option<f32> {
        self.tkr.borrow().voice_value(self.port)
    }
    pub fn visit_horn<F>(&self, mut f: F)
    where
        F: FnMut(&Horn),
    {
        let tkr = self.tkr.borrow();
        {
            match tkr.voices().get(self.port) {
                Some(voice) => f(voice.borrow().horn()),
                None => (),
            }
        }
    }

    pub fn audio_buffer(&self) -> Option<AudioBuf> {
        let res;
        let tkr = self.tkr.borrow();
        {
            let voice = tkr.voices().get(self.port)?;
            res = voice.borrow().audio_buffer();
        }
        res
    }
    pub fn control_buffer(&self) -> Option<ControlBuf> {
        let res;
        let tkr = self.tkr.borrow();
        {
            let voice = tkr.voices().get(self.port)?;
            res = voice.borrow().control_buffer();
        }
        res
    }
    pub fn cv_buffer(&self) -> Option<CvBuf> {
        let res;
        let tkr = self.tkr.borrow();
        {
            let voice = tkr.voices().get(self.port)?;
            res = voice.borrow().cv_buffer();
        }
        res
    }

    pub fn set(&mut self, tkr: RTalker, port: Index) {
        self.tkr = tkr;
        self.port = port;
    }

    pub fn listen(&self, tick: i64, len: usize) -> usize {
        let port = self.port;
        {
            let tkr = self.tkr.borrow();
            let voice = tkr.voices().get(port).unwrap().borrow();

            if tick == voice.tick() {
                return usize::min(len, voice.len());
            }
        }

        self.tkr.borrow_mut().talk(port, tick, len)
    }
}

pub fn def_audio_talk(value: f32) -> RTalk {
    Talk::new(def_audio_talker(value), 0)
}
pub fn def_control_talk(value: f32) -> RTalk {
    Talk::new(def_control_talker(value), 0)
}
pub fn def_cv_talk(value: f32) -> RTalk {
    Talk::new(def_cv_talker(value), 0)
}
pub fn def_talk(port_type: PortType, value: f32) -> RTalk {
    match port_type {
        PortType::Audio => Talk::new(def_audio_talker(value), 0),
        PortType::Control => Talk::new(def_control_talker(value), 0),
        PortType::Cv => Talk::new(def_cv_talker(value), 0),
    }
}

pub struct Hum {
    tag: String,
    port_type: PortType,
    min_value: f32,
    max_value: f32,
    def_value: f32,
    talks: Vec<RTalk>,
    horn: Option<Horn>,
}
pub type RHum = RefCell<Hum>;

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
    ) -> Hum {
        Self {
            tag: tag.unwrap_or(DEF_HUM_TAG).to_string(),
            port_type,
            min_value,
            max_value,
            def_value,
            talks: Vec::new(),
            horn: None,
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
        let mut hum = Hum::new(tag, port_type, min_value, max_value, def_value);

        match init {
            Init::Empty => (),
            Init::DefValue => hum.add_value(def_value)?,
            Init::Value(v) => hum.add_value(*v)?,
            Init::Voice(tkr, port) => hum.add_voice(tkr, *port)?,
        }

        Ok(hum)
    }

    pub fn clone(&self) -> Hum {
        let mut talks = Vec::new();

        for talk in &self.talks {
            talks.push(RefCell::new(talk.borrow().clone()));
        }

        let horn = if talks.len() > 1 {
            Some(self.port_type.to_horn())
        } else {
            None
        };

        Self {
            tag: self.tag.to_string(),
            port_type: self.port_type,
            min_value: self.min_value,
            max_value: self.max_value,
            def_value: self.def_value,
            talks,
            horn,
        }
    }

    pub fn visit_horn<F>(&self, mut f: F)
    where
        F: FnMut(&Horn),
    {
        if self.talks.len() == 1 {
            self.talks[0].borrow().visit_horn(f)
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
    pub fn value(&self) -> Option<f32> {
        for talk in &self.talks {
            match talk.borrow().value() {
                Some(v) => return Some(v),
                _ => (),
            }
        }
        None
    }
    pub fn value_or_default(&self) -> f32 {
        self.value().unwrap_or(self.def_value)
    }
    pub fn audio_buffer(&self) -> Option<AudioBuf> {
        if self.talks.len() > 1 {
            self.horn.as_ref().unwrap().audio_buffer()
        } else if self.talks.len() == 1 {
            self.talks[0].borrow().audio_buffer()
        } else {
            None
        }
    }
    pub fn control_buffer(&self) -> Option<ControlBuf> {
        if self.talks.len() > 1 {
            self.horn.as_ref().unwrap().control_buffer()
        } else if self.talks.len() == 1 {
            self.talks[0].borrow().control_buffer()
        } else {
            None
        }
    }
    pub fn cv_buffer(&self) -> Option<CvBuf> {
        if self.talks.len() > 1 {
            self.horn.as_ref().unwrap().cv_buffer()
        } else if self.talks.len() == 1 {
            self.talks[0].borrow().cv_buffer()
        } else {
            println!("Hum::cv_buffer nb talks = {}", self.talks.len());
            None
        }
    }
    pub fn talks<'a>(&'a self) -> &'a Vec<RTalk> {
        &self.talks
    }
    pub fn reproduce(&self) -> Hum {
        self.clone()
        /*
                Hum {
                    horn: None,
                    port_type: self.port_type,
                    tag: self.tag.to_string(),
                    min_value: self.min_value,
                    max_value: self.max_value,
                    def_value: self.def_value,
                    talks: Vec::new(),
                }
        */
    }
    fn check(&mut self) -> Result<(), failure::Error> {
        match self.horn {
            Some(_) => {
                if self.talks.len() < 2 {
                    self.horn = None;
                }
            }
            None => {
                if self.talks.len() > 1 {
                    self.horn = Some(self.port_type.to_horn());
                }
            }
        }
        Ok(())
    }
    pub fn add_talk(&mut self, talk: RTalk) -> Result<(), failure::Error> {
        self.talks.push(talk);
        self.check()
    }
    pub fn add_value(&mut self, value: f32) -> Result<(), failure::Error> {
        self.add_talk(def_talk(self.port_type, value))
    }
    pub fn add_voice(&mut self, talker: &RTalker, port: Index) -> Result<(), failure::Error> {
        if self
            .port_type()
            .can_hear(talker.borrow().voice_port_type(port))
        {
            self.add_talk(Talk::new(talker.clone(), port))
        } else {
            Err(failure::err_msg(format!(
                "Talker {} voice {} type is not compatible with talks {}!",
                talker.borrow().name(),
                port,
                self.tag
            )))
        }
    }
    pub fn set_value(&mut self, value: f32) -> Result<(), failure::Error> {
        self.talks.clear();
        self.add_value(value)
    }
    pub fn set_voice(&mut self, talker: &RTalker, port: Index) -> Result<(), failure::Error> {
        self.talks.clear();
        self.add_voice(talker, port)
    }
    pub fn sup_talk(&mut self, index: Index) -> Result<(), failure::Error> {
        let _ = self.talks.remove(index);
        self.check()
    }

    pub fn set_talk_value(&self, talk_idx: Index, value: f32) -> Result<(), failure::Error> {
        self.talks[talk_idx]
            .borrow_mut()
            .set(def_talker(self.port_type, value), 0);
        Ok(())
    }

    pub fn set_talk_voice(
        &self,
        talk_idx: Index,
        talker: &RTalker,
        port: Index,
    ) -> Result<(), failure::Error> {
        if self
            .port_type()
            .can_hear(talker.borrow().voice_port_type(port))
        {
            self.talks[talk_idx].borrow_mut().set(talker.clone(), port);
            Ok(())
        } else {
            Err(failure::err_msg(format!(
                "Talker {} voice {} type {} is not compatible with {} type {}!",
                talker.borrow().name(),
                port,
                talker.borrow().voice_port_type(port).to_string(),
                self.tag(),
                self.port_type.to_string()
            )))
        }
    }

    pub fn listen(&self, tick: i64, len: usize) -> usize {
        let mut ln = len;

        for talk in &self.talks {
            ln = talk.borrow().listen(tick, ln);
        }

        if self.talks.len() > 1 {
            match &self.horn.as_ref().unwrap() {
                Horn::Audio(out_buf) => {
                    let first_buf = self.talks[0].borrow().audio_buffer().unwrap();

                    for i in 0..ln {
                        out_buf.get()[i].set(first_buf.get()[i].get());
                    }

                    for ti in 1..self.talks.len() {
                        let in_buf = self.talks[ti].borrow().audio_buffer().unwrap();

                        for i in 0..ln {
                            let v = out_buf.get()[i].get() + in_buf.get()[i].get();
                            out_buf.get()[i].set(v);
                        }
                    }
                    let c = 1. / (self.talks.len() as f32);

                    for i in 0..ln {
                        let v = out_buf.get()[i].get() * c;
                        out_buf.get()[i].set(v);
                    }
                }
                Horn::Control(out_buf) => {
                    let first_buf = self.talks[0].borrow().control_buffer().unwrap();
                    let mut v = first_buf.get();

                    for ti in 1..self.talks.len() {
                        let in_buf = self.talks[ti].borrow().control_buffer().unwrap();
                        v += in_buf.get();
                    }
                    out_buf.set(v / (self.talks.len() as f32));
                }
                Horn::Cv(out_buf) => {
                    let first_buf = self.talks[0].borrow().cv_buffer().unwrap();

                    for i in 0..ln {
                        out_buf.get()[i].set(first_buf.get()[i].get());
                    }

                    for ti in 1..self.talks.len() {
                        let in_buf = self.talks[ti].borrow().cv_buffer().unwrap();

                        for i in 0..ln {
                            let v = out_buf.get()[i].get() + in_buf.get()[i].get();
                            out_buf.get()[i].set(v);
                        }
                    }
                    let c = 1. / (self.talks.len() as f32);

                    for i in 0..ln {
                        let v = out_buf.get()[i].get() * c;
                        out_buf.get()[i].set(v);
                    }
                }
            }
        }
        ln
    }
}

pub struct Set {
    hums: Vec<RHum>,
}

impl Set {
    pub fn new(hums: Vec<RHum>) -> Set {
        Self { hums }
    }

    pub fn from_attributs(
        hums_attributs: &Vec<(&str, PortType, f32, f32, f32, Init)>,
    ) -> Result<Set, failure::Error> {
        let mut hums = Vec::new();

        for (tag, port_type, min_value, max_value, def_value, init) in hums_attributs {
            let hum_tag = if tag.len() > 0 { tag } else { DEF_EAR_TAG };

            hums.push(RefCell::new(Hum::initialized(
                Some(hum_tag),
                *port_type,
                *min_value,
                *max_value,
                *def_value,
                init,
            )?));
        }
        Ok(Self { hums })
    }

    pub fn hums<'a>(&'a self) -> &'a Vec<RHum> {
        &self.hums
    }

    pub fn hums_len(&self) -> usize {
        self.hums.len()
    }

    pub fn reproduce(&self) -> Set {
        let mut hums: Vec<RHum> = Vec::new();

        for hum in &self.hums {
            hums.push(RefCell::new(hum.borrow().reproduce()));
        }
        Set { hums }
    }

    pub fn get_hum_audio_buffer(&self, hum_idx: Index) -> Option<AudioBuf> {
        self.hums[hum_idx].borrow().audio_buffer()
    }

    pub fn get_hum_cv_buffer(&self, hum_idx: Index) -> Option<CvBuf> {
        self.hums[hum_idx].borrow().cv_buffer()
    }
}

pub struct Ear {
    tag: String,
    multi_hum: bool,
    stem_set: Option<Set>,
    sets: RefCell<Vec<Set>>,
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
            sets: RefCell::new(sets.unwrap_or(Vec::new())),
        }
    }
    pub fn new_mono_hum(tag: Option<&str>, multi_set: bool, hum: Hum) -> Ear {
        let stem_set = if multi_set {
            Some(Set::new(vec![RefCell::new(hum.reproduce())]))
        } else {
            None
        };

        Self {
            tag: tag.unwrap_or(DEF_EAR_TAG).to_string(),
            multi_hum: false,
            stem_set,
            sets: RefCell::new(vec![Set::new(vec![RefCell::new(hum)])]),
        }
    }
    pub fn tag<'a>(&'a self) -> &'a String {
        &self.tag
    }
    pub fn sets<'a>(&'a self) -> &'a RefCell<Vec<Set>> {
        &self.sets
    }
    pub fn is_multi_set(&self) -> bool {
        self.stem_set.is_some()
    }
    pub fn is_multi_hum(&self) -> bool {
        self.multi_hum
    }
    pub fn sets_len(&self) -> usize {
        self.sets.borrow().len()
    }
    pub fn hums_len(&self) -> usize {
        self.sets.borrow()[0].hums.len()
    }

    pub fn hum_range(&self, hum_idx: usize) -> (f32, f32, f32) {
        if self.sets.borrow().len() > 0 && self.sets.borrow()[0].hums.len() > hum_idx {
            return self.sets.borrow()[0].hums[hum_idx].borrow().range();
        }
        return (0., 0., 0.);
    }

    pub fn talk_def_value(&self, hum_idx: usize) -> f32 {
        if self.sets.borrow().len() > 0 && self.sets.borrow()[0].hums.len() > hum_idx {
            return self.sets.borrow()[0].hums[hum_idx].borrow().def_value();
        }
        return 0.;
    }

    pub fn talk_value_or_default(&self, set_idx: usize, hum_idx: usize) -> f32 {
        if self.sets.borrow().len() > 0 && self.sets.borrow()[set_idx].hums.len() > hum_idx {
            return self.sets.borrow()[set_idx].hums[hum_idx]
                .borrow()
                .value_or_default();
        }
        return 0.;
    }

    pub fn get_set_hum_audio_buffer(&self, set_idx: Index, hum_idx: Index) -> Option<AudioBuf> {
        self.sets.borrow()[set_idx].get_hum_audio_buffer(hum_idx)
    }
    pub fn get_set_audio_buffer(&self, set_idx: Index) -> Option<AudioBuf> {
        self.get_set_hum_audio_buffer(set_idx, 0)
    }
    pub fn get_audio_buffer(&self) -> Option<AudioBuf> {
        self.get_set_audio_buffer(0)
    }

    pub fn get_set_hum_cv_buffer(&self, set_idx: Index, hum_idx: Index) -> Option<CvBuf> {
        self.sets.borrow()[set_idx].get_hum_cv_buffer(hum_idx)
    }
    pub fn get_set_cv_buffer(&self, set_idx: Index) -> Option<CvBuf> {
        self.get_set_hum_cv_buffer(set_idx, 0)
    }
    pub fn get_cv_buffer(&self) -> Option<CvBuf> {
        self.get_set_cv_buffer(0)
    }

    pub fn listen(&self, tick: i64, len: usize) -> usize {
        let mut ln = len;

        for set in self.sets.borrow().iter() {
            for hum in &set.hums {
                ln = hum.borrow().listen(tick, ln);
            }
        }
        ln
    }

    pub fn iter_talks<F, P>(&self, mut f: F, p: &mut P) -> Result<(), failure::Error>
    where
        F: FnMut(&Talk, &mut P) -> Result<(), failure::Error>,
    {
        for set in self.sets.borrow().iter() {
            for hum in &set.hums {
                for talk in &hum.borrow().talks {
                    f(&talk.borrow(), p)?;
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
        for (set_idx, set) in self.sets.borrow().iter().enumerate() {
            for hum in &set.hums {
                for talk in &hum.borrow().talks {
                    acc = f(&self.tag, set_idx, hum.borrow().tag(), &talk.borrow(), acc)?;
                }
            }
        }
        Ok(acc)
    }

    pub fn iter_talkers<F, P>(&self, mut f: F, p: &mut P) -> Result<(), failure::Error>
    where
        F: FnMut(&RTalker, &mut P) -> Result<(), failure::Error>,
    {
        self.iter_talks(|tlk, p| f(&tlk.tkr, p), p)
    }

    pub fn visit_horn<F>(&self, f: F)
    where
        F: FnMut(&Horn),
    {
        if self.sets.borrow().len() == 1 && self.sets.borrow()[0].hums.len() == 1 {
            self.sets.borrow()[0].hums[0].borrow().visit_horn(f);
        }
    }

    fn provide_set(&self, set_idx: Index) {
        if let Some(stem_set) = &self.stem_set {
            let mut new_sets = Vec::new();

            for _ in self.sets.borrow().len()..(set_idx + 1) {
                new_sets.push(stem_set.reproduce());
            }
            self.sets.borrow_mut().append(&mut new_sets);
        }
    }
    fn add_set(&self) -> Index {
        /*
        if self.sets.borrow().len() == 0 {
            self.sets.borrow_mut().push(Set::new(Vec::new()));
        } else {
            let new_set = self.sets.borrow()[0].reproduce();
            self.sets.borrow_mut().push(new_set);
        }
         */
        let n = self.sets.borrow().len();
        self.provide_set(n);
        self.sets.borrow().len() - 1
    }
    /*
    fn add_set(&self, oset: Option<Set>) -> Index {
        if Some(set) = oset {
            self.sets.borrow_mut().push(set);
        } else {
            let new_set = self.sets.borrow()[0].reproduce();
            self.sets.borrow_mut().push(new_set);
        }
        self.sets.borrow().len() - 1
    }
    */
    pub fn sup_set(&self, set_idx: Index) -> Result<(), failure::Error> {
        self.sets.borrow_mut().remove(set_idx);
        Ok(())
    }

    pub fn visit_set<F, P>(&self, set_idx: Index, mut f: F, p: P) -> Result<P, failure::Error>
    where
        F: FnMut(&Set, P) -> Result<P, failure::Error>,
    {
        if let Some(set) = self.sets.borrow().get(set_idx) {
            f(set, p)
        } else {
            Err(failure::err_msg(format!("Ear set {} not found!", set_idx)))
        }
    }

    pub fn clone_hum(&self, set_idx: Index, hum_idx: Index) -> Result<RHum, failure::Error> {
        if set_idx < self.sets.borrow().len() && hum_idx < self.sets.borrow()[set_idx].hums.len() {
            Ok(RefCell::new(
                self.sets.borrow()[set_idx].hums[hum_idx].borrow().clone(),
            ))
        } else {
            Err(failure::err_msg(format!(
                "Ear set {} hum {} not found!",
                set_idx, hum_idx
            )))
        }
    }

    pub fn add_hum_value_by_tag(
        &self,
        set_idx: Index,
        hum_tag: &str,
        value: f32,
    ) -> Result<(), failure::Error> {
        self.provide_set(set_idx);

        if self.sets.borrow().len() > set_idx {
            for hum in &self.sets.borrow()[set_idx].hums {
                if hum.borrow().tag() == hum_tag {
                    return hum.borrow_mut().add_value(value);
                }
            }
        }
        Err(failure::err_msg(format!(
            "Ear {} hum {} not found!",
            self.tag(),
            hum_tag
        )))
    }
    pub fn add_hum_voice_by_tag(
        &self,
        set_idx: Index,
        hum_tag: &str,
        talker: &RTalker,
        port: usize,
    ) -> Result<(), failure::Error> {
        self.provide_set(set_idx);

        if self.sets.borrow().len() > set_idx {
            for hum in &self.sets.borrow()[set_idx].hums {
                if hum.borrow().tag() == hum_tag {
                    return hum.borrow_mut().add_voice(talker, port);
                }
            }
        }
        Err(failure::err_msg(format!(
            "Ear {} hum {} not found!",
            self.tag(),
            hum_tag
        )))
    }
    pub fn set_hum_value(
        &self,
        set_idx: Index,
        hum_idx: Index,
        value: f32,
    ) -> Result<(), failure::Error> {
        self.sets.borrow()[set_idx].hums[hum_idx]
            .borrow_mut()
            .set_value(value)
    }
    pub fn set_hum_voice(
        &self,
        set_idx: Index,
        hum_idx: Index,
        talker: &RTalker,
        port: usize,
    ) -> Result<(), failure::Error> {
        self.sets.borrow()[set_idx].hums[hum_idx]
            .borrow_mut()
            .set_voice(talker, port)
    }
    pub fn set_talk_value(
        &self,
        set_idx: Index,
        hum_idx: Index,
        talk_idx: Index,
        value: f32,
    ) -> Result<(), failure::Error> {
        self.sets.borrow()[set_idx].hums[hum_idx]
            .borrow()
            .set_talk_value(talk_idx, value)
    }
    pub fn set_talk_voice(
        &self,
        set_idx: Index,
        hum_idx: Index,
        talk_idx: Index,
        talker: &RTalker,
        port: usize,
    ) -> Result<(), failure::Error> {
        self.sets.borrow()[set_idx].hums[hum_idx]
            .borrow()
            .set_talk_voice(talk_idx, talker, port)
    }
    pub fn add_value_to_hum(
        &self,
        set_idx: Index,
        hum_idx: Index,
        value: f32,
    ) -> Result<(), failure::Error> {
        self.sets.borrow()[set_idx].hums[hum_idx]
            .borrow_mut()
            .add_value(value)
    }

    pub fn add_voice_to_hum(
        &self,
        set_idx: Index,
        hum_idx: Index,
        voice_talker: &RTalker,
        port: usize,
    ) -> Result<(), failure::Error> {
        self.sets.borrow()[set_idx].hums[hum_idx]
            .borrow_mut()
            .add_voice(voice_talker, port)
    }

    pub fn sup_talk(
        &self,
        set_idx: Index,
        hum_idx: Index,
        talk_idx: Index,
    ) -> Result<(), failure::Error> {
        self.sets.borrow()[set_idx].hums[hum_idx]
            .borrow_mut()
            .sup_talk(talk_idx)
    }

    pub fn add_set_value(&self, hum_idx: Index, value: f32) -> Result<(), failure::Error> {
        let set_idx = self.add_set();
        self.sets.borrow()[set_idx].hums[hum_idx]
            .borrow_mut()
            .add_value(value)
    }

    pub fn add_set_voice(
        &self,
        hum_idx: Index,
        voice_talker: &RTalker,
        port: usize,
    ) -> Result<(), failure::Error> {
        let set_idx = self.add_set();
        self.sets.borrow()[set_idx].hums[hum_idx]
            .borrow_mut()
            .add_voice(voice_talker, port)
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
    let mut hum = Hum::new(None, PortType::Control, min_value, max_value, def_value);
    hum.add_value(def_value)?;
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
        Some(initial_set.reproduce())
    } else {
        None
    };

    Ok(Ear::new(tag, multi_hum, stem_set, Some(vec![initial_set])))
}

/*
pub fn visit_ear_flatten_index<F>(
    ears: &Vec<Ear>,
    index: Index,
    mut f: F,
) -> Result<(), failure::Error>
where
    F: FnMut(&RTalk) -> Result<(), failure::Error>,
{
    let mut res = Err(failure::err_msg(format!("Ear {} not found!", index)));

    ears.into_iter().try_fold(0, |i, ear| match ear {
        Ear::Talk(talk) => {
            if i == index {
                res = f(talk);
                return None;
            }
            return Some(i + 1);
        }
        Ear::Talks(talks) => {
            let ri = index - i;

            if ri < talks.borrow().talks.len() {
                res = f(talks.borrow().talks.get(ri).unwrap());
                return None;
            }
            return Some(i + talks.borrow().talks.len());
        }
    });
    res
}
*/

/*
fn visit_ear_tag<F>(ears: &Vec<Ear>, tag: &String, f: F)where  -> bool {
    for ear in ears {
            match ear {
                Ear::Talk(talk) => {
                    if talk.borrow().tag() == tag {
                        if f(talk) {
                            return true}
                    }
                }
                Ear::Talks(talks) => {
                    let mut tlks = talks.borrow_mut();

                        for talk in tlks.talks() {
                            if talk.borrow().tag() == tag {
                        if f(talk) {
                            return true}
                            }
                        }
                    }
            }
        }
        false
}
*/
