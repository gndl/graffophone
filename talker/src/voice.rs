use crate::audio_format::AudioFormat;
use crate::horn;
use crate::horn::{AudioBuf, ControlBuf, CvBuf, Horn};
use std::cell::RefCell;

pub const DEF_OUTPUT_TAG: &'static str = "Out";

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum PortType {
    Audio,
    Control,
    Cv,
}
impl PortType {
    pub fn can_hear(&self, port_type: PortType) -> bool {
        match (port_type, self) {
            (PortType::Audio, PortType::Audio) => true,
            (_, PortType::Cv) => true,
            (_, PortType::Control) => true,
            _ => false,
        }
    }
}

pub struct Voice {
    port_type: PortType,
    tag: String,
    tick: i64,
    len: usize,
    horn: Horn,
}

impl Voice {
    pub fn new(port_type: PortType, tag: Option<&str>, len: usize, horn: Horn) -> Self {
        Self {
            port_type,
            tag: tag.unwrap_or(DEF_OUTPUT_TAG).to_string(),
            tick: -1,
            len,
            horn,
        }
    }
    pub fn port_type(&self) -> PortType {
        self.port_type
    }
    pub fn tag<'a>(&'a self) -> &'a String {
        &self.tag
    }
    pub fn tick(&self) -> i64 {
        self.tick
    }
    pub fn set_tick(&mut self, tick: i64) {
        self.tick = tick;
    }
    pub fn len(&self) -> usize {
        self.len
    }
    pub fn set_len(&mut self, len: usize) {
        self.len = len;
    }
    pub fn horn<'a>(&'a self) -> &'a Horn {
        &self.horn
    }
    pub fn audio_buffer(&self) -> Option<AudioBuf> {
        match &self.horn {
            Horn::Audio(b) => Some(b.clone()),
            _ => None,
        }
    }
    pub fn control_buffer(&self) -> Option<ControlBuf> {
        match &self.horn {
            Horn::Control(b) => Some(b.clone()),
            _ => None,
        }
    }
    pub fn cv_buffer(&self) -> Option<CvBuf> {
        match &self.horn {
            Horn::Cv(b) => Some(b.clone()),
            Horn::Audio(b) => Some(b.clone()),
            _ => None,
        }
    }
    pub fn audio_value(&self, index: usize) -> Option<f32> {
        match &self.horn {
            Horn::Audio(b) => Some(b.get()[index].get()),
            _ => None,
        }
    }
    pub fn control_value(&self, _index: usize) -> Option<f32> {
        match &self.horn {
            Horn::Control(b) => Some(b.get()),
            _ => None,
        }
    }
    pub fn cv_value(&self, index: usize) -> Option<f32> {
        match &self.horn {
            Horn::Cv(b) => Some(b.get()[index].get()),
            _ => None,
        }
    }
}

pub type MVoice = RefCell<Voice>;

pub fn audio(tag: Option<&str>, value: f32, buf: Option<AudioBuf>) -> MVoice {
    let len = AudioFormat::chunk_size();
    RefCell::new(Voice::new(
        PortType::Audio,
        tag,
        len,
        Horn::Audio(buf.unwrap_or(horn::audio_buf(value, Some(len)))),
    ))
}

pub fn control(tag: Option<&str>, value: f32, buf: Option<ControlBuf>) -> MVoice {
    RefCell::new(Voice::new(
        PortType::Control,
        tag,
        1,
        Horn::Control(buf.unwrap_or(horn::control_buf(value))),
    ))
}

pub fn cv(tag: Option<&str>, value: f32, buf: Option<CvBuf>) -> MVoice {
    let len = AudioFormat::chunk_size();
    RefCell::new(Voice::new(
        PortType::Cv,
        tag,
        len,
        Horn::Cv(buf.unwrap_or(horn::cv_buf(value, Some(len)))),
    ))
}
