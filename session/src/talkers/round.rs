use talker::ctalker;
use talker::talker::{CTalker, Talker, TalkerBase};
use talker::talker_handler::TalkerHandlerBase;

use tables::round;
use talkers::table_talker::TableTalker;

pub const MODEL: &str = "Round";

pub struct Round {
    table_talker: TableTalker,
}

impl Round {
    pub fn new(mut base: TalkerBase) -> Result<CTalker, failure::Error> {
        let table_talker = TableTalker::new(&mut base, round::LEN)?;

        Ok(ctalker!(base, Self { table_talker }))
    }

    pub fn descriptor() -> TalkerHandlerBase {
        TalkerHandlerBase::builtin("Oscillator", MODEL, "Round")
    }
}

impl Talker for Round {
    fn talk(&mut self, base: &TalkerBase, port: usize, tick: i64, len: usize) -> usize {
        self.table_talker.talk(base, port, tick, len, &round::TAB)
    }
}
