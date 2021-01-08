use std::cell::RefCell;
use std::rc::Rc;

use crate::audio_format::AudioFormat;

pub type AudioVal = f32;
pub type ControlVal = f32;
pub type CvVal = f32;
pub type AudioBuf = Rc<RefCell<Vec<AudioVal>>>;
pub type ControlBuf = Rc<RefCell<Vec<ControlVal>>>;
pub type CvBuf = Rc<RefCell<Vec<CvVal>>>;

pub enum Horn {
    Audio(AudioBuf),
    Control(ControlBuf),
    Cv(CvBuf),
}

pub fn audio_buf(value: Option<AudioVal>, len: Option<usize>) -> AudioBuf {
    Rc::new(RefCell::new(vec![
        value.unwrap_or(0.);
        len.unwrap_or(AudioFormat::chunk_size())
    ]))
}

pub fn control_buf(value: Option<ControlVal>) -> ControlBuf {
    Rc::new(RefCell::new(vec![value.unwrap_or(0.); 1]))
}

pub fn cv_buf(value: Option<CvVal>, len: Option<usize>) -> CvBuf {
    Rc::new(RefCell::new(vec![
        value.unwrap_or(0.);
        len.unwrap_or(AudioFormat::chunk_size())
    ]))
}
pub fn audio(value: Option<AudioVal>, len: Option<usize>) -> Horn {
    Horn::Audio(audio_buf(value, len))
}

pub fn control(value: Option<ControlVal>) -> Horn {
    Horn::Control(control_buf(value))
}

pub fn cv(value: Option<CvVal>, len: Option<usize>) -> Horn {
    Horn::Cv(cv_buf(value, len))
}
