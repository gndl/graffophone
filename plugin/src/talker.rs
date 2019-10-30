extern crate failure;
use crate::ear::Ear;
use crate::identifier::Identifier;
use std::sync::atomic::{AtomicU32, Ordering};

static TALKER_COUNT: AtomicU32 = AtomicU32::new(1);

pub struct TalkerBase {
    pub identifier: Identifier,
    get_ear_call: bool,
}

impl TalkerBase {
    pub fn new() -> Self {
        Self {
            identifier: Identifier::new("", "", TALKER_COUNT.fetch_add(1, Ordering::SeqCst)),
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
    fn ears<'a>(&'a self) -> &'a Vec<Ear>;
}

pub struct TalkerHandlerBase {
    pub kind: String,
    pub category: String,
}

impl TalkerHandlerBase {
    pub fn new(kind: &str, category: &str) -> Self {
        Self {
            kind: kind.to_string(),
            category: category.to_string(),
        }
    }
}

pub trait TalkerHandler {
    fn base<'a>(&'a self) -> &'a TalkerHandlerBase;

    fn kind<'a>(&'a self) -> &'a String {
        &self.base().kind
    }
    fn category<'a>(&'a self) -> &'a String {
        &self.base().category
    }

    fn make(&self) -> Result<Box<dyn Talker>, failure::Error>;
}
