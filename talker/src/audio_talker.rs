use crate::data::Data;
use crate::talker::{CTalker, Talker, TalkerBase};
use crate::voice;
use ctalker;

pub const MODEL: &str = "AudioTalker";

pub struct AudioTalker {}

impl AudioTalker {
    pub fn new(def_value: f32, hidden: Option<bool>) -> CTalker {
        let value = if def_value.is_nan() { 0. } else { def_value };
        let mut base = TalkerBase::new_data("", MODEL, Data::f(value));

        base.add_voice(voice::audio(None, value));
        base.set_hidden(hidden.unwrap_or(false));

        ctalker!(base, Self {})
    }
}

impl Talker for AudioTalker {
    fn talk(&mut self, _base: &TalkerBase, _port: usize, _tick: i64, len: usize) -> usize {
        len
    }
}
