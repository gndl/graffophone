use crate::data::Data;
use crate::talker::{Talker, TalkerBase};
use crate::voice;

pub const MODEL: &str = "ControlTalker";

pub struct ControlTalker {
    base: TalkerBase,
}

impl ControlTalker {
    pub fn new(def_value: f32, hidden: Option<bool>) -> ControlTalker {
        let value = if def_value.is_nan() { 1. } else { def_value };
        let mut base = TalkerBase::new_data("", MODEL, Data::f(value));
        let voice = voice::control(None, value, None);
        base.add_voice(voice);
        base.set_hidden(hidden.unwrap_or(false));

        Self { base }
    }
}

impl Talker for ControlTalker {
    fn base<'a>(&'a self) -> &'a TalkerBase {
        &self.base
    }
    fn model(&self) -> &str {
        MODEL
    }

    fn talk(&mut self, _port: usize, tick: i64, len: usize) -> usize {
        for voice in self.voices() {
            let mut vc = voice.borrow_mut();

            vc.set_tick(tick);
        }
        len
    }

    fn voice_value(&self, port: usize) -> Option<f32> {
        if self.is_hidden() {
            if let Some(voice) = self.voices().get(port) {
                return voice.borrow().control_value(0);
            }
        }
        None
    }
}
