use crate::data::Data;
use crate::talker::{self, CTalker, Talker, TalkerBase};
use ctalker;

pub const MODEL: &str = "ControlTalker";

pub struct ControlTalker {}

impl ControlTalker {
    pub fn new(def_value: f32) -> CTalker {
        let value = if def_value.is_nan() { 1. } else { def_value };
        let mut base = TalkerBase::new_data(
            talker::get_next_unreachable_id(),
            "",
            MODEL,
            Data::f(value),
            true,
        );

        base.add_control_voice(None, value);
        base.set_hidden(true);

        ctalker!(base, Self {})
    }
}

impl Talker for ControlTalker {
    fn talk(&mut self, _base: &TalkerBase, _port: usize, _tick: i64, len: usize) -> usize {
        len
    }
}
