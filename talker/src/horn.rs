use std::cell::RefCell;
use std::rc::Rc;
/*
use lilv::port::buffer::CellBuffer;
use lilv::port::buffer::VecBuffer;
*/
use crate::audio_format::AudioFormat;

pub type AudioBuf = Rc<RefCell<Vec<f32>>>;
pub type ControlBuf = Rc<RefCell<Vec<f32>>>;
pub type CvBuf = Rc<RefCell<Vec<f32>>>;

pub enum Horn {
    Audio(AudioBuf),
    Control(ControlBuf),
    Cv(CvBuf),
}

pub fn audio_buf(value: Option<f32>, len: Option<usize>) -> AudioBuf {
    Rc::new(RefCell::new(vec![
        value.unwrap_or(0.);
        len.unwrap_or(AudioFormat::chunk_size())
    ]))
}

pub fn control_buf(value: Option<f32>) -> ControlBuf {
    Rc::new(RefCell::new(vec![value.unwrap_or(0.); 1]))
}

pub fn cv_buf(value: Option<f32>, len: Option<usize>) -> CvBuf {
    Rc::new(RefCell::new(vec![
        value.unwrap_or(0.);
        len.unwrap_or(AudioFormat::chunk_size())
    ]))
}
pub fn audio(value: Option<f32>, len: Option<usize>) -> Horn {
    Horn::Audio(audio_buf(value, len))
}

pub fn control(value: Option<f32>) -> Horn {
    Horn::Control(control_buf(value))
}

pub fn cv(value: Option<f32>, len: Option<usize>) -> Horn {
    Horn::Cv(cv_buf(value, len))
}
