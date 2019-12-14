extern crate failure;
use crate::ear;
use crate::ear::Ear;
use crate::identifier::Identifier;
use crate::voice::MVoice;
use crate::voice::PortType;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicU32, Ordering};

static TALKER_COUNT: AtomicU32 = AtomicU32::new(1);

pub enum ValueType {
    Nil,
    Int(i64),
    Float(f32),
    String(String),
    Text(String),
    File(String),
}

pub struct TalkerBase {
    identifier: Identifier,
    ears: Vec<Ear>,
    voices: Vec<MVoice>,
    ear_call: bool,
    hidden: bool,
}

impl TalkerBase {
    pub fn new() -> Self {
        Self {
            identifier: Identifier::new("", "", TALKER_COUNT.fetch_add(1, Ordering::SeqCst)),
            ears: Vec::new(),
            voices: Vec::new(),
            ear_call: false,
            hidden: false,
        }
    }
    pub fn add_ear<'a>(&'a mut self, ear: Ear) {
        self.ears.push(ear);
    }
    pub fn add_voice<'a>(&'a mut self, voice: MVoice) {
        self.voices.push(voice);
    }
    pub fn identifier<'a>(&'a self) -> &'a Identifier {
        &self.identifier
    }

    pub fn id(&self) -> u32 {
        self.identifier.id()
    }
    pub fn name<'a>(&'a self) -> &'a String {
        self.identifier.name()
    }
    pub fn set_name(&mut self, name: &String) {
        self.identifier.set_name(name);
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
    fn name<'a>(&'a self) -> &'a String {
        self.base().name()
    }
    fn is_hidden(&self) -> bool {
        self.base().is_hidden()
    }
    fn depends_of(&self, id: u32) -> bool {
        self.base().id() == id
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

    fn set_ear_value_by_tag(&mut self, tag: &String, value: f32) -> bool {
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
    fn set_ear_voice_by_tag(&mut self, tag: &String, talker: &MTalker, port: usize) -> bool {
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

    fn set_ear_voice_by_index(&self, index: usize, talker: &MTalker, port: usize) -> bool {
        ear::visit_ear_flatten_index(self.ears(), index, |talk| {
            ear::set_talk_voice(talk, talker, port)
        })
    }

    fn talk(&mut self, port: usize, tick: i64, len: usize) -> usize;
}

pub type MTalker = Rc<RefCell<dyn Talker>>;
