use crate::cornet;
use crate::talker::Talker;

type Port = i32;

pub struct Voice {
    tick: i64,
    len: usize,
    cor: cornet::Cornet,
    tkr: Box<dyn Talker>,
    port: Port,
    tag: String,
}

impl Voice {
    pub fn new(tick: i64, len: usize, tkr: Box<dyn Talker>, port: i32, tag: String) -> Self {
        Self {
            tick: tick,
            len: len,
            cor: cornet::new(len),
            tkr: tkr,
            port: port,
            tag: tag,
        }
    }

    pub fn check_length(&mut self, len: usize) {
        if self.cor.len() < len {
            self.cor = cornet::new(len);
        }
    }

    pub fn is_from(&mut self, tkr_id: u32, port: Port) -> bool {
        self.port == port && self.tkr.id() == tkr_id
    }
}
