use gpplugin::ear::Ear;
use gpplugin::talker;
use gpplugin::talker::{Base, Talker};

pub struct LilvTalker {
base: talker::Base,
ears: Vec<Ear>,
}
impl LilvTalker {
    pub fn new() -> Result<Self, failure::Error> {
        Ok(Self {
base:     gpplugin::talker::Base::new(),
ears: Vec::new(),
        })
    }
}

impl Talker for LilvTalker {
    fn base<'a>(&'a self) -> &'a Base{&self.base}
    fn depends_of(&self, id: u32) -> bool{true}
    fn get_ears<'a>(&'a self) -> &'a Vec<Ear>{&self.ears}
}
