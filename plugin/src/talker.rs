extern crate failure;
use crate::ear::Ear;
use crate::identifier::Identifier;
use crate::voice::Voice;
use std::sync::atomic::{AtomicU32, Ordering};

static TALKER_COUNT: AtomicU32 = AtomicU32::new(1);

pub struct TalkerBase {
    pub identifier: Identifier,
    ears: Vec<Ear>,
    voices: Vec<Voice>,
    get_ear_call: bool,
}

impl TalkerBase {
    pub fn new() -> Self {
        Self {
            identifier: Identifier::new("", "", TALKER_COUNT.fetch_add(1, Ordering::SeqCst)),
            ears: Vec::new(),
            voices: Vec::new(),
            get_ear_call: false,
        }
    }
    /*
            pub fn identifier<'a>(&'a self) -> &'a Identifier{
                self.identifier
            }

    pub fn get_id(&self) -> u32 {self.identifier.get_id()}
    pub fn get_name(&self) -> u32 {self.identifier.get_name()}
    */
}

pub trait Talker {
    fn base<'a>(&'a self) -> &'a TalkerBase;
    fn id(&self) -> u32 {
        self.base().identifier.get_id()
    }
    fn name<'a>(&'a self) -> &'a String {
        self.base().identifier.get_name()
    }
    fn depends_of(&self, id: u32) -> bool;

    fn ears<'a>(&'a self) -> &'a Vec<Ear> {
        &self.base().ears
    }
    fn voices<'a>(&'a self) -> &'a Vec<Voice> {
        &self.base().voices
    }
}
