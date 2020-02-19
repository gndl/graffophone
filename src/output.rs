// use std::cell::RefCell;
// use std::rc::Rc;
use std::boxed::Box;
extern crate failure;
use crate::audio_data::Vector;

pub const KIND: &str = "output";

pub trait Output {
    fn open(&self) -> Result<(), failure::Error>;

    fn write(
        &self,
        channels: &Vec<Vector>,
        nb_samples_per_channel: usize,
    ) -> Result<(), failure::Error>;

    fn close(&self) -> Result<(), failure::Error>;
}

//pub type ROutput = Rc<RefCell<dyn Output>>;
pub type ROutput = Box<dyn Output>;
