use crate::talker::{Talker, TalkerBase};
use crate::voice;
use crate::voice::{AudioVoice, Voice};

pub struct AudioTalker {
    base: TalkerBase,
    voice: AudioVoice,
}

impl AudioTalker {
    pub fn new(value: Option<f32>, hidden: Option<bool>) -> AudioTalker {
        let mut base = TalkerBase::new();
        let voice = voice::audio(None, value);
        base.set_hidden(hidden.unwrap_or(false));
        //        base.add_voice(Voice::Audio(voice.clone()));

        Self { base, voice }
    }
}

impl Talker for AudioTalker {
    fn base<'a>(&'a self) -> &'a TalkerBase {
        &self.base
    }
    // fn voices<'a>(&'a self) -> &'a Vec<Voice> {
    //     &self.base().voices
    // }
    fn talk(&mut self, _port: u32, tick: i64, len: usize) {
        let mut voice = self.voice.get_mut();

        voice.check_length(len);
        voice.set_len(len);
        voice.set_tick(tick);
    }
}
