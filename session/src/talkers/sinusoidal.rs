use std::f32;
use std::f64::consts::PI;

use talker::audio_format::AudioFormat;
use talker::ctalker;
use talker::ear;
use talker::ear::Init;
use talker::talker::{CTalker, Talker, TalkerBase};
use talker::talker_handler::TalkerHandlerBase;

pub const MODEL: &str = "Sinusoidal";

pub struct Sinusoidal {
    frequence_coef: f64,
    last_tick: i64,
    last_angle: f64,
}

impl Sinusoidal {
    pub fn new(mut base: TalkerBase) -> Result<CTalker, failure::Error> {
        base.add_ear(ear::cv(Some("freq"), 0., 20000., 440., &Init::DefValue)?);
        base.add_ear(ear::audio(Some("phase"), -1., 2., 0., &Init::DefValue)?);
        base.add_ear(ear::audio(Some("gain"), -1., 1., 1., &Init::DefValue)?);

        base.add_audio_voice(None, 0.);

        Ok(ctalker!(
            base,
            Self {
                frequence_coef: AudioFormat::frequence_coef(),
                last_tick: 0,
                last_angle: 0.,
            }
        ))
    }

    pub fn descriptor() -> TalkerHandlerBase {
        TalkerHandlerBase::builtin("Oscillator", MODEL, MODEL)
    }
}

impl Talker for Sinusoidal {
    fn talk(&mut self, base: &TalkerBase, port: usize, tick: i64, len: usize) -> usize {
        let ln = base.listen(tick, len);
        let freq_buf = base.ear_cv_buffer(0);
        let phase_buf = base.ear_audio_buffer(1);
        let gain_buf = base.ear_audio_buffer(2);
        let voice_buf = base.voice(port).audio_buffer();
        let c = self.frequence_coef;

        let mut last_angle = if self.last_tick == tick {
            self.last_angle
        } else {
            if tick == 0 {
                0.
            } else {
                -freq_buf[0] as f64 * c
            }
        };

        for i in 0..ln {
            let f = freq_buf[i] as f64;
            let a = last_angle + c * f;
            let p = phase_buf[i] as f64 * PI;

            voice_buf[i] = ((a + p).sin() as f32) * gain_buf[i];
            last_angle = a;
        }

        self.last_angle = last_angle;
        self.last_tick = tick + ln as i64;
        ln
    }
}
