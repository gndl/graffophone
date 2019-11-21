use crate::audio_format::AudioFormat;
use crate::cornet;

pub struct Voice {
    tick: i64,
    len: usize,
    cor: cornet::Cornet,
    tag: String,
}

impl Voice {
    pub fn init(tag: String) -> Self {
        let len = AudioFormat::chunk_size();
        Self {
            tick: 0,
            len,
            cor: cornet::new(len),
            tag: tag,
        }
    }

    pub fn new(tick: i64, len: usize, tag: String) -> Self {
        Self {
            tick: tick,
            len: len,
            cor: cornet::new(len),
            tag: tag,
        }
    }

    pub fn check_length(&mut self, len: usize) {
        if self.cor.len() < len {
            self.cor = cornet::new(len);
        }
    }
    /*
        pub fn is_from(&mut self, tkr_id: u32, port: Port) -> bool {
            self.port == port && self.tkr.id() == tkr_id
        }
    */
}
