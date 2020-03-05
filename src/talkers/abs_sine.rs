use gpplugin::audio_format::AudioFormat;
use gpplugin::ear;
use gpplugin::talker::{Talker, TalkerBase};
use gpplugin::talker_handler::TalkerHandlerBase;
use gpplugin::voice;
use std::f64::consts::PI;

pub const MODEL: &str = "AbsSine";

pub struct AbsSine {
    base: TalkerBase,
}

impl AbsSine {
    pub fn new() -> AbsSine {
        let mut base = TalkerBase::new("", MODEL);

        let freq = ear::audio(Some("frequence"), Some(440.), None);
        base.add_ear(freq);

        let voice = voice::audio(None, None, None);
        base.add_voice(voice);

        Self { base }
    }
    pub fn descriptor() -> TalkerHandlerBase {
        TalkerHandlerBase::new("Oscillator", MODEL, "Absolute sinusoidal")
    }
}

impl Talker for AbsSine {
    fn base<'a>(&'a self) -> &'a TalkerBase {
        &self.base
    }
    fn model(&self) -> &str {
        MODEL
    }

    fn talk(&mut self, _port: usize, tick: i64, len: usize) -> usize {
        let mut ln = len;
        let c = (PI * 2.0) / AudioFormat::sample_rate() as f64;

        for ear in self.ears() {
            ln = ear.listen(tick, ln);
        }
        for voice in self.voices() {
            let freq_buf = self.ear_audio_buffer(0).unwrap();
            let mut vc = voice.borrow_mut();
            let voice_buf = vc.audio_buffer().unwrap();

            for i in 0..ln {
                let sample = ((tick + i as i64) as f64 * freq_buf[i].get() as f64 * c).sin() as f32;
                voice_buf.get()[i].set(sample);
            }
            vc.set_len(ln);
            vc.set_tick(tick);
        }
        ln
    }
}
