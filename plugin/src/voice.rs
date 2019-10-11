use crate::talker::Talker;
use lilv::port::buffer::VecBuffer;
use std::rc::Rc;

type Port = i32;

pub struct Voice {
    tick: i64,
    len: usize,
    cor: Rc<VecBuffer<f32>>,
    tkr: Box<dyn Talker>,
    port: Port,
    tag: String,
}

impl Voice {
    pub fn new(tick: i64, len: usize, tkr: Box<dyn Talker>, port: i32, tag: String) -> Self {
        Self {
            tick: tick,
            len: len,
            cor: Rc::new(VecBuffer::new(0f32, len)),
            tkr: tkr,
            port: port,
            tag: tag,
        }
    }

    pub fn check_length(&mut self, len: usize) {
        if self.cor.len() < len {
            self.cor = Rc::new(VecBuffer::new(0f32, len));
        }
    }

    pub fn is_from(&mut self, tkr_id: u32, port: Port) -> bool {
        self.port == port && self.tkr.get_id() == tkr_id
    }
}
