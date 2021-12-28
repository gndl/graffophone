use std::cell::RefCell;
use std::f32;
use std::rc::Rc;

use audio_talker::AudioTalker;
use control_talker::ControlTalker;
use cv_talker::CvTalker;
use horn::{AudioBuf, CvBuf, Horn};
use identifier::Index;
use talker::RTalker;
use voice::PortType;

pub const DEF_INPUT_TAG: &'static str = "In";

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
    /*
        pub fn horn<'a>(&'a self) -> &'a Horn {
            let res;
            let tkr = self.tkr.borrow();
            {
                let voice = &tkr.voices()[self.port];
                res = voice.borrow().horn();
            }
            res
        }
    */
    pub fn audio_buffer(&self) -> Option<AudioBuf> {
        let res;
        let tkr = self.tkr.borrow();
        {
            let voice = tkr.voices().get(self.port)?;
            res = voice.borrow().audio_buffer();
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

impl Hum {
    pub fn new(
        tag: Option<&str>,
        port_type: PortType,
        min_value: f32,
        max_value: f32,
        def_value: f32,
    ) -> RHum {
        RefCell::new(Self {
            tag: tag.unwrap_or(DEF_INPUT_TAG).to_string(),
            port_type,
            min_value,
            max_value,
            def_value,
            talks: Vec::new(),
            horn: None,
        })
    }

    pub fn from_value(
        tag: Option<&str>,
        port_type: PortType,
        min_value: f32,
        max_value: f32,
        def_value: f32,
    ) -> RHum {
        RefCell::new(Self {
            tag: tag.unwrap_or(DEF_INPUT_TAG).to_string(),
            port_type,
            min_value,
            max_value,
            def_value,
            talks: vec![def_talk(port_type, def_value)],
            horn: None,
        })
    }

    pub fn from_voice(
        tag: Option<&str>,
        min_value: f32,
        max_value: f32,
        talker: &RTalker,
        port: Index,
    ) -> RHum {
        let port_type;
        {
            port_type = talker.borrow().voice_port_type(port);
        }
        RefCell::new(Self {
            tag: tag.unwrap_or(DEF_INPUT_TAG).to_string(),
            port_type,
            min_value,
            max_value,
            def_value: 0.,
            talks: vec![Talk::new(talker.clone(), port)],
            horn: None,
        })
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
    /*
        pub fn horn<'a>(&'a self) -> &'a Horn {
            if self.talks.len() == 1 {
                self.talks[0].borrow().horn()
            } else {
                &self.horn.as_ref().unwrap()
            }
        }
    */
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
        let first_talk = self.talks.first()?;
        return first_talk.borrow().value();
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
    pub fn cv_buffer(&self) -> Option<CvBuf> {
        if self.talks.len() > 1 {
            self.horn.as_ref().unwrap().cv_buffer()
        } else if self.talks.len() == 1 {
            self.talks[0].borrow().cv_buffer()
        } else {
            None
        }
    }
    pub fn talks<'a>(&'a self) -> &'a Vec<RTalk> {
        &self.talks
    }
    pub fn clone(&self) -> Hum {
        Hum {
            horn: None,
            port_type: self.port_type,
            tag: self.tag.to_string(),
            min_value: self.min_value,
            max_value: self.max_value,
            def_value: self.def_value,
            talks: Vec::new(),
        }
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
    pub fn add_talk_value(&mut self, value: f32) -> Result<(), failure::Error> {
        self.add_talk(def_talk(self.port_type, value))
    }
    pub fn add_talk_voice(&mut self, talker: &RTalker, port: Index) -> Result<(), failure::Error> {
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
                    // let c = 1. / f32::try_from(u16::try_from(self.talks.len())).unwrap();
                    for i in 0..ln {
                        let v = out_buf.get()[i].get() * c;
                        out_buf.get()[i].set(v);
                    }
                }
                Horn::Control(_) => {}
                Horn::Cv(_) => {}
            }
        }
        ln
    }
}

pub type RHum = RefCell<Hum>;

