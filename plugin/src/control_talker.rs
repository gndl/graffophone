use crate::talker::{Talker, TalkerBase};
use crate::voice;

pub struct ControlTalker {
    base: TalkerBase,
}

impl ControlTalker {
    pub fn new(value: Option<f32>, hidden: Option<bool>) -> ControlTalker {
        let mut base = TalkerBase::new();
        let voice = voice::control(None, Some(value.unwrap_or(1.)), None);

        base.set_hidden(hidden.unwrap_or(false));
        base.add_voice(voice);
        Self { base }
    }
}

impl Talker for ControlTalker {
    fn base<'a>(&'a self) -> &'a TalkerBase {
        &self.base
    }

    fn talk(&mut self, _port: usize, tick: i64, _len: usize) {
        self.voices().iter().for_each(|voice| {
            let mut vc = voice.borrow_mut();

            vc.set_tick(tick);
        })
    }
}
