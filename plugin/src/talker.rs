extern crate failure;
use crate::ear::Ear;
use crate::identifier::Identifier;
use crate::voice::Voice;
use std::sync::atomic::{AtomicU32, Ordering};

static TALKER_COUNT: AtomicU32 = AtomicU32::new(1);

pub struct TalkerBase {
    identifier: Identifier,
    ears: Vec<Ear>,
    voices: Vec<Voice>,
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
    pub fn add_voice<'a>(&'a mut self, voice: Voice) {
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
    fn voices<'a>(&'a mut self) -> &'a Vec<Voice> {
        &self.base().voices
    }
    /*
        fn iter_voices(&mut self, f: FnMut(&Voice)) {
            for vc in &self.base().voices {
                f(vc);
            }
        }
    */
    fn talk(&mut self, port: usize, tick: i64, len: usize) -> usize;
}
