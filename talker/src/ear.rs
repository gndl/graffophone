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

pub struct Talk {
    port_type: PortType,
    tag: String,
    tkr: RTalker,
    port: Index,
    min_value: f32,
    max_value: f32,
    def_value: f32,
}

impl Talk {
    pub fn port_type(&self) -> PortType {
        self.port_type
    }
    pub fn tag<'a>(&'a self) -> &'a String {
        &self.tag
    }
    pub fn talker<'a>(&'a self) -> &'a RTalker {
        &self.tkr
    }
    pub fn port(&self) -> Index {
        self.port
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
        self.tkr.borrow().voice_value(self.port)
    }
    pub fn value_or_default(&self) -> f32 {
        self.tkr
            .borrow()
            .voice_value(self.port)
            .unwrap_or(self.def_value)
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
    pub fn cv_buffer(&self) -> Option<CvBuf> {
        let res;
        let tkr = self.tkr.borrow();
        {
            let voice = tkr.voices().get(self.port)?;
            res = voice.borrow().cv_buffer();
        }
        res
    }
}

pub type RTalk = RefCell<Talk>;

pub struct Talks {
    port_type: PortType,
    tag: String,
    talks: Vec<RTalk>,
}

impl Talks {
    pub fn port_type(&self) -> PortType {
        self.port_type
    }
    pub fn tag<'a>(&'a self) -> &'a String {
        &self.tag
    }
    pub fn talks<'a>(&'a self) -> &'a Vec<RTalk> {
        &self.talks
    }
    pub fn add_talk(&mut self, talk: RTalk) {
        self.talks.push(talk)
    }
    pub fn add_talk_value(&mut self, value: f32) -> Result<(), failure::Error> {
        self.talks.push(new_talk_value(&self.port_type, value));
        Ok(())
    }
    pub fn add_talk_voice(&mut self, talker: &RTalker, port: Index) -> Result<(), failure::Error> {
        if self
            .port_type()
            .can_hear(talker.borrow().voice_port_type(port))
        {
            self.talks.push(new_talk_voice(talker, port));
            Ok(())
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
        Ok(())
    }
}

pub type RTalks = RefCell<Talks>;

pub struct Group {
    tag: String,
    talks: Vec<RTalks>,
}
pub type RGroup = RefCell<Group>;

pub struct Ear {
    tag: String,
    Groups: Vec<RGroup>,
}

impl Ear {
    pub fn is_multi_talk(&self) -> bool {
        match self {
            Ear::Talks(_) => true,
            _ => false,
        }
    }

    pub fn talk_range(&self, talk_idx: usize) -> (f32, f32, f32) {
        match self {
            Ear::Talk(talk) => talk.borrow().range(),
            Ear::Talks(talks) => talks.borrow().talks()[talk_idx].borrow().range(),
        }
    }

    pub fn talk_def_value(&self, talk_idx: usize) -> f32 {
        match self {
            Ear::Talk(talk) => talk.borrow().def_value(),
            Ear::Talks(talks) => talks.borrow().talks()[talk_idx].borrow().def_value(),
        }
    }

    pub fn talk_value_or_default(&self, talk_idx: usize) -> f32 {
        match self {
            Ear::Talk(talk) => talk.borrow().value_or_default(),
            Ear::Talks(talks) => talks.borrow().talks()[talk_idx].borrow().value_or_default(),
        }
    }

    pub fn audio_buffer(&self) -> Option<AudioBuf> {
        match self {
            Ear::Talk(talk) => talk.borrow().audio_buffer(),
            _ => None,
        }
    }

    pub fn cv_buffer(&self) -> Option<CvBuf> {
        match self {
            Ear::Talk(talk) => talk.borrow().cv_buffer(),
            _ => None,
        }
    }

