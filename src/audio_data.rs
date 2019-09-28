extern crate failure;

const VECTOR_SIZE: usize = 20790;

pub type Vector = [f32; VECTOR_SIZE];

pub struct Interleaved {
    channels: usize,
    samples: usize,
    vector: Vector,
}

impl Interleaved {
    pub fn new(channels: usize, samples: usize) -> Self {
        Self {
            channels: channels,
            samples: samples,
            vector: [0.0; VECTOR_SIZE],
        }
    }
    pub fn channels(&self) -> usize {
        self.channels
    }
    pub fn samples(&self) -> usize {
        self.samples
    }
    pub fn vector(&self) -> Vector {
        self.vector
    }
}

pub trait AudioOutput {
    fn open(&self) -> Result<(), failure::Error>;
    fn write(&self, data: Interleaved) -> Result<(), failure::Error>;
    fn close(&self) -> Result<(), failure::Error>;
}
