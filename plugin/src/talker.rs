use crate::ear::Ear;
use crate::identifier::Identifier;
use std::sync::atomic::{AtomicU32, Ordering};

static TALKER_COUNT: AtomicU32 = AtomicU32::new(1);

pub struct Base {
    pub identifier: Identifier,
    get_ear_call: bool,
}

impl Base {
    pub fn new() -> Self {
        Self {
            identifier: Identifier::new("", "", TALKER_COUNT.fetch_add(1, Ordering::SeqCst)),
            get_ear_call: false,
        }
    }
    /*
        fn identifier<'a>(&'a self) -> &'a Identifier{
            self.identifier
        }
    */
}

pub trait Talker {
    fn base<'a>(&'a self) -> &'a Base;
    fn get_id(&self) -> u32 {
        self.base().identifier.get_id()
    }
    fn depends_of(&self, id: u32) -> bool;
    fn get_ears<'a>(&'a self) -> &'a Vec<Ear>;
}
