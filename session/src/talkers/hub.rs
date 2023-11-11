use talker::ctalker;
use talker::ear;
use talker::ear::Init;
use talker::talker::{CTalker, Talker, TalkerBase};
use talker::talker_handler::TalkerHandlerBase;
use talker::voice;

pub const MODEL: &str = "Hub";

pub struct Hub {}
impl Hub {
    pub fn new() -> Result<CTalker, failure::Error> {
        let mut base = TalkerBase::new(MODEL, MODEL);

        base.add_ear(ear::audio(None, -1., 1., 0., &Init::DefValue)?);

        base.add_voice(voice::audio(None, 0.));

        Ok(ctalker!(base, Self {}))
    }

    pub fn descriptor() -> TalkerHandlerBase {
        TalkerHandlerBase::builtin("Modulator", MODEL, MODEL)
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
