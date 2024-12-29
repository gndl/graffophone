use talker::ctalker;
use talker::ear;
use talker::ear::Init;
use talker::talker::{CTalker, Talker, TalkerBase};
use talker::talker_handler::TalkerHandlerBase;

pub const MODEL: &str = "Hub";

pub struct Hub {}
impl Hub {
    pub fn new(mut base: TalkerBase) -> Result<CTalker, failure::Error> {
        base.add_ear(ear::audio(None, -1., 1., 0., &Init::DefValue)?);

        base.add_audio_voice(None, 0.);

        Ok(ctalker!(base, Self {}))
    }

    pub fn descriptor() -> TalkerHandlerBase {
        TalkerHandlerBase::builtin("Mixer", MODEL, MODEL)
    }
}

impl Talker for Hub {
    fn talk(&mut self, base: &TalkerBase, port: usize, tick: i64, len: usize) -> usize {
        let ln = base.listen(tick, len);
        let input_buf = base.ear_audio_buffer(0);
        let voice_buf = base.voice(port).audio_buffer();

        for i in 0..ln {
            voice_buf[i] = input_buf[i];
        }

        ln
    }
}
