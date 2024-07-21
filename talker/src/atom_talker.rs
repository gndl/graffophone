use crate::lv2_handler::Lv2Handler;
use crate::talker::{CTalker, Talker, TalkerBase};
use ctalker;

pub const MODEL: &str = "AtomTalker";

pub struct AtomTalker {}

impl AtomTalker {
    pub fn new(olv2_handler: Option<&Lv2Handler>, hidden: Option<bool>) -> CTalker {
        let mut base = TalkerBase::new("", MODEL, true);

        base.add_atom_voice(None, olv2_handler);
        base.set_hidden(hidden.unwrap_or(false));

        ctalker!(base, Self {})
    }
}

impl Talker for AtomTalker {
    fn talk(&mut self, _base: &TalkerBase, _port: usize, _tick: i64, len: usize) -> usize {
        len
    }
}
