use crate::data::Data;
use crate::talker::{Talker, TalkerBase};
use crate::voice;

pub const MODEL: &str = "ControlTalker";

pub struct ControlTalker {
    base: TalkerBase,
}

impl ControlTalker {
    pub fn new(ovalue: Option<f32>, hidden: Option<bool>) -> ControlTalker {
        let value = ovalue.unwrap_or(1.);
        let mut base = TalkerBase::new_data("", MODEL, Data::f(value));
        let voice = voice::control(None, Some(value), None);
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
        //        self.voices().iter().for_each(|voice| {
        for voice in self.voices() {
            let mut vc = voice.borrow_mut();

            vc.set_tick(tick);
        }
        len
    }
}
