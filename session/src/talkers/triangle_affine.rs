use std::f32;

use talker::audio_format::AudioFormat;
use talker::ctalker;
use talker::ear;
use talker::ear::Init;
use talker::identifier::Index;
use talker::talker::{CTalker, Talker, TalkerBase};
use talker::talker_handler::TalkerHandlerBase;

pub const MODEL: &str = "Triangle";
//ascent and end of descent

pub struct Triangle {
    sample_rate: f64,
    ascent_a: f64,
    ascent_b: f64,
    ascent_end_tick: i64,
    descent_a: f64,
    descent_b: f64,
    descent_end_tick: i64,
}

const FREQ_EAR_INDEX: Index = 0;
const RATIO_EAR_INDEX: Index = 1;
const GAIN_EAR_INDEX: Index = 2;

impl Triangle {
    pub fn new(mut base: TalkerBase) -> Result<CTalker, failure::Error> {
        base.add_ear(ear::cv(Some("freq"), 0., 20000., 440., &Init::DefValue)?);
        base.add_ear(ear::audio(Some("ratio"), -1., 1., 0., &Init::DefValue)?);
        base.add_ear(ear::cv(Some("gain"), -1., 4., 1., &Init::DefValue)?);

        base.add_audio_voice(None, 0.);

        Ok(ctalker!(
            base,
            Self {
                sample_rate: AudioFormat::sample_rate() as f64,
                ascent_a: 0.0,
                ascent_b: 0.0,
                ascent_end_tick: 0,
                descent_a: 0.0,
                descent_b: 0.0,
                descent_end_tick: 0,
            }
        ))
    }

    pub fn descriptor() -> TalkerHandlerBase {
        TalkerHandlerBase::builtin("Oscillator", MODEL, MODEL)
    }
}

impl Talker for Triangle {
    fn activate(&mut self) {
        self.ascent_end_tick = 0;
        self.descent_end_tick = 0;
    }

    fn talk(&mut self, base: &TalkerBase, port: usize, tick: i64, len: usize) -> usize {
        let ln = base.listen(tick, len);
        let freq_buf = base.ear_cv_buffer(FREQ_EAR_INDEX);
        let ratio_buf = base.ear_audio_buffer(RATIO_EAR_INDEX);
        let gain_buf = base.ear_cv_buffer(GAIN_EAR_INDEX);
        let voice_buf = base.voice(port).audio_buffer();

        let mut ascent_a = self.ascent_a;
        let mut ascent_b = self.ascent_b;
        let mut ascent_end_idx = if self.ascent_end_tick < tick {
            0
        } else {
            (self.ascent_end_tick - tick) as usize
        };

        let mut descent_a = self.descent_a;
        let mut descent_b = self.descent_b;
        let mut descent_end_idx = if self.descent_end_tick < tick {
            0
        } else {
            (self.descent_end_tick - tick) as usize
        };

        let mut i: usize = 0;
        let mut x = tick as f64;

        while i < ln {
            if i == descent_end_idx {
                let f = f64::EPSILON.max(freq_buf[i] as f64);
                let r = f64::EPSILON.max((ratio_buf[i] as f64 + 1.) * 0.5);
                let p = self.sample_rate / f;

                ascent_a = 2. / (p * r);
                descent_a = -2. / (p * (1. + f64::EPSILON - r));

                let mut ascent_end_x = x + (p * r);
                let mut descent_end_x = x + p;

                if tick == 0 && i == 0 {
                    let a_e = p * r * 0.5;
                    ascent_end_x = a_e;
                    descent_end_x = p - a_e;
                }

                ascent_b = 1.0 - ascent_a * ascent_end_x;
                descent_b = -1.0 - descent_a * descent_end_x;

                ascent_end_idx = (ascent_end_x - tick as f64) as usize;
                descent_end_idx = (descent_end_x - tick as f64) as usize;

                println!("i {}, ascent_a {}, ascent_end_idx {}, descent_a {}, descent_end_idx {}", i, ascent_a, ascent_end_idx, descent_a, descent_end_idx);
            }

            let ascent_end = ln.min(ascent_end_idx);

            while i < ascent_end {
                voice_buf[i] = (ascent_a * x + ascent_b).clamp(-1.0, 1.0) as f32 * gain_buf[i];
                x += 1.0;
                i += 1;
            }

            let descent_end = ln.min(descent_end_idx);

            while i < descent_end {
                voice_buf[i] = (descent_a * x + descent_b).clamp(-1.0, 1.0) as f32 * gain_buf[i];
                x += 1.0;
                i += 1;
            }
        }

        self.ascent_a = ascent_a;
        self.ascent_b = ascent_b;
        self.ascent_end_tick = ascent_end_idx as i64 + tick;
        self.descent_a = descent_a;
        self.descent_b = descent_b;
        self.descent_end_tick = descent_end_idx as i64 + tick;

        ln
    }
}
