use talker::ctalker;
use talker::talker::{CTalker, Talker, TalkerBase};
use talker::talker_handler::TalkerHandlerBase;

use tables::round;
use talkers::bounded_table_talker::BoundedTableTalker;

pub const MODEL: &str = "BoundedRound";

pub struct BoundedRound {
    table_talker: BoundedTableTalker,
}

impl BoundedRound {
    pub fn new(mut base: TalkerBase) -> Result<CTalker, failure::Error> {
        let table_talker = BoundedTableTalker::new(&mut base, round::LEN)?;

        Ok(ctalker!(base, Self { table_talker }))
    }

    pub fn descriptor() -> TalkerHandlerBase {
        TalkerHandlerBase::builtin("Oscillator", MODEL, MODEL)
    }
}

impl Talker for BoundedRound {
    fn talk(&mut self, base: &TalkerBase, port: usize, tick: i64, len: usize) -> usize {
        self.table_talker.talk(base, port, tick, len, &round::TAB)
    }
}
