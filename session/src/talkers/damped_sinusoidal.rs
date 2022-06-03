use std::f32;
use std::f64::consts::PI;

use talker::audio_format::AudioFormat;
use talker::ctalker;
use talker::ear;
use talker::ear::Init;
use talker::talker::{CTalker, Talker, TalkerBase};
use talker::talker_handler::TalkerHandlerBase;
use talker::voice;

pub const MODEL: &str = "Sinusoidal";

pub struct Sinusoidal {
    last_tick: i64,
    last_angle: f64,
}

impl Sinusoidal {
    pub fn new() -> Result<CTalker, failure::Error> {
        let mut base = TalkerBase::new("Sin", MODEL);

        base.add_ear(ear::cv(Some("freq"), 0., 20000., 440., &Init::DefValue)?);
        base.add_ear(ear::audio(Some("phase"), -1., 1., 0., &Init::DefValue)?);
        base.add_ear(ear::audio(Some("gain"), -1., 1., 1., &Init::DefValue)?);

        base.add_voice(voice::audio(None, 0.));

        Ok(ctalker!(
            base,
            Self {
                last_tick: 0,
                last_angle: 0.,
            }
        ))
    }

    pub fn descriptor() -> TalkerHandlerBase {
        TalkerHandlerBase::new("Oscillator", MODEL, "Sin")
    }
}

impl Talker for Sinusoidal {
    fn talk(&mut self, base: &TalkerBase, port: usize, tick: i64, len: usize) -> usize {
        let ln = base.listen(tick, len);
        let freq_buf = base.ear_cv_buffer(0);
        let phase_buf = base.ear_audio_buffer(1);
        let gain_buf = base.ear_audio_buffer(2);
        let voice_buf = base.voice(port).audio_buffer();
        let c = AudioFormat::frequence_coef();

        let mut last_angle = if self.last_tick == tick {
            self.last_angle
        } else {
            -freq_buf[0] as f64 * c
        };

        for i in 0..ln {
            let gain = gain_buf[i];

            if gain == 0. {
                last_angle = 0.;

                if i > 0 {
                    voice_buf[i] = voice_buf[i - 1] * 0.95;
                } else {
                    voice_buf[i] = 0.;
                }
            } else {
                let a = last_angle + freq_buf[i] as f64 * c;
                let p = phase_buf[i] as f64 * PI;

                voice_buf[i] = ((a + p).sin() as f32) * gain;
                last_angle = a;
            }
        }

        self.last_angle = last_angle;
        self.last_tick = tick + ln as i64;
        ln
    }
}
