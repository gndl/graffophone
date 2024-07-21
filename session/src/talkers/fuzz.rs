use talker::ctalker;
use talker::ear;
use talker::ear::Init;
use talker::talker::{CTalker, Talker, TalkerBase};
use talker::talker_handler::TalkerHandlerBase;

pub const MODEL: &str = "Fuzz";

pub struct Fuzz {}
impl Fuzz {
    pub fn new(mut base: TalkerBase) -> Result<CTalker, failure::Error> {
        base.add_ear(ear::audio(None, -1., 1., 0., &Init::DefValue)?);
        base.add_ear(ear::cv(Some("iterations"), 0., 100., 1., &Init::DefValue)?);

        base.add_audio_voice(None, 0.);

        Ok(ctalker!(base, Self {}))
    }

    pub fn descriptor() -> TalkerHandlerBase {
        TalkerHandlerBase::builtin("Modulator", MODEL, MODEL)
    }
}

impl Talker for Fuzz {
    fn talk(&mut self, base: &TalkerBase, port: usize, tick: i64, len: usize) -> usize {
        let ln = base.listen(tick, len);
        let src_buf = base.ear_audio_buffer(0);
        let iterations_buf = base.ear_cv_buffer(1);
        let voice_buf = base.voice(port).audio_buffer();

        for i in 0..ln {
            let mut input = src_buf[i];
            let num_iter = iterations_buf[i] as usize;

            for _ in 0..num_iter {
                if input < 0.0 {
                    let negated = input + 1.0;
                    input = (negated * negated * negated) - 1.0;
                } else {
                    let negated = input - 1.0;
                    input = (negated * negated * negated) + 1.0;
                }
            }
            voice_buf[i] = input;
        }

        ln
    }
}
