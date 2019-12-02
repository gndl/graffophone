use crate::audio_format::AudioFormat;
use crate::horn;
use std::cell::Cell;
use std::rc::Rc;

pub const DEF_OUTPUT_TAG: &'static str = "O";

pub struct VoiceT<T> {
    tag: String,
    tick: i64,
    len: usize,
    horn: T,
    new: bool,
}

impl<T> VoiceT<T> {
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
    pub fn horn<'a>(&'a self) -> &'a T {
        &self.horn
    }
    pub fn is_new(&self) -> bool {
        self.new
    }
}

pub type AudioVoiceT = VoiceT<horn::Audio>;
pub type ControlVoiceT = VoiceT<horn::Control>;
pub type CvVoiceT = VoiceT<horn::Cv>;

impl AudioVoiceT {
    pub fn new(tag: Option<String>, value: Option<f32>) -> Self {
        let len = AudioFormat::chunk_size();
        Self {
            tag: tag.unwrap_or(DEF_OUTPUT_TAG.to_string()),
            tick: 0,
            len,
            horn: horn::audio(value, len),
            new: true,
        }
    }
    pub fn check_length(&mut self, len: usize) {
        if self.horn.len() < len {
            let value = self.horn.get()[0].get();
            self.horn = horn::audio(Some(value), len);
            self.len = len;
            self.new = true;
        } else {
            self.new = false;
        }
    }
}

impl ControlVoiceT {
    pub fn new(tag: Option<String>, value: Option<f32>) -> Self {
        let len = AudioFormat::chunk_size();
        Self {
            tag: tag.unwrap_or(DEF_OUTPUT_TAG.to_string()),
            tick: 0,
            len,
            horn: horn::control(value),
            new: true,
        }
    }
    /*
        pub fn check_length(&mut self, len: usize) {
            if self.horn.len() < len {
                let value = self.horn.get()[0].get();
                self.horn = horn::control(Some(value), len);
                self.len = len;
                self.new = true;
            } else {
                self.new = false;
            }
        }
    */
}
/*
impl CvVoiceT {
    pub fn new(tag: Option<String>, value: Option<f32>) -> Self {
        let len = AudioFormat::chunk_size();
        Self {
            tag: tag.unwrap_or(DEF_OUTPUT_TAG.to_string()),
            tick: 0,
            len,
            horn: horn::cv(value, len),
            new: true,
        }
    }
    pub fn check_length(&mut self, len: usize) {
        if self.horn.len() < len {
            let value = self.horn.get()[0].get();
            self.horn = horn::cv(Some(value), len);
            self.len = len;
            self.new = true;
        } else {
            self.new = false;
        }
    }
}
*/
pub type AudioVoice = Cell<AudioVoiceT>;
pub type ControlVoice = Cell<ControlVoiceT>;
pub type CvVoice = Cell<CvVoiceT>;

pub fn audio(tag: Option<String>, value: Option<f32>) -> AudioVoice {
    //  Rc::new(
    Cell::new(AudioVoiceT::new(tag, value))
    //    )
}

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

pub enum Voice {
    Audio(AudioVoice),
    Control(ControlVoice),
    Cv(CvVoice),
}

/*
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
