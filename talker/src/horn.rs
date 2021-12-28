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

impl Horn {
    pub fn audio_buffer(&self) -> Option<AudioBuf> {
        match self {
            Horn::Audio(b) => Some(b.clone()),
            _ => None,
        }
    }
    pub fn control_buffer(&self) -> Option<ControlBuf> {
        match self {
            Horn::Control(b) => Some(b.clone()),
            _ => None,
        }
    }
    pub fn cv_buffer(&self) -> Option<CvBuf> {
        match self {
            Horn::Cv(b) => Some(b.clone()),
            Horn::Audio(b) => Some(b.clone()),
            _ => None,
        }
    }
    pub fn audio_value(&self, index: usize) -> Option<f32> {
        match self {
            Horn::Audio(b) => Some(b.get()[index].get()),
            _ => None,
        }
    }
    pub fn control_value(&self, _index: usize) -> Option<f32> {
        match self {
            Horn::Control(b) => Some(b.get()),
            _ => None,
        }
    }
    pub fn cv_value(&self, index: usize) -> Option<f32> {
        match self {
            Horn::Cv(b) => Some(b.get()[index].get()),
            _ => None,
        }
    }
}

pub fn audio_buf(value: f32, len: Option<usize>) -> AudioBuf {
    Rc::new(VecBuffer::new(
        if value.is_nan() { 0. } else { value },
        len.unwrap_or(AudioFormat::chunk_size()),
    ))
}

pub fn control_buf(value: f32) -> ControlBuf {
    Rc::new(CellBuffer::new(if value.is_nan() { 1. } else { value }))
}

pub fn cv_buf(value: f32, len: Option<usize>) -> CvBuf {
    Rc::new(VecBuffer::new(
        if value.is_nan() { 0. } else { value },
        len.unwrap_or(AudioFormat::chunk_size()),
    ))
}
pub fn audio(value: f32, len: Option<usize>) -> Horn {
    Horn::Audio(audio_buf(value, len))
}

pub fn control(value: f32) -> Horn {
    Horn::Control(control_buf(value))
}

pub fn cv(value: f32, len: Option<usize>) -> Horn {
    Horn::Cv(cv_buf(value, len))
}