    pub fn talks<'a>(&'a self) -> Option<&'a RTalks> {
        match self {
            Ear::Talks(talks) => Some(&talks),
            _ => None,
        }
    }

    pub fn listen(&self, tick: i64, len: usize) -> usize {
        match self {
            Ear::Talk(talk) => listen_talk(&talk.borrow(), tick, len),
            Ear::Talks(talks) => {
                let mut ln = len;

                for talk in &talks.borrow().talks {
                    ln = listen_talk(&talk.borrow(), tick, ln);
                }
                ln
            }
        }
    }

    pub fn iter_talks<F, P>(&self, mut f: F, p: &mut P) -> Result<(), failure::Error>
    where
        F: FnMut(&Talk, &mut P) -> Result<(), failure::Error>,
    {
        match self {
            Ear::Talk(talk) => f(&talk.borrow(), p),
            Ear::Talks(talks) => {
                for talk in &talks.borrow().talks {
                    f(&talk.borrow(), p)?;
                }
                Ok(())
            }
        }
    }

    pub fn fold_talks<F, P>(&self, mut f: F, p: P) -> Result<P, failure::Error>
    where
        F: FnMut(&Talk, P) -> Result<P, failure::Error>,
    {
        match self {
            Ear::Talk(talk) => f(&talk.borrow(), p),
            Ear::Talks(talks) => {
                let mut acc = p;
                for talk in &talks.borrow().talks {
                    acc = f(&talk.borrow(), acc)?;
                }
                Ok(acc)
            }
        }
    }

    pub fn iter_talkers<F, P>(&self, mut f: F, p: &mut P) -> Result<(), failure::Error>
    where
        F: FnMut(&RTalker, &mut P) -> Result<(), failure::Error>,
    {
        self.iter_talks(|tlk, p| f(&tlk.tkr, p), p)
    }

    pub fn fold_talkers<F, P>(&self, mut f: F, p: P) -> Result<P, failure::Error>
    where
        F: FnMut(&RTalker, P) -> Result<P, failure::Error>,
    {
        self.fold_talks(|tlk, p| f(&tlk.tkr, p), p)
    }

    pub fn visit_horn<F>(&self, f: F)
    where
        F: FnMut(&Horn),
    {
        match self {
            Ear::Talk(talk) => visit_talk_horn(&talk.borrow(), f),
            Ear::Talks(_talks) => (),
        }
    }
}

pub fn def_audio_talker(def_value: f32) -> RTalker {
    Rc::new(RefCell::new(AudioTalker::new(def_value, Some(true))))
}
pub fn def_audio_talk(tag: Option<&str>, min_value: f32, max_value: f32, def_value: f32) -> RTalk {
    RefCell::new(Talk {
        port_type: PortType::Audio,
        tag: tag.unwrap_or(DEF_INPUT_TAG).to_string(),
        tkr: def_audio_talker(def_value),
        port: 0,
        min_value,
        max_value,
        def_value,
    })
}
pub fn def_control_talker(def_value: f32) -> RTalker {
    Rc::new(RefCell::new(ControlTalker::new(def_value, Some(true))))
}
pub fn def_control_talk(
    tag: Option<&str>,
    min_value: f32,
    max_value: f32,
    def_value: f32,
) -> RTalk {
    RefCell::new(Talk {
        port_type: PortType::Control,
        tag: tag.unwrap_or(DEF_INPUT_TAG).to_string(),
        tkr: def_control_talker(def_value),
        port: 0,
        min_value,
        max_value,
        def_value,
    })
}
pub fn def_cv_talker(def_value: f32) -> RTalker {
    Rc::new(RefCell::new(CvTalker::new(def_value, Some(true))))
}
pub fn def_cv_talk(tag: Option<&str>, min_value: f32, max_value: f32, def_value: f32) -> RTalk {
    RefCell::new(Talk {
        port_type: PortType::Cv,
        tag: tag.unwrap_or(DEF_INPUT_TAG).to_string(),
        tkr: def_cv_talker(def_value),
        port: 0,
        min_value,
        max_value,
        def_value,
    })
}

pub fn def_ear() -> Ear {
    Ear::Talk(def_control_talk(None, f32::MIN, f32::MAX, f32::NAN))
}

pub fn control(tag: Option<&str>, min_value: f32, max_value: f32, def_value: f32) -> Ear {
    Ear::Talk(def_control_talk(tag, min_value, max_value, def_value))
}

