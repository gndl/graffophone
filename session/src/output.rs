use std::cell::RefCell;
use std::rc::Rc;
//use std::boxed::Box;

extern crate failure;

use talker::identifier::{Identifier, RIdentifier};

use crate::audio_data::Vector;

pub const KIND: &str = "output";

pub fn new_identifier(name: &str, model: &str) -> RIdentifier {
    RefCell::new(Identifier::new(name, model))
}

pub trait Output {
    fn identifier<'a>(&'a self) -> &'a RIdentifier;

    fn id(&self) -> u32 {
        self.identifier().borrow().id()
    }

    fn model(&self) -> &str;

    fn name(&self) -> String {
        self.identifier().borrow().name().to_string()
    }
    fn set_name(&self, name: &str) {
        self.identifier().borrow_mut().set_name(name);
    }

    fn nb_channels(&self) -> usize;

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
    fn backup(&self) -> (&str, &str, &str);
}

pub type ROutput = Rc<RefCell<dyn Output>>;
//pub type ROutput = Box<dyn Output>;
