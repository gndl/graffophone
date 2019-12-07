use crate::audio_format::AudioFormat;
use lilv::port::buffer::CellBuffer;
use lilv::port::buffer::VecBuffer;
use std::rc::Rc;

pub type AudioBuf = Rc<VecBuffer<f32>>;
pub type ControlBuf = Rc<CellBuffer<f32>>;
pub type CvBuf = Rc<VecBuffer<f32>>;

pub enum Horn {
    Audio(AudioBuf),
    Control(ControlBuf),
    Cv(CvBuf),
}

pub fn audio_buf(value: Option<f32>, len: Option<usize>) -> AudioBuf {
    Rc::new(VecBuffer::new(
        value.unwrap_or(0.),
        len.unwrap_or(AudioFormat::chunk_size()),
    ))
}

pub fn control_buf(value: Option<f32>) -> ControlBuf {
    Rc::new(CellBuffer::new(value.unwrap_or(0.)))
}

pub fn cv_buf(value: Option<f32>, len: Option<usize>) -> CvBuf {
    Rc::new(VecBuffer::new(
        value.unwrap_or(0.),
        len.unwrap_or(AudioFormat::chunk_size()),
    ))
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
