use crate::audio_talker::AudioTalker;
use crate::control_talker::ControlTalker;
use crate::cv_talker::CvTalker;
use crate::horn::{AudioBuf, Horn};
use crate::talker::MTalker;
use std::cell::RefCell;
use std::rc::Rc;
use voice::PortType;

pub const DEF_INPUT_TAG: &'static str = "I";

pub struct Talk {
    port_type: PortType,
    tag: String,
    tkr: MTalker,
    port: usize,
}

impl Talk {
    pub fn port_type(&self) -> PortType {
        self.port_type
    }
    pub fn tag<'a>(&'a self) -> &'a String {
        &self.tag
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
}

pub type MTalk = RefCell<Talk>;

pub struct Talks {
    port_type: PortType,
    tag: String,
    talks: Vec<MTalk>,
}

impl Talks {
    pub fn port_type(&self) -> PortType {
        self.port_type
    }
    pub fn tag<'a>(&'a self) -> &'a String {
        &self.tag
    }
    pub fn talks<'a>(&'a self) -> &'a Vec<MTalk> {
        &self.talks
    }
    pub fn add_talk(&mut self, talk: MTalk) {
        self.talks.push(talk)
    }
    pub fn add_talk_value(&mut self, value: f32) -> bool {
        self.talks.push(new_talk_value(&self.port_type, value));
        true
    }
    pub fn add_talk_voice(&mut self, talker: &MTalker, port: usize) -> bool {
        if talker.borrow().voice_port_type_is(port, self.port_type) {
            self.talks.push(new_talk_voice(talker, port));
            return true;
        }
        return false;
    }
}

pub type MTalks = RefCell<Talks>;

pub enum Ear {
    Talk(MTalk),
    Talks(MTalks),
}

pub fn def_audio_talker(value: Option<f32>) -> MTalker {
    Rc::new(RefCell::new(AudioTalker::new(value, Some(true))))
}
pub fn def_audio_talk(tag: Option<String>, value: Option<f32>) -> MTalk {
    RefCell::new(Talk {
        port_type: PortType::Audio,
        tag: tag.unwrap_or(DEF_INPUT_TAG.to_string()),
        tkr: def_audio_talker(value),
        port: 0,
    })
}
pub fn def_control_talker(value: Option<f32>) -> MTalker {
    Rc::new(RefCell::new(ControlTalker::new(value, Some(true))))
}
pub fn def_control_talk(tag: Option<String>, value: Option<f32>) -> MTalk {
    RefCell::new(Talk {
        port_type: PortType::Control,
        tag: tag.unwrap_or(DEF_INPUT_TAG.to_string()),
        tkr: def_control_talker(value),
        port: 0,
    })
}
pub fn def_cv_talker(value: Option<f32>) -> MTalker {
    Rc::new(RefCell::new(CvTalker::new(value, Some(true))))
}
pub fn def_cv_talk(tag: Option<String>, value: Option<f32>) -> MTalk {
    RefCell::new(Talk {
        port_type: PortType::Cv,
        tag: tag.unwrap_or(DEF_INPUT_TAG.to_string()),
        tkr: def_cv_talker(value),
        port: 0,
    })
}

pub fn def_ear() -> Ear {
    Ear::Talk(def_control_talk(None, None))
}

pub fn control(tag: Option<String>, value: Option<f32>) -> Ear {
    Ear::Talk(def_control_talk(tag, value))
}

pub fn audio(
    tag: Option<String>,
    value: Option<f32>,
    talker_port: Option<(&MTalker, usize)>,
) -> Ear {
    match value {
        Some(_v) => Ear::Talk(def_audio_talk(tag, value)),
        None => match talker_port {
            Some((tkr, port)) => Ear::Talk(RefCell::new(Talk {
                port_type: PortType::Audio,
                tag: tag.unwrap_or(DEF_INPUT_TAG.to_string()),
                tkr: Rc::clone(tkr),
                port: port,
            })),
            None => Ear::Talk(def_audio_talk(tag, None)),
        },
    }
}
pub fn cv(tag: Option<String>, value: Option<f32>, talker_port: Option<(&MTalker, usize)>) -> Ear {
    match value {
        Some(_v) => Ear::Talk(def_cv_talk(tag, value)),
        None => match talker_port {
            Some((tkr, port)) => Ear::Talk(RefCell::new(Talk {
                port_type: PortType::Cv,
                tag: tag.unwrap_or(DEF_INPUT_TAG.to_string()),
                tkr: Rc::clone(tkr),
                port: port,
            })),
            None => Ear::Talk(def_cv_talk(tag, None)),
        },
    }
}

pub fn set_talk_value(talk: &MTalk, value: f32) -> bool {
    let mut tlk = talk.borrow_mut();
    match tlk.port_type {
        PortType::Audio => {
            tlk.tkr = def_audio_talker(Some(value));
            tlk.port = 0;
        }
        PortType::Control => {
            tlk.tkr = def_control_talker(Some(value));
            tlk.port = 0;
        }
        PortType::Cv => {
            tlk.tkr = def_cv_talker(Some(value));
            tlk.port = 0;
        }
    }
    true
}
pub fn set_talk_voice(talk: &MTalk, talker: &MTalker, port: usize) -> bool {
    if talker
        .borrow()
        .voice_port_type_is(port, talk.borrow().port_type())
    {
        let mut tlk = talk.borrow_mut();
        tlk.tkr = talker.clone();
        tlk.port = port;
        return true;
    }
    false
}

pub fn new_talk_value(port_type: &PortType, value: f32) -> MTalk {
    match port_type {
        PortType::Audio => def_audio_talk(None, Some(value)),
        PortType::Control => def_control_talk(None, Some(value)),
        PortType::Cv => def_cv_talk(None, Some(value)),
    }
}

pub fn new_talk_voice(talker: &MTalker, port: usize) -> MTalk {
    let port_type;
    {
        port_type = talker.borrow().voice_port_type(port);
    }
    RefCell::new(Talk {
        port_type,
        tag: DEF_INPUT_TAG.to_string(),
        tkr: talker.clone(),
        port,
    })
}

pub fn audio_buffer(ear: &Ear) -> Option<AudioBuf> {
    match ear {
        Ear::Talk(talk) => talk.borrow().audio_buffer(),
        _ => None,
    }
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

pub fn listen(ear: &Ear, tick: i64, len: usize) -> usize {
    match ear {
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

pub fn visit_ear_flatten_index<F>(ears: &Vec<Ear>, index: usize, mut f: F) -> bool
where
    F: FnMut(&MTalk) -> bool,
{
    let mut res = false;
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

pub fn visit_horn<F>(ear: &Ear, f: F)
where
    F: FnMut(&Horn),
{
    match ear {
        Ear::Talk(talk) => visit_talk_horn(&talk.borrow(), f),
        Ear::Talks(_talks) => (),
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
