use std::boxed::Box;
extern crate failure;

//const VECTOR_SIZE: usize = 960;

//pub type Vector = [f32; VECTOR_SIZE];
pub type Vector = Vec<f32>;

pub struct Interleaved {
    nb_channels: usize,
    nb_samples_per_channel: usize,
    vector: Vector,
    is_end: bool,
}

impl Interleaved {
    pub fn new(channels: &Vec<Vector>, nb_samples_per_channel: usize) -> Self {
        let nb_channels = channels.len();
        //        let mut vector = Vec::with_capacity(nb_channels * nb_samples_per_channel);
        let mut vector = vec![0.; nb_channels * nb_samples_per_channel];

        for (ch_n, ch) in channels.iter().enumerate() {
            for i in 0..nb_samples_per_channel {
                vector[nb_channels * i + ch_n] = ch[i];
            }
        }
        Self {
            nb_channels,
            nb_samples_per_channel,
            vector,
            is_end: false,
        }
    }
    pub fn end() -> Self {
        Self {
            nb_channels: 0,
            nb_samples_per_channel: 0,
            vector: Vec::new(),
            is_end: true,
        }
    }
    pub fn nb_channels(&self) -> usize {
        self.nb_channels
    }
    pub fn nb_samples_per_channel(&self) -> usize {
        self.nb_samples_per_channel
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

    fn write(
        &self,
        channels: &Vec<Vector>,
        nb_samples_per_channel: usize,
    ) -> Result<(), failure::Error>;

    fn close(&self) -> Result<(), failure::Error>;
}

pub type MAudioOutput = Box<dyn AudioOutput>;
