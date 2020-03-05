use crate::data::Data;
use crate::talker::{Talker, TalkerBase};
use crate::voice;

pub const MODEL: &str = "CvTalker";

pub struct CvTalker {
    base: TalkerBase,
    value: f32,
}

impl CvTalker {
    pub fn new(ovalue: Option<f32>, hidden: Option<bool>) -> CvTalker {
        let mut base = TalkerBase::new("", MODEL);
	let value = ovalue.unwrap_or(0.);
        let voice = voice::cv(None, Some(value), None);
        base.add_voice(voice);
        base.set_hidden(hidden.unwrap_or(false));

        Self { base, value }
    }
}

impl Talker for CvTalker {
    fn base<'a>(&'a self) -> &'a TalkerBase {
        &self.base
    }
    fn model(&self) -> &str {
        MODEL
    }
    fn data(&self) -> Data {
        Data::f(self.value)
    }

    fn talk(&mut self, _port: usize, tick: i64, len: usize) -> usize {
        for voice in self.voices() {
            let mut vc = voice.borrow_mut();

            vc.set_len(len);
            vc.set_tick(tick);
        }

        len
    }
}
