use std::f32;

use talker::ctalker;
use talker::ear;
use talker::ear::Init;
use talker::talker::{CTalker, Talker, TalkerBase};
use talker::talker_handler::TalkerHandlerBase;

pub const MODEL: &str = "Accumulator";

pub struct Accumulator {
    prev_error: f32,
    mid_error: f32,
    prev_output: f32,
    integ_val: f32,
}
impl Accumulator {
    pub fn new(mut base: TalkerBase) -> Result<CTalker, failure::Error> {
        base.add_ear(ear::audio(None, -1., 1., 0., &Init::DefValue)?);
        base.add_ear(ear::cv(Some("integral"), 0., 1000., 1., &Init::DefValue)?);
        base.add_ear(ear::cv(Some("damper"), 0., 1000., 1., &Init::DefValue)?);

        base.add_audio_voice(None, 0.);

        Ok(ctalker!(
            base,
            Self {
                prev_error: 0.,
                mid_error: 0.,
                prev_output: 0.,
                integ_val: 0.,
            }
        ))
    }

    pub fn descriptor() -> TalkerHandlerBase {
        TalkerHandlerBase::builtin("Shaper", MODEL, MODEL)
    }
}

impl Talker for Accumulator {
    fn talk(&mut self, base: &TalkerBase, port: usize, tick: i64, len: usize) -> usize {
        let ln = base.listen(tick, len);
        let src_buf = base.ear_audio_buffer(0);
        let integral_buf = base.ear_cv_buffer(1);
        let damper_buf = base.ear_cv_buffer(2);
        let voice_buf = base.voice(port).audio_buffer();

        for i in 0..ln {
            let v = src_buf[i];
            let ik = integral_buf[i];
            let dk = damper_buf[i];
            let e = v - self.prev_output;

            if (e > 0. && e > self.prev_error) || (e < 0. && e < self.prev_error) {
                self.mid_error = e * 0.5;
                self.prev_error = e;
            } else if e == 0. {
                self.mid_error = 0.;
                self.prev_error = 0.;
            }

            self.integ_val = self.integ_val + (e - (self.mid_error * dk)) * ik;
            self.prev_output = self.prev_output + self.integ_val;

            voice_buf[i] = self.prev_output;
        }

        ln
    }
}
