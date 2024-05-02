use std::cell::RefCell;
use std::rc::Rc;
//use std::boxed::Box;

extern crate failure;

use talker::identifier::{Identifier, RIdentifier};

use crate::audio_data::Vector;

pub const KIND: &str = "output";

const EMPTY_STR: &str = "";

pub fn new_identifier(name: &str, model: &str) -> RIdentifier {
    RefCell::new(Identifier::new(name, model))
}

pub trait Output {
    fn identifier<'a>(&'a self) -> &'a RIdentifier;

    fn id(&self) -> u32 {
        self.identifier().borrow().id()
    }

    fn model(&self) -> String {
        self.identifier().borrow().model().to_string()
    }

    fn name(&self) -> String {
        self.identifier().borrow().name().to_string()
    }
    fn set_name(&self, name: &str) {
        self.identifier().borrow_mut().set_name(name);
    }

    fn codec_name<'a>(&'a self) -> &'a str {
        EMPTY_STR
    }

    fn sample_rate(&self) -> usize;

    fn channel_layout<'a>(&'a self) -> &'a str;
    
    fn channels(&self) -> usize;

    fn channels_names(&self) -> Vec<&'static str>;

    fn file_path<'a>(&'a self) -> &'a str {
        EMPTY_STR
    }

    fn open(&mut self) -> Result<(), failure::Error>;

    fn write(
        &mut self,
        channels: &Vec<Vector>,
        nb_samples_per_channel: usize,
    ) -> Result<(), failure::Error>;

    fn pause(&mut self) -> Result<(), failure::Error>;

    fn run(&mut self) -> Result<(), failure::Error>;

    fn close(&mut self) -> Result<(), failure::Error>;

    //                           kind  model configuration
    fn backup(&self) -> (&str, &str, String);
}

pub type ROutput = Rc<RefCell<dyn Output>>;
//pub type ROutput = Box<dyn Output>;
