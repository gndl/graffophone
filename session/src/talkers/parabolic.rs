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
    pub fn new() -> Result<CTalker, failure::Error> {
        let mut base = TalkerBase::new("Parabolic", MODEL);
        let table_talker = TableTalker::new(&mut base, parabolic::LEN)?;

        Ok(ctalker!(base, Self { table_talker }))
    }

    pub fn descriptor() -> TalkerHandlerBase {
        TalkerHandlerBase::builtin("Oscillator", MODEL, "Parabolic")
    }
}

impl Talker for Parabolic {
    fn talk(&mut self, base: &TalkerBase, port: usize, tick: i64, len: usize) -> usize {
        self.table_talker
            .talk(base, port, tick, len, &parabolic::TAB)
    }
}
