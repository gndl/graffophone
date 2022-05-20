use crate::audio_format::AudioFormat;
use crate::horn::{
    AudioVal, ControlVal, CvVal, Horn, MAtomBuf, MAudioBuf, MControlBuf, MCvBuf, PortType,
};
use crate::lv2_handler::Lv2Handler;
use std::cell::Cell;

pub const DEF_OUTPUT_TAG: &'static str = "Out";

pub struct Voice {
    tag: String,
    tick: Cell<i64>,
    len: Cell<usize>,
    horn: Horn,
}

impl Voice {
    pub fn new(tag: Option<&str>, len: usize, horn: Horn) -> Self {
        Self {
            tag: tag.unwrap_or(DEF_OUTPUT_TAG).to_string(),
            tick: Cell::new(-1),
            len: Cell::new(len),
            horn,
        }
    }
    pub fn port_type(&self) -> PortType {
        self.horn.port_type()
    }
    pub fn tag<'a>(&'a self) -> &'a String {
        &self.tag
    }
    pub fn tick(&self) -> i64 {
        self.tick.get()
    }
    pub fn set_tick(&self, tick: i64) {
        self.tick.set(tick);
    }
    pub fn len(&self) -> usize {
        self.len.get()
    }
    pub fn set_len(&self, len: usize) {
        self.len.set(len);
    }
    pub fn set_tick_len(&self, tick: i64, len: usize) {
        self.tick.set(tick);
        self.len.set(len);
    }

    pub fn horn<'a>(&'a self) -> &'a Horn {
        &self.horn
    }

    pub fn audio_buffer(&self) -> MAudioBuf {
        self.horn.audio_buffer()
    }
    pub fn control_buffer(&self) -> MControlBuf {
        self.horn.control_buffer()
    }
    pub fn cv_buffer(&self) -> MCvBuf {
        self.horn.cv_buffer()
    }
    pub fn atom_buffer(&self) -> MAtomBuf {
        self.horn.atom_buffer()
    }
    pub fn audio_value(&self, index: usize) -> AudioVal {
        self.horn.audio_value(index)
    }
    pub fn control_value(&self) -> ControlVal {
        self.horn.control_value()
    }
    pub fn set_control_value(&self, value: ControlVal) {
        self.horn.set_control_value(value)
    }
    pub fn cv_value(&self, index: usize) -> CvVal {
        self.horn.cv_value(index)
    }
    pub fn can_have_a_value(&self) -> bool {
        self.horn.port_type() != PortType::Atom
    }
    pub fn value(&self, index: usize) -> f32 {
        self.horn.value(index)
    }
}

pub fn audio(tag: Option<&str>, value: f32) -> Voice {
    let len = AudioFormat::chunk_size();
    Voice::new(tag, len, Horn::audio(value, None))
}

pub fn control(tag: Option<&str>, value: f32) -> Voice {
    Voice::new(tag, 1, Horn::control(value))
}

pub fn cv(tag: Option<&str>, value: f32) -> Voice {
    let len = AudioFormat::chunk_size();
    Voice::new(tag, len, Horn::cv(value, None))
}

pub fn atom(tag: Option<&str>, olv2_handler: Option<&Lv2Handler>) -> Voice {
    let len = AudioFormat::chunk_size();
    Voice::new(tag, len, Horn::atom(olv2_handler))
}
