use talker::ctalker;
use talker::ear;
use talker::ear::Init;
use talker::talker::{CTalker, Talker, TalkerBase};
use talker::talker_handler::TalkerHandlerBase;

pub const MODEL: &str = "Pan";

const LEFT_VOICE_PORT: usize = 0;

pub struct Pan {}
impl Pan {
    pub fn new(mut base: TalkerBase) -> Result<CTalker, failure::Error> {
        base.add_ear(ear::audio(Some("pan"), -1., 1., 0., &Init::DefValue)?);

        base.add_cv_voice(Some("L"), 0.);
        base.add_cv_voice(Some("R"), 0.);

        Ok(ctalker!(base, Self {}))
    }

    pub fn descriptor() -> TalkerHandlerBase {
        TalkerHandlerBase::builtin("Mixer", MODEL, MODEL)
    }
}

impl Talker for Pan {
    fn talk(&mut self, base: &TalkerBase, port: usize, tick: i64, len: usize) -> usize {
        let ln = base.listen(tick, len);
        let pan_buf = base.ear_audio_buffer(0);

        let voice_buf = base.voice(port).cv_buffer();

        if port == LEFT_VOICE_PORT {
            for i in 0..ln {
                voice_buf[i] =  (1. - pan_buf[i]) * 0.5;
            }
        }
        else {
            for i in 0..ln {
                voice_buf[i] =  (1. + pan_buf[i]) * 0.5;
            }
        }
        ln
    }
}
