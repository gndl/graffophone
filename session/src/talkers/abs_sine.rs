use std::f64::consts::PI;

use talker::audio_format::AudioFormat;
use talker::ctalker;
use talker::ear;
use talker::ear::Init;
use talker::talker::{CTalker, Talker, TalkerBase};
use talker::talker_handler::TalkerHandlerBase;
use talker::voice;

pub const MODEL: &str = "AbsSine";

pub struct AbsSine {}

impl AbsSine {
    pub fn new() -> Result<CTalker, failure::Error> {
        let mut base = TalkerBase::new("", MODEL);

        let freq = ear::audio(Some("freq"), 0., 20000., 440., &Init::DefValue)?;
        base.add_ear(freq);
        base.add_voice(voice::audio(None, 0.));

        Ok(ctalker!(base, Self {}))
    }
    pub fn descriptor() -> TalkerHandlerBase {
        TalkerHandlerBase::new("Oscillator", MODEL, "Absolute sinusoidal")
    }
}

impl Talker for AbsSine {
    fn talk(&mut self, base: &TalkerBase, port: usize, tick: i64, len: usize) -> usize {
        let ln = base.listen(tick, len);
        let c = (PI * 2.0) / AudioFormat::sample_rate() as f64;

        let freq_buf = base.ear_audio_buffer(0);
        let voice_buf = base.voice(port).audio_buffer();

        for i in 0..ln {
            let sample = ((tick + i as i64) as f64 * freq_buf[i] as f64 * c).sin() as f32;
            voice_buf[i] = sample;
        }
        ln
    }
}
