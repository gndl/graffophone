use crate::data::Data;
use crate::talker::{CTalker, Talker, TalkerBase};
use ctalker;

pub const MODEL: &str = "CvTalker";

pub struct CvTalker {}

impl CvTalker {
    pub fn new(def_value: f32, hidden: Option<bool>) -> CTalker {
        let value = if def_value.is_nan() { 0. } else { def_value };
        let mut base = TalkerBase::new_data("", MODEL, Data::f(value), true);

        base.add_cv_voice(None, value);
        base.set_hidden(hidden.unwrap_or(false));

        ctalker!(base, Self {})
    }
}

impl Talker for CvTalker {
    fn talk(&mut self, _base: &TalkerBase, _port: usize, _tick: i64, len: usize) -> usize {
        len
    }
}
