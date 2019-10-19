use lilv::plugin::Plugin;
use lilv::world::World;
use gpplugin::ear::Ear;
use gpplugin::talker;
use gpplugin::talker::{Base, Handler, Talker};

pub struct Lv2Talker {
    base: talker::Base,
    ears: Vec<Ear>,
}
impl Lv2Talker {
    pub fn new(uri: &String) -> Self {
        println!("Lv2Talker plugin uri : {}", uri);
        Self {
            base: gpplugin::talker::Base::new(),
            ears: Vec::new(),
        }
    }
}

impl Talker for Lv2Talker {
    fn base<'a>(&'a self) -> &'a Base {
        &self.base
    }
    fn depends_of(&self, id: u32) -> bool {
        true
    }
    fn get_ears<'a>(&'a self) -> &'a Vec<Ear> {
        &self.ears
    }
}

