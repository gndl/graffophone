use std::f32;
use std::f64::consts::PI;

use talker::audio_format::AudioFormat;
use talker::ctalker;
use talker::ear;
use talker::ear::Init;
use talker::talker::{CTalker, Talker, TalkerBase};
use talker::talker_handler::TalkerHandlerBase;
use talker::voice;

use tables::round;

pub const MODEL: &str = "Round";

pub struct Round {
    len_on_sr: f32,
    last_tick: i64,
    last_pos: usize,
}

impl Round {
    pub fn new() -> Result<CTalker, failure::Error> {
        let mut base = TalkerBase::new("Round", MODEL);

        base.add_ear(ear::cv(Some("freq"), 0., 20000., 440., &Init::DefValue)?);
        base.add_ear(ear::audio(Some("phase"), -1., 1., 0., &Init::DefValue)?);
        base.add_ear(ear::audio(Some("gain"), -1., 1., 1., &Init::DefValue)?);

        base.add_voice(voice::audio(None, 0.));

        let len_on_sr = (round::LEN as f64 / AudioFormat::sample_rate() as f64) as f32;

        Ok(ctalker!(
            base,
            Self {
                len_on_sr,
                last_tick: 0,
                last_pos: 0,
            }
        ))
    }

    pub fn descriptor() -> TalkerHandlerBase {
        TalkerHandlerBase::new("Oscillator", MODEL, "Round")
    }
}

impl Talker for Round {
    fn talk(&mut self, base: &TalkerBase, port: usize, tick: i64, len: usize) -> usize {
        let ln = base.listen(tick, len);
        let freq_buf = base.ear_cv_buffer(0);
        let phase_buf = base.ear_audio_buffer(1);
        let gain_buf = base.ear_audio_buffer(2);
        let voice_buf = base.voice(port).audio_buffer();

        let phase_coef = round::LEN as f32 / 2.;
        let mut last_pos: usize = 0;

        if self.last_tick == tick {
            last_pos = self.last_pos;
        }

        for i in 0..ln {
            let p = (phase_buf[i] * phase_coef) as usize;
            let g = gain_buf[i];

            let mut pos = last_pos + (freq_buf[i] * self.len_on_sr) as usize + p;

            if pos >= round::LEN {
                pos -= round::LEN;
            }

            voice_buf[i] = round::TAB[pos] * g;
            last_pos = pos;
        }

        self.last_pos = last_pos;
        self.last_tick = tick + ln as i64;
        ln
    }
}
