use talker::ctalker;
use talker::talker::{CTalker, Talker, TalkerBase};
use talker::talker_handler::TalkerHandlerBase;

use tables::parabolic;
use talkers::table_talker::TableTalker;

pub const MODEL: &str = "Parabolic";

pub struct Parabolic {
    table_talker: TableTalker,
}

impl Parabolic {
    pub fn new(mut base: TalkerBase) -> Result<CTalker, failure::Error> {
        let table_talker = TableTalker::new(&mut base, parabolic::LEN)?;

        Ok(ctalker!(base, Self { table_talker }))
    }

    pub fn descriptor() -> TalkerHandlerBase {
        TalkerHandlerBase::builtin("Oscillator", MODEL, MODEL)
    }
}

impl Talker for Parabolic {
    fn talk(&mut self, base: &TalkerBase, port: usize, tick: i64, len: usize) -> usize {
        self.table_talker
            .talk(base, port, tick, len, &parabolic::TAB)
    }
}
