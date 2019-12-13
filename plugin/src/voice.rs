use crate::audio_format::AudioFormat;
use crate::horn;
use crate::horn::{AudioBuf, ControlBuf, CvBuf, Horn};
use std::cell::RefCell;

pub const DEF_OUTPUT_TAG: &'static str = "O";

pub enum PortType {
    Audio,
    Control,
    Cv,
}

pub struct T {
    port_type: PortType,
    tag: String,
    tick: i64,
    len: usize,
    horn: Horn,
}

impl T {
    pub fn new(port_type: PortType, tag: Option<String>, len: usize, horn: Horn) -> Self {
        Self {
            port_type,
            tag: tag.unwrap_or(DEF_OUTPUT_TAG.to_string()),
            tick: -1,
            len,
            horn,
        }
    }
    pub fn port_type<'a>(&'a self) -> &'a PortType {
        &self.port_type
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
}

pub type Voice = RefCell<T>;

pub fn audio(tag: Option<String>, value: Option<f32>, buf: Option<AudioBuf>) -> Voice {
    let len = AudioFormat::chunk_size();
    RefCell::new(T::new(
        PortType::Audio,
        tag,
        len,
        Horn::Audio(buf.unwrap_or(horn::audio_buf(value, Some(len)))),
    ))
}

pub fn control(tag: Option<String>, value: Option<f32>, buf: Option<ControlBuf>) -> Voice {
    RefCell::new(T::new(
        PortType::Control,
        tag,
        1,
        Horn::Control(buf.unwrap_or(horn::control_buf(value))),
    ))
}

pub fn cv(tag: Option<String>, value: Option<f32>, buf: Option<CvBuf>) -> Voice {
    let len = AudioFormat::chunk_size();
    RefCell::new(T::new(
        PortType::Cv,
        tag,
        len,
        Horn::Cv(buf.unwrap_or(horn::cv_buf(value, Some(len)))),
    ))
}

/*
pub fn control(tag: Option<String>, value: Option<f32>) -> ControlVoice {
    //Rc::new(
    Cell::new(ControlVoiceT::new(tag, value))
    //)
}

pub fn cv(tag: Option<String>, value: Option<f32>) -> CvVoice {
    //  Rc::new(
    Cell::new(CvVoiceT::new(tag, value))
    //)
}

impl Voice {
    pub fn init(tag: String) -> Self {
        let len = AudioFormat::chunk_size();
        Self {
            tag: tag,
            tick: 0,
            len,
            cor: cornet::new(len),
            new: true,
        }
    }

    pub fn new(tick: i64, len: usize, tag: String) -> Self {
        Self {
            tag: tag,
            tick: tick,
            len: len,
            cor: cornet::new(len),
            new: true,
        }
    }

}
    */
