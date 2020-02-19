use crate::horn::{AudioBuf, CvBuf};
extern crate failure;
use crate::data::Data;
use crate::ear;
use crate::ear::Ear;
use crate::identifier::{Identifier, RIdentifier};
use crate::voice::MVoice;
use crate::voice::PortType;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicU32, Ordering};

static TALKER_COUNT: AtomicU32 = AtomicU32::new(1);

pub struct TalkerBase {
    identifier: RIdentifier,
    ears: Vec<Ear>,
    voices: Vec<MVoice>,
    //    ear_call: bool,
    hidden: bool,
}

impl TalkerBase {
    pub fn new() -> Self {
        Self {
            identifier: RefCell::new(Identifier::new(
                "",
                "",
                TALKER_COUNT.fetch_add(1, Ordering::SeqCst),
            )),
            ears: Vec::new(),
            voices: Vec::new(),
            //            ear_call: false,
            hidden: false,
        }
    }
    pub fn add_ear<'a>(&'a mut self, ear: Ear) {
        self.ears.push(ear);
    }
    pub fn add_voice<'a>(&'a mut self, voice: MVoice) {
        self.voices.push(voice);
    }
    pub fn identifier<'a>(&'a self) -> &'a RIdentifier {
        &self.identifier
    }
    pub fn id(&self) -> u32 {
        self.identifier.borrow().id()
    }
    pub fn name(&self) -> String {
        self.identifier.borrow().name().to_string()
    }
    pub fn set_name(&self, name: &String) {
        self.identifier.borrow_mut().set_name(name);
    }
    pub fn is_hidden(&self) -> bool {
        self.hidden
    }
    pub fn set_hidden(&mut self, hidden: bool) {
        self.hidden = hidden;
    }
}

pub trait Talker {
    fn base<'a>(&'a self) -> &'a TalkerBase;
    fn id(&self) -> u32 {
        self.base().id()
    }
    fn model(&self) -> &str {
        ""
    }
    fn name(&self) -> String {
        self.base().name()
    }
    fn set_name(&self, name: &String) {
        self.base().set_name(name);
    }
    fn is_hidden(&self) -> bool {
        self.base().is_hidden()
    }
    fn depends_of(&self, id: u32) -> bool {
        self.base().id() == id
    }

    fn data(&self) -> Data {
        Data::Nil
    }
    fn set_data(&mut self, data: Data) -> Result<(), failure::Error> {
        Err(data.notify_incompatibility("Nil"))
    }
    fn get_data_string(&self) -> String {
        self.data().to_string()
    }
    fn set_data_from_string(&mut self, s: &str) -> Result<(), failure::Error> {
        match self.data().birth(s) {
            Ok(d) => self.set_data(d),
            Err(e) => Err(e),
        }
    }
    fn get_float_data(&self) -> Result<f32, failure::Error> {
        self.data().to_f()
    }

    fn ears<'a>(&'a self) -> &'a Vec<Ear> {
        &self.base().ears
    }
    fn voices<'a>(&'a self) -> &'a Vec<MVoice> {
        &self.base().voices
    }
    fn voice<'a>(&'a self, port: usize) -> &'a MVoice {
        &self.voices().get(port).unwrap()
    }
    fn voice_port_type(&self, port: usize) -> PortType {
        self.voice(port).borrow().port_type()
    }
    fn voice_port_type_is(&self, port: usize, port_type: PortType) -> bool {
        self.voice_port_type(port) == port_type
    }
    fn voice_port(&self, tag: &str) -> Result<usize, failure::Error> {
        for (port, voice) in self.voices().iter().enumerate() {
            if voice.borrow().tag() == tag {
                return Ok(port);
            }
        }
        Err(failure::err_msg(format!(
            "Unknow voice {} for talker {}",
            tag,
            self.name()
        )))
    }

    fn ear_audio_buffer(&self, port: usize) -> Option<AudioBuf> {
        let ear = self.ears().get(port)?;
        ear.audio_buffer()
    }

    fn ear_cv_buffer(&self, port: usize) -> Option<CvBuf> {
        let ear = self.ears().get(port)?;
        ear.cv_buffer()
    }
    /*
        fn ear_talks(&self, port: usize) -> Option<MTalks> {
            let ear = self.ears().get(port)?;
            ear.talks()
        }
    */
    fn set_ear_value_by_tag(&mut self, tag: &str, value: f32) -> bool {
        for ear in self.ears() {
            match ear {
                Ear::Talk(talk) => {
                    if talk.borrow().tag() == tag {
                        return ear::set_talk_value(talk, value);
                    }
                }
                Ear::Talks(talks) => {
                    let mut tlks = talks.borrow_mut();

                    if tlks.tag() == tag {
                        return tlks.add_talk_value(value);
                    } else {
                        for talk in tlks.talks() {
                            if talk.borrow().tag() == tag {
                                return ear::set_talk_value(talk, value);
                            }
                        }
                    }
                }
            }
        }
        false
    }
    fn set_ear_voice_by_tag(&mut self, tag: &str, talker: &RTalker, port: usize) -> bool {
        for ear in self.ears() {
            match ear {
                Ear::Talk(talk) => {
                    if talk.borrow().tag() == tag {
                        return ear::set_talk_voice(&talk, talker, port);
                    }
                }
                Ear::Talks(talks) => {
                    let mut tlks = talks.borrow_mut();
                    if tlks.tag() == tag {
                        return tlks.add_talk_voice(talker, port);
                    } else {
                        for talk in tlks.talks() {
                            if talk.borrow().tag() == tag {
                                return ear::set_talk_voice(&talk, talker, port);
                            }
                        }
                    }
                }
            }
        }
        false
    }

    fn set_ear_value_by_index(&self, index: usize, value: f32) -> bool {
        ear::visit_ear_flatten_index(self.ears(), index, |talk| ear::set_talk_value(talk, value))
    }

    fn set_ear_voice_by_index(&self, index: usize, talker: &RTalker, port: usize) -> bool {
        ear::visit_ear_flatten_index(self.ears(), index, |talk| {
            ear::set_talk_voice(talk, talker, port)
        })
    }

    fn activate(&mut self) {}
    fn deactivate(&mut self) {}

    fn talk(&mut self, _port: usize, _tick: i64, _len: usize) -> usize {
        0
    }

    fn listen_ears(&self, tick: i64, len: usize) -> usize {
        let mut ln = len;
        for ear in self.ears() {
            ln = ear.listen(tick, ln);
        }
        ln
    }

    fn backup<'a>(&'a self) -> (&str, std::string::String, &std::vec::Vec<ear::Ear>) {
        (self.model(), self.get_data_string(), self.ears())
    }
}

pub type RTalker = Rc<RefCell<dyn Talker>>;
