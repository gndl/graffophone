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
    current_output: f32,
    dy: f32,
    period_part_end_idx: usize,
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
                current_output: 0.,
                dy: -1.,
                period_part_end_idx: 0,
            }
        ))
    }

    pub fn descriptor() -> TalkerHandlerBase {
        TalkerHandlerBase::builtin("Oscillator", MODEL, MODEL)
    }
}

impl Talker for Triangle {
    fn activate(&mut self) {
        self.current_output = 0.;
        self.dy = -1.;
        self.period_part_end_idx = 0;
    }

    fn talk(&mut self, base: &TalkerBase, port: usize, tick: i64, len: usize) -> usize {
        let ln = base.listen(tick, len);
        let freq_buf = base.ear_cv_buffer(FREQ_EAR_INDEX);
        let ratio_buf = base.ear_audio_buffer(RATIO_EAR_INDEX);
        let gain_buf = base.ear_cv_buffer(GAIN_EAR_INDEX);
        let voice_buf = base.voice(port).audio_buffer();

        let mut current_output = self.current_output;
        let mut dy = self.dy;
        let mut period_part_end_idx = self.period_part_end_idx;

        let mut i: usize = 0;

        while i < ln {
            if i == period_part_end_idx {
                let freq = f64::EPSILON.max(freq_buf[i] as f64);
                let ratio = ((ratio_buf[i] as f64 + 1.) * 0.5).clamp(f64::EPSILON, 1. - f64::EPSILON);
                let period = self.sample_rate / freq;
                let mut period_part = period * ratio;

                if dy < 0. {
                    current_output = -1.;
                    dy = (2. / period_part) as f32;
                    
                    if tick == 0 && i == 0 {
                        current_output = 0.;
                        period_part *= 0.5;
                    }
                }
                else {
                    current_output = 1.;
                    period_part = period * (1. - ratio);
                    dy = (-2. / period_part) as f32;
                }

                period_part_end_idx = i + period_part as usize;
            }

            let end_idx = ln.min(period_part_end_idx);

            while i < end_idx {
                voice_buf[i] = current_output * gain_buf[i];
                current_output += dy;
                i += 1;
            }
        }

        self.current_output = current_output;
        self.dy = dy;
        self.period_part_end_idx = period_part_end_idx - ln;

        ln
    }
}
