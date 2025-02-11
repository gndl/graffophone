use talker::audio_format::AudioFormat;
use talker::ctalker;
use talker::talker::{CTalker, Talker, TalkerBase};
use talker::talker_handler::TalkerHandlerBase;

pub const MODEL: &str = "SecondDegreeFrequencyProgression";

pub struct SecondDegreeFrequencyProgression {
    f: f64,
    a: f64,
    b: f64,
    c: f64,
}

impl SecondDegreeFrequencyProgression {
    pub fn new(f: f64, a: f64, b: f64, c: f64, mut base: TalkerBase) -> Result<CTalker, failure::Error> {
        base.add_audio_voice(None, 0.);

        Ok(ctalker!(base, Self { f, a, b, c }))
    }

    pub fn descriptor() -> TalkerHandlerBase {
        TalkerHandlerBase::builtin("Oscillator", MODEL, "Second Degree Frequency Progression")
    }
}

impl Talker for SecondDegreeFrequencyProgression {
    fn talk(&mut self, base: &TalkerBase, port: usize, tick: i64, len: usize) -> usize {
        let c = AudioFormat::frequence_coef();
        let f = self.f;

        let voice_buf = base.voice(port).audio_buffer();

        for i in 0..len {
            let t = (tick + i as i64) as f64;
            println!("tick = {}, i = {}, t = {}", tick, i, t);
            let sample = (t * f * c).sin() as f32;
            voice_buf[i] = sample;
        }

        self.f = self.a * f * f + self.b * f + self.c;
        len
    }
}