pub fn audio(
    tag: Option<&str>,
    min_value: f32,
    max_value: f32,
    def_value: f32,
    talker_port: Option<(&RTalker, Index)>,
) -> Ear {
    match talker_port {
        Some((tkr, port)) => Ear::Talk(RefCell::new(Talk {
            port_type: PortType::Audio,
            tag: tag.unwrap_or(DEF_INPUT_TAG).to_string(),
            tkr: Rc::clone(tkr),
            port: port,
            min_value,
            max_value,
            def_value,
        })),
        None => Ear::Talk(def_audio_talk(tag, min_value, max_value, def_value)),
    }
}
pub fn cv(
    tag: Option<&str>,
    min_value: f32,
    max_value: f32,
    def_value: f32,
    talker_port: Option<(&RTalker, Index)>,
) -> Ear {
    match talker_port {
        Some((tkr, port)) => Ear::Talk(RefCell::new(Talk {
            port_type: PortType::Cv,
            tag: tag.unwrap_or(DEF_INPUT_TAG).to_string(),
            tkr: Rc::clone(tkr),
            port: port,
            min_value,
            max_value,
            def_value,
        })),
        None => Ear::Talk(def_cv_talk(tag, min_value, max_value, def_value)),
    }
}

pub fn def_talks(tag: Option<&str>, port_type: PortType) -> RTalks {
    RefCell::new(Talks {
        port_type,
        tag: tag.unwrap_or(DEF_INPUT_TAG).to_string(),
        talks: Vec::new(),
    })
}

pub fn talks(tag: Option<&str>, port_type: PortType) -> Ear {
    Ear::Talks(def_talks(tag, port_type))
}

pub fn controls(tag: Option<&str>) -> Ear {
    talks(tag, PortType::Control)
}

pub fn audios(tag: Option<&str>) -> Ear {
    talks(tag, PortType::Audio)
}

pub fn cvs(tag: Option<&str>) -> Ear {
    talks(tag, PortType::Cv)
}

pub fn set_talk_value(talk: &RTalk, value: f32) -> Result<(), failure::Error> {
    let mut tlk = talk.borrow_mut();
    match tlk.port_type {
        PortType::Audio => {
            tlk.tkr = def_audio_talker(value);
            tlk.port = 0;
        }
        PortType::Control => {
            tlk.tkr = def_control_talker(value);
            tlk.port = 0;
        }
        PortType::Cv => {
            tlk.tkr = def_cv_talker(value);
            tlk.port = 0;
        }
    }
    Ok(())
}
pub fn set_talk_voice(talk: &RTalk, talker: &RTalker, port: Index) -> Result<(), failure::Error> {
    if talk
        .borrow()
        .port_type()
        .can_hear(talker.borrow().voice_port_type(port))
    {
        let mut tlk = talk.borrow_mut();
        tlk.tkr = talker.clone();
        tlk.port = port;
        Ok(())
    } else {
        Err(failure::err_msg(format!(
            "Talker {} voice {} type is not compatible with talker {} talk {}!",
            talker.borrow().name(),
            port,
            talk.borrow().tkr.borrow().name(),
            talk.borrow().tag
        )))
    }
}

pub fn new_talk_value(port_type: &PortType, value: f32) -> RTalk {
    match port_type {
        PortType::Audio => def_audio_talk(None, f32::MIN, f32::MAX, value),
        PortType::Control => def_control_talk(None, f32::MIN, f32::MAX, value),
        PortType::Cv => def_cv_talk(None, f32::MIN, f32::MAX, value),
    }
}

pub fn new_talk_voice(talker: &RTalker, port: Index) -> RTalk {
    let port_type;
    {
        port_type = talker.borrow().voice_port_type(port);
    }
    RefCell::new(Talk {
        port_type,
        tag: DEF_INPUT_TAG.to_string(),
        tkr: talker.clone(),
        port,
        min_value: f32::MIN,
        max_value: f32::MAX,
        def_value: 0.,
    })
}

pub fn listen_talk(talk: &Talk, tick: i64, len: usize) -> usize {
    let port = talk.port;
    {
        let tkr = talk.tkr.borrow();
        let voice = tkr.voices().get(port).unwrap().borrow();

        if tick == voice.tick() {
            return voice.len();
        }
    }

    talk.tkr.borrow_mut().talk(port, tick, len)
}

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

pub fn visit_talk_horn<F>(talk: &Talk, mut f: F)
where
    F: FnMut(&Horn),
{
    let tkr = talk.tkr.borrow();
    {
        match tkr.voices().get(talk.port) {
            Some(voice) => f(voice.borrow().horn()),
            None => (),
        }
    }
}

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