pub struct Set {
    hums: Vec<RHum>,
}

impl Set {
    pub fn new(hums: Vec<RHum>) -> Set {
        Self { hums }
    }

    pub fn hums<'a>(&'a self) -> &'a Vec<RHum> {
        &self.hums
    }

    pub fn clone(&self) -> Set {
        let mut hums: Vec<RHum> = Vec::new();

        for hum in &self.hums {
            hums.push(RefCell::new(hum.borrow().clone()));
        }
        Set { hums }
    }
}

pub struct Ear {
    tag: String,
    multi_set: bool,
    sets: RefCell<Vec<Set>>,
}
//pub type REar = RefCell<Ear>;

impl Ear {
    pub fn new(tag: Option<&str>, multi_set: bool, hum: RHum) -> Ear {
        Self {
            tag: tag.unwrap_or(DEF_INPUT_TAG).to_string(),
            multi_set,
            sets: RefCell::new(vec![Set::new(vec![hum])]),
        }
    }
    pub fn tag<'a>(&'a self) -> &'a String {
        &self.tag
    }
    pub fn sets<'a>(&'a self) -> &'a RefCell<Vec<Set>> {
        &self.sets
    }
    pub fn is_multi_set(&self) -> bool {
        self.multi_set
    }
    pub fn sets_len(&self) -> usize {
        self.sets.borrow().len()
    }

    pub fn talk_range(&self, hum_idx: usize) -> (f32, f32, f32) {
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

    pub fn talk_value_or_default(&self, hum_idx: usize) -> f32 {
        if self.sets.borrow().len() > 0 && self.sets.borrow()[0].hums.len() > hum_idx {
            return self.sets.borrow()[0].hums[hum_idx]
                .borrow()
                .value_or_default();
        }
        return 0.;
    }

    pub fn get_set_hum_audio_buffer(&self, set_idx: Index, hum_idx: Index) -> Option<AudioBuf> {
        self.sets.borrow()[set_idx].hums[hum_idx]
            .borrow()
            .audio_buffer()
    }
    pub fn get_set_audio_buffer(&self, set_idx: Index) -> Option<AudioBuf> {
        self.get_set_hum_audio_buffer(set_idx, 0)
    }
    pub fn get_audio_buffer(&self) -> Option<AudioBuf> {
        self.get_set_audio_buffer(0)
    }

    pub fn get_set_hum_cv_buffer(&self, set_idx: Index, hum_idx: Index) -> Option<CvBuf> {
        self.sets.borrow()[set_idx].hums[hum_idx]
            .borrow()
            .cv_buffer()
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
    /*
        pub fn fold_talkers<F, P>(&self, mut f: F, p: P) -> Result<P, failure::Error>
        where
            F: FnMut(&RTalker, P) -> Result<P, failure::Error>,
        {
            self.fold_talks(|tlk, p| f(&tlk.tkr, p), p)
        }
    */
    pub fn visit_horn<F>(&self, f: F)
    where
        F: FnMut(&Horn),
    {
        if self.sets.borrow().len() == 1 && self.sets.borrow()[0].hums.len() == 1 {
            self.sets.borrow()[0].hums[0].borrow().visit_horn(f);
        }
    }

    fn provide_set(&self, set_idx: Index) {
        for _ in self.sets.borrow().len()..(set_idx + 1) {
            let new_set = self.sets.borrow()[0].clone();
            self.sets.borrow_mut().push(new_set)
        }
    }
    fn add_set(&self) -> Index {
        let set_idx = self.sets.borrow().len();
        self.provide_set(set_idx);
        set_idx
    }
    pub fn sup_set(&self, set_idx: Index) -> Result<(), failure::Error> {
        self.sets.borrow_mut().remove(set_idx);
        Ok(())
    }

    pub fn set_talk_value_by_tag(
        &self,
        set_idx: Index,
        hum_tag: &str,
        value: f32,
    ) -> Result<(), failure::Error> {
        self.provide_set(set_idx);

        for hum in &self.sets.borrow()[set_idx].hums {
            if hum.borrow().tag() == hum_tag {
                return hum.borrow_mut().add_talk_value(value);
            }
        }
        Err(failure::err_msg(format!(
            "Ear {} hum {} not found!",
            self.tag(),
            hum_tag
        )))
    }
    pub fn set_talk_voice_by_tag(
        &self,
        set_idx: Index,
        hum_tag: &str,
        talker: &RTalker,
        port: usize,
    ) -> Result<(), failure::Error> {
        self.provide_set(set_idx);

        for hum in &self.sets.borrow()[set_idx].hums {
            if hum.borrow().tag() == hum_tag {
                return hum.borrow_mut().add_talk_voice(talker, port);
            }
        }
        Err(failure::err_msg(format!(
            "Ear {} hum {} not found!",
            self.tag(),
            hum_tag
        )))
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
    pub fn add_talk_value(
        &self,
        set_idx: Index,
        hum_idx: Index,
        value: f32,
    ) -> Result<(), failure::Error> {
        self.sets.borrow()[set_idx].hums[hum_idx]
            .borrow_mut()
            .add_talk_value(value)
    }

    pub fn add_talk_voice(
        &self,
        set_idx: Index,
        hum_idx: Index,
        voice_talker: &RTalker,
        port: usize,
    ) -> Result<(), failure::Error> {
        self.sets.borrow()[set_idx].hums[hum_idx]
            .borrow_mut()
            .add_talk_voice(voice_talker, port)
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

    pub fn add_value(&self, hum_idx: Index, value: f32) -> Result<(), failure::Error> {
        let set_idx = self.add_set();
        self.sets.borrow()[set_idx].hums[hum_idx]
            .borrow_mut()
            .add_talk_value(value)
    }

    pub fn add_voice(
        &self,
        hum_idx: Index,
        voice_talker: &RTalker,
        port: usize,
    ) -> Result<(), failure::Error> {
        let set_idx = self.add_set();
        self.sets.borrow()[set_idx].hums[hum_idx]
            .borrow_mut()
            .add_talk_voice(voice_talker, port)
    }
}

pub fn audio(
    tag: Option<&str>,
    min_value: f32,
    max_value: f32,
    def_value: f32,
    talker_port: Option<(&RTalker, Index)>,
) -> Ear {
    match talker_port {
        Some((tkr, port)) => Ear::new(
            tag,
            false,
            Hum::from_voice(tag, min_value, max_value, tkr, port),
        ),
        None => Ear::new(
            tag,
            false,
            Hum::from_value(tag, PortType::Audio, min_value, max_value, def_value),
        ),
    }
}

pub fn control(tag: Option<&str>, min_value: f32, max_value: f32, def_value: f32) -> Ear {
    Ear::new(
        tag,
        false,
        Hum::from_value(tag, PortType::Control, min_value, max_value, def_value),
    )
}

pub fn cv(
    tag: Option<&str>,
    min_value: f32,
    max_value: f32,
    def_value: f32,
    talker_port: Option<(&RTalker, Index)>,
) -> Ear {
    match talker_port {
        Some((tkr, port)) => Ear::new(
            tag,
            false,
            Hum::from_voice(tag, min_value, max_value, tkr, port),
        ),
        None => Ear::new(
            tag,
            false,
            Hum::from_value(tag, PortType::Cv, min_value, max_value, def_value),
        ),
    }
}

pub fn def_ear() -> Ear {
    control(None, f32::MIN, f32::MAX, f32::NAN)
}

pub fn multi(tag: Option<&str>, port_type: PortType) -> Ear {
    Ear::new(
        tag,
        true,
        Hum::new(tag, port_type, f32::MIN, f32::MAX, f32::NAN),
    )
}

pub fn controls(tag: Option<&str>) -> Ear {
    multi(tag, PortType::Control)
}

pub fn audios(tag: Option<&str>) -> Ear {
    multi(tag, PortType::Audio)
}

pub fn cvs(tag: Option<&str>) -> Ear {
    multi(tag, PortType::Cv)
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
