use crate::talker::{CTalker, Talker, TalkerBase};
use crate::voice;
use ctalker;

pub const MODEL: &str = "AtomTalker";

pub struct AtomTalker {}

impl AtomTalker {
    pub fn new(hidden: Option<bool>) -> CTalker {
        let mut base = TalkerBase::new("", MODEL);

        base.add_voice(voice::atom(None));
        base.set_hidden(hidden.unwrap_or(false));

        ctalker!(base, Self {})
    }
}

impl Talker for AtomTalker {
    fn talk(&mut self, _base: &TalkerBase, _port: usize, _tick: i64, len: usize) -> usize {
        len
    }
}
