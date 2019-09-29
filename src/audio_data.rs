extern crate failure;

//const VECTOR_SIZE: usize = 960;

//pub type Vector = [f32; VECTOR_SIZE];
pub type Vector = Vec<f32>;

pub struct Interleaved {
    channels: usize,
    samples: usize,
    vector: Vector,
    is_end: bool,
}

impl Interleaved {
    pub fn new(channels: usize, samples: usize, vec: &Vector) -> Self {
        Self {
            channels: channels,
            samples: samples,
            vector: vec.to_vec(),
            is_end: false,
        }
    }
    pub fn end() -> Self {
        Self {
            channels: 0,
            samples: 0,
            vector: Vec::new(),
            is_end: true,
        }
    }
    pub fn channels(&self) -> usize {
        self.channels
    }
    pub fn samples(&self) -> usize {
        self.samples
    }
    pub fn vector(&self) -> Vector {
        self.vector.to_vec()
    }
    pub fn is_end(&self) -> bool {
        self.is_end
    }
}

pub trait AudioOutput {
    fn open(&self) -> Result<(), failure::Error>;
    fn write(&self, data: Interleaved) -> Result<(), failure::Error>;
    fn close(&self) -> Result<(), failure::Error>;
}
