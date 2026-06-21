use crate::lv2_handler::Lv2Handler;
use crate::talker::{self, CTalker, Talker, TalkerBase};
use ctalker;

pub const MODEL: &str = "AtomTalker";

pub struct AtomTalker {}

impl AtomTalker {
    pub fn new(olv2_handler: Option<&Lv2Handler>) -> CTalker {
        let mut base = TalkerBase::new(
            talker::get_next_unreachable_id(),
            "",
            MODEL,
            true,
        );

        base.add_atom_voice(None, olv2_handler);
        base.set_hidden(true);

        ctalker!(base, Self {})
    }
}

impl Talker for AtomTalker {
    fn talk(&mut self, _base: &TalkerBase, _port: usize, _tick: i64, len: usize) -> usize {
        len
    }
}
