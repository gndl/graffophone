use crate::talker::{Talker, TalkerBase};
use crate::voice;
use crate::voice::{ControlVoice, Voice};

pub struct ControlTalker {
    base: TalkerBase,
    voice: ControlVoice,
}

impl ControlTalker {
    pub fn new(value: Option<f32>, hidden: Option<bool>) -> ControlTalker {
        let mut base = TalkerBase::new();
        let voice = voice::control(None, Some(value.unwrap_or(1.)));

        base.set_hidden(hidden.unwrap_or(false));
        //        base.add_voice(Voice::Control(voice.clone()));
        Self { base, voice }

        /*
        let tkr = Self {
            base: TalkerBase::new(),
            voice: voice::control(None, Some(value.unwrap_or(1.))),
        };
        tkr.base.set_hidden(hidden.unwrap_or(false));
        tkr.base.add_voice(Voice::Control(&tkr.voice));

        return tkr;
        */
    }
}

impl Talker for ControlTalker {
    fn base<'a>(&'a self) -> &'a TalkerBase {
        &self.base
    }
    fn talk(&mut self, _port: u32, tick: i64, _len: usize) {
        //        self.voice.set_tick(tick);
    }
}
