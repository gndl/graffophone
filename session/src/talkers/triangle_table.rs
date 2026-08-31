use std::f32;

use talker::audio_format::AudioFormat;
use talker::ctalker;
use talker::ear;
use talker::ear::Init;
use talker::identifier::Index;
use talker::talker::{CTalker, Talker, TalkerBase};
use talker::talker_handler::TalkerHandlerBase;

use tables::ramp;

pub const MODEL: &str = "Triangle";
//ascent and end of descent

pub struct Triangle {
    sample_rate: f64,
    ramp_pos: f64,
    ramp_step: f64,
    ascent_end_tick: i64,
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
                ramp_pos: 0.0,
                ramp_step: 0.0,
                ascent_end_tick: 0,
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

        let ramp_len = ramp::LEN as f64;
        let mut ramp_pos = self.ramp_pos;
        let mut ramp_step = self.ramp_step;

        let mut ascent_end_idx = if self.ascent_end_tick < tick {
            usize::MAX
        } else {
            (self.ascent_end_tick - tick) as usize
        };

        let mut descent_end_idx = if self.descent_end_tick < tick {
            usize::MAX
        } else {
            (self.descent_end_tick - tick) as usize
        };

        let mut end_idx = if ascent_end_idx < descent_end_idx { ascent_end_idx } else { descent_end_idx };

        let mut i: usize = 0;
        println!(">> talk : i {}, ramp_pos {}, ramp_step {}, ascent_end_idx {}, descent_end_idx {}, end_idx {}", i, ramp_pos, ramp_step, ascent_end_idx, descent_end_idx, end_idx);

        while i < ln {
            if i == descent_end_idx {
                let freq = f64::EPSILON.max(freq_buf[i] as f64);
                let ratio = ((ratio_buf[i] as f64 + 1.) * 0.5).clamp(f64::EPSILON, 1.0);
                let period = self.sample_rate / freq;

                ramp_pos = 0.0;

                let mut ascent_period = period * ratio;
                ramp_step = ramp_len / ascent_period;

                if tick == 0 && i == 0 {
                    ramp_pos = ramp_len * 0.5;
                    ascent_period *= 0.5;
                }

                ascent_end_idx = i + ascent_period as usize;
                end_idx = ln.min(ascent_end_idx);
            }
            if i == ascent_end_idx {
                let freq = f64::EPSILON.max(freq_buf[i] as f64);
                let ratio = ((ratio_buf[i] as f64 + 1.) * 0.5).clamp(f64::EPSILON, 1.0);
                let period = self.sample_rate / freq;

                ramp_pos = ramp_len - 1.0;

                let descent_period = period * (1. + f64::EPSILON - ratio);
                ramp_step = -ramp_len / descent_period;
                descent_end_idx = i + descent_period as usize;

                if descent_end_idx == i {
                    continue;
                }
                end_idx = ln.min(descent_end_idx);
            }
            println!("i {}, ramp_pos {}, ramp_step {}, ascent_end_idx {}, descent_end_idx {}, end_idx {}", i, ramp_pos, ramp_step, ascent_end_idx, descent_end_idx, end_idx);

            while i < end_idx {
                let v = ramp::TAB[ramp_pos as usize] * gain_buf[i];
                // println!("{}", v);
                voice_buf[i] = v;
                ramp_pos += ramp_step;
                i += 1;
            }
        }

        self.ramp_pos = ramp_pos;
        self.ramp_step = ramp_step;
        self.ascent_end_tick = ascent_end_idx as i64 + tick;
        self.descent_end_tick = descent_end_idx as i64 + tick;

        ln
    }
}
