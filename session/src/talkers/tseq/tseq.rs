use std::f32;

//use talker::audio_format::AudioFormat;
use talker::ctalker;
use talker::data::Data;
//use talker::ear;
//use talker::ear::Init;
use talker::talker::{CTalker, Talker, TalkerBase};
use talker::talker_handler::TalkerHandlerBase;
//use talker::voice;

pub const MODEL: &str = "Tseq";

enum Progression {
    I,
    L,
    D,
    CS,
}

struct Note {
    start_tick: i64,
    end_tick: i64,
    start_freq: f32,
    end_freq: f32,
    prog: Progression,
}

pub struct Tseq {
    notes: Vec<Note>,
}

impl Tseq {
    pub fn new() -> Result<CTalker, failure::Error> {
        let mut base = TalkerBase::new("", MODEL);

        Ok(ctalker!(base, Self { notes: Vec::new() }))
    }

    pub fn descriptor() -> TalkerHandlerBase {
        TalkerHandlerBase::new("Oscillator", MODEL, MODEL)
    }
}

impl Talker for Tseq {
    fn set_data_update(
        &mut self,
        base: &TalkerBase,
        data: Data,
    ) -> Result<Option<TalkerBase>, failure::Error> {
        /*        base.add_ear(ear::cv(Some("freq"), 0., 20000., 440., &Init::DefValue)?);
                base.add_ear(ear::audio(Some("phase"), -1., 1., 0., &Init::DefValue)?);

                base.add_voice(voice::audio(None, 0.));
        */
        base.set_data(data);
        Ok(None)
    }

    fn talk(&mut self, base: &TalkerBase, port: usize, tick: i64, len: usize) -> usize {
        let ln = base.listen(tick, len);
        ln
    }
}
