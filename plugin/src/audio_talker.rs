use crate::talker::{Talker, TalkerBase};
use crate::voice;

pub struct AudioTalker {
    base: TalkerBase,
}

impl AudioTalker {
    pub fn new(value: Option<f32>, hidden: Option<bool>) -> AudioTalker {
        let mut base = TalkerBase::new();
        let voice = voice::audio(None, value, None);
        base.set_hidden(hidden.unwrap_or(false));
        base.add_voice(voice);

        Self { base }
    }
}

impl Talker for AudioTalker {
    fn base<'a>(&'a self) -> &'a TalkerBase {
        &self.base
    }

    fn talk(&mut self, _port: usize, tick: i64, len: usize) {
        self.voices().iter().for_each(|voice| {
            let mut vc = voice.borrow_mut();

            vc.set_len(len);
            vc.set_tick(tick);
        })
    }
}
