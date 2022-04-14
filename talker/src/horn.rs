use std::cell::Cell;
use std::fmt;

use livi::event::LV2AtomSequence;

use crate::audio_format::AudioFormat;
use crate::lv2_handler;
use crate::lv2_handler::Lv2Handler;

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum PortType {
    Audio,
    Control,
    Cv,
    Atom,
}
impl PortType {
    pub fn can_hear(&self, port_type: PortType) -> bool {
        match (port_type, self) {
            (PortType::Audio, PortType::Audio) => true,
            (_, PortType::Cv) => true,
            (_, PortType::Control) => true,
            (PortType::Atom, PortType::Atom) => true,
            _ => false,
        }
    }
    pub fn to_horn(&self) -> Horn {
        match self {
            PortType::Audio => Horn::audio(0., None),
            PortType::Control => Horn::control(0.),
            PortType::Cv => Horn::cv(0., None),
            PortType::Atom => Horn::atom(None),
        }
    }
    pub fn to_string(&self) -> &str {
        match self {
            PortType::Audio => "Audio",
            PortType::Control => "Control",
            PortType::Cv => "Cv",
            PortType::Atom => "Atom",
        }
    }
}

impl fmt::Display for PortType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

pub type AudioVal = f32;
pub type AudioBuf<'a> = &'a [AudioVal];
pub type MAudioBuf<'a> = &'a mut [AudioVal];

pub type ControlVal = f32;
pub type ControlBuf<'a> = &'a [ControlVal];
pub type MControlBuf<'a> = &'a mut [ControlVal];

pub type CvVal = f32;
pub type CvBuf<'a> = &'a [CvVal];
pub type MCvBuf<'a> = &'a mut [CvVal];

pub type AtomBuf<'a> = &'a LV2AtomSequence;
pub type MAtomBuf<'a> = &'a mut LV2AtomSequence;

pub type HBuf = Cell<Vec<f32>>;
pub type HAudioBuf = HBuf;
pub type HControlBuf = HBuf;
pub type HCvBuf = HBuf;
pub type HAtomBuf = Option<Cell<LV2AtomSequence>>;

fn buf_val(value: f32, default: f32) -> f32 {
    if value.is_nan() {
        default
    } else {
        value
    }
}
fn buf_len(olen: Option<usize>) -> usize {
    olen.unwrap_or(AudioFormat::chunk_size())
}
pub fn empty_buf() -> Cell<Vec<f32>> {
    Cell::new(Vec::new())
}
pub fn empty_audio_buf() -> HAudioBuf {
    Cell::new(Vec::new())
}

const ATOM_CAPACITY: usize = 64;

pub struct Horn {
    port_type: PortType,
    buf: HBuf,
    atom: HAtomBuf,
}

impl Horn {
    pub fn audio(value: AudioVal, len: Option<usize>) -> Horn {
        Horn {
            port_type: PortType::Audio,
            buf: Horn::audio_buf(value, len),
            atom: None,
        }
    }

    pub fn control(value: ControlVal) -> Horn {
        Horn {
            port_type: PortType::Control,
            buf: Horn::control_buf(value),
            atom: None,
        }
    }

    pub fn cv(value: CvVal, len: Option<usize>) -> Horn {
        Horn {
            port_type: PortType::Cv,
            buf: Horn::cv_buf(value, len),
            atom: None,
        }
    }

    pub fn atom(olv2_handler: Option<&Lv2Handler>) -> Horn {
        Horn {
            port_type: PortType::Atom,
            buf: empty_buf(),
            atom: Horn::atom_buf(olv2_handler),
        }
    }

    pub fn port_type(&self) -> PortType {
        self.port_type
    }

    pub fn audio_buffer(&self) -> MAudioBuf {
        unsafe { self.buf.as_ptr().as_mut().unwrap().as_mut_slice() }
    }
    pub fn control_buffer(&self) -> MControlBuf {
        unsafe { self.buf.as_ptr().as_mut().unwrap().as_mut_slice() }
    }
    pub fn cv_buffer(&self) -> MCvBuf {
        unsafe { self.buf.as_ptr().as_mut().unwrap().as_mut_slice() }
    }
    pub fn atom_buffer(&self) -> MAtomBuf {
        unsafe { &mut *self.atom.as_ref().unwrap().as_ptr().as_mut().unwrap() }
    }

    pub fn audio_value(&self, index: usize) -> AudioVal {
        self.audio_buffer()[index]
    }
    pub fn control_value(&self) -> ControlVal {
        self.control_buffer()[0]
    }
    pub fn set_control_value(&self, value: ControlVal) {
        let cb = self.control_buffer();
        if value != cb[0] {
            cb.fill(value);
        }
    }
    pub fn cv_value(&self, index: usize) -> CvVal {
        self.cv_buffer()[index]
    }

    pub fn value(&self, index: usize) -> f32 {
        match self.port_type {
            PortType::Audio => self.audio_value(index),
            PortType::Control => self.control_value(),
            PortType::Cv => self.cv_value(index),
            PortType::Atom => 0.,
        }
    }

    pub fn audio_buf(value: AudioVal, olen: Option<usize>) -> HAudioBuf {
        Cell::new(vec![buf_val(value, 0.); buf_len(olen)])
    }

    pub fn control_buf(value: ControlVal) -> HControlBuf {
        Cell::new(vec![buf_val(value, 1.); buf_len(None)])
    }

    pub fn cv_buf(value: CvVal, olen: Option<usize>) -> HCvBuf {
        Cell::new(vec![buf_val(value, 0.); buf_len(olen)])
    }

    pub fn atom_buf(olv2_handler: Option<&Lv2Handler>) -> HAtomBuf {
        match olv2_handler {
            Some(lv2_handler) => Some(Cell::new(LV2AtomSequence::new(
                &lv2_handler.features,
                ATOM_CAPACITY,
            ))),
            None => lv2_handler::visit(|lv2_handler| {
                Ok(Some(Cell::new(LV2AtomSequence::new(
                    &lv2_handler.features,
                    ATOM_CAPACITY,
                ))))
            })
            .unwrap_or(None),
        }
    }
}
