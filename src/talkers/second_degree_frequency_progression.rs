use gpplugin::audio_format::AudioFormat;
use gpplugin::ear;
use gpplugin::talker::{Talker, TalkerBase};
use gpplugin::talker_handler::TalkerHandlerBase;
use gpplugin::voice;
use std::f64::consts::PI;

pub struct SecondDegreeFrequencyProgression {
    base: TalkerBase,
    f: f64,
    a: f64,
    b: f64,
    c: f64,
}

impl SecondDegreeFrequencyProgression {
    pub fn new(f: f64, a: f64, b: f64, c: f64) -> SecondDegreeFrequencyProgression {
        let mut base = TalkerBase::new();

        let voice = voice::audio(None, None, None);
        base.add_voice(voice);

        Self { base, f, a, b, c }
    }

    pub fn id() -> &'static str {
        "SecondDegreeFrequencyProgression"
    }
    pub fn descriptor() -> TalkerHandlerBase {
        TalkerHandlerBase::new(
            SecondDegreeFrequencyProgression::id(),
            "Second degree frequency progression",
            "Generator",
        )
    }
}

impl Talker for SecondDegreeFrequencyProgression {
    fn base<'a>(&'a self) -> &'a TalkerBase {
        &self.base
    }
    fn talk(&mut self, _port: usize, tick: i64, len: usize) -> usize {
        let c = AudioFormat::frequence_coef();
        let f = self.f;

        for voice in self.voices() {
            let mut vc = voice.borrow_mut();
            let voice_buf = vc.audio_buffer().unwrap();

            for i in 0..len {
                let t = (tick + i as i64) as f64;
                println!("tick = {}, i = {}, t = {}", tick, i, t);
                let sample = (t * f * c).sin() as f32;
                voice_buf.get()[i].set(sample);
            }
            vc.set_len(len);
            vc.set_tick(tick);
        }
        self.f = self.a * f * f + self.b * f + self.c;
        len
    }
}