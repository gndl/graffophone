use crate::talker::{CTalker, Talker, TalkerBase};
use crate::voice;
use ctalker;

pub const MODEL: &str = "AtomTalker";

pub struct AtomTalker {}

impl AtomTalker {
    pub fn new(hidden: Option<bool>) -> CTalker {
        let mut base = TalkerBase::new("", MODEL);
        let voice = voice::atom(None);
        base.add_voice(voice);
        base.set_hidden(hidden.unwrap_or(false));

        ctalker!(base, Self {})
    }
}

impl Talker for AtomTalker {
    fn talk(&mut self, base: &TalkerBase, _port: usize, tick: i64, len: usize) -> usize {
        for voice in base.voices() {
            voice.set_len(len);
            voice.set_tick(tick);
        }
        len
    }
}
