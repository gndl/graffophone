use crate::audio_talker::AudioTalker;
use crate::control_talker::ControlTalker;
use crate::cv_talker::CvTalker;
use crate::horn::{AudioBuf, CvBuf, Horn};
use crate::talker::RTalker;
use std::cell::RefCell;
use std::rc::Rc;
use voice::PortType;

pub const DEF_INPUT_TAG: &'static str = "I";

pub struct Talk {
    port_type: PortType,
    tag: String,
    tkr: RTalker,
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
    pub fn add_talk_value(&mut self, value: f32) -> Result<(), failure::Error> {
        self.talks.push(new_talk_value(&self.port_type, value));
        Ok(())
    }
    pub fn add_talk_voice(&mut self, talker: &RTalker, port: usize) -> Result<(), failure::Error> {
        if talker.borrow().voice_port_type_is(port, self.port_type) {
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
}

pub type MTalks = RefCell<Talks>;

pub enum Ear {
    Talk(MTalk),
    Talks(MTalks),
}

impl Ear {
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

    pub fn talks<'a>(&'a self) -> Option<&'a MTalks> {
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

pub fn def_audio_talker(value: Option<f32>) -> RTalker {
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
pub fn def_control_talker(value: Option<f32>) -> RTalker {
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
pub fn def_cv_talker(value: Option<f32>) -> RTalker {
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
    talker_port: Option<(&RTalker, usize)>,
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
pub fn cv(tag: Option<String>, value: Option<f32>, talker_port: Option<(&RTalker, usize)>) -> Ear {
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

pub fn def_talks(tag: Option<String>, port_type: PortType) -> MTalks {
    RefCell::new(Talks {
        port_type,
        tag: tag.unwrap_or(DEF_INPUT_TAG.to_string()),
        talks: Vec::new(),
    })
}

pub fn talks(tag: Option<String>, port_type: PortType) -> Ear {
    Ear::Talks(def_talks(tag, port_type))
}

pub fn controls(tag: Option<String>) -> Ear {
    talks(tag, PortType::Control)
}

pub fn audios(tag: Option<String>) -> Ear {
    talks(tag, PortType::Audio)
}

pub fn cvs(tag: Option<String>) -> Ear {
    talks(tag, PortType::Cv)
}

pub fn set_talk_value(talk: &MTalk, value: f32) -> Result<(), failure::Error> {
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
    Ok(())
}
pub fn set_talk_voice(talk: &MTalk, talker: &RTalker, port: usize) -> Result<(), failure::Error> {
    if talker
        .borrow()
        .voice_port_type_is(port, talk.borrow().port_type())
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

pub fn new_talk_value(port_type: &PortType, value: f32) -> MTalk {
    match port_type {
        PortType::Audio => def_audio_talk(None, Some(value)),
        PortType::Control => def_control_talk(None, Some(value)),
        PortType::Cv => def_cv_talk(None, Some(value)),
    }
}

pub fn new_talk_voice(talker: &RTalker, port: usize) -> MTalk {
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
    index: usize,
    mut f: F,
) -> Result<(), failure::Error>
where
    F: FnMut(&MTalk) -> Result<(), failure::Error>,
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
