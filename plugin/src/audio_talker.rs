use crate::talker::{Talker, TalkerBase};
use crate::voice;

pub struct AudioTalker {
    base: TalkerBase,
}

impl AudioTalker {
    pub fn new(value: Option<f32>, hidden: Option<bool>) -> AudioTalker {
        let mut base = TalkerBase::new();
        let voice = voice::audio(None, value, None);
        base.add_voice(voice);
        base.set_hidden(hidden.unwrap_or(false));

        Self { base }
    }
}

impl Talker for AudioTalker {
    fn base<'a>(&'a self) -> &'a TalkerBase {
        &self.base
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
