use crate::audio_talker::AudioTalker;
use crate::control_talker::ControlTalker;
use crate::cv_talker::CvTalker;
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
    pub fn port_type<'a>(&'a self) -> &'a PortType {
        &self.port_type
    }
    pub fn tag<'a>(&'a self) -> &'a String {
        &self.tag
    }
}
pub type MTalk = RefCell<Talk>;

pub struct Talks {
    port_type: PortType,
    tag: String,
    talks: Vec<MTalk>,
}

impl Talks {
    pub fn port_type<'a>(&'a self) -> &'a PortType {
        &self.port_type
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
    pub fn add_talk_value(&mut self, value: f32) {
        self.talks.push(new_talk_type(&self.port_type, value))
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

pub fn set_talk_value(talk: &MTalk, value: f32) {
    /*    if talk.tkr.is_hidden() {
         talk.tkr.set_value(Float value)
       }else{
         talk.voice <- ((new hiddenConstant ~value ())#getVoice "")
       }}
    */
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
}

pub fn new_talk_type(port_type: &PortType, value: f32) -> MTalk {
    match port_type {
        PortType::Audio => def_audio_talk(None, Some(value)),
        PortType::Control => def_control_talk(None, Some(value)),
        PortType::Cv => def_cv_talk(None, Some(value)),
    }
}

pub fn listen_talk(talk: &Talk, tick: i64, len: usize) -> usize {
    //    let tlk = talk.borrow();

    let port = talk.port;
    {
        let mut tkr = talk.tkr.borrow_mut();
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
