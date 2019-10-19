use lilv::plugin::Plugin;
use lilv::world::World;
use lilv::port::buffer::CellBuffer;
use lilv::port::buffer::VecBuffer;
use lilv::port::Port;
use lilv::port::TypedPort;
use lilv::port::{UnknownInputPort, UnknownOutputPort};
use lv2::core::ports::Audio;
use lv2::core::ports::Control;

use lv2::core::{Feature, FeatureBuffer, FeatureSet};
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
    let world = World::new().unwrap();
    let plugin = world
        .get_plugin_by_uri(uri.as_str())
        .unwrap();

    show_plugin(&plugin);

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

fn show_plugin(plugin: &Plugin) {
    println!("> {:?}", plugin);
    for port in plugin.inputs() {
        println!("> {:?}", port);
    }
    for port in plugin.outputs() {
        println!("< {:?}", port);
    }
    for port in plugin
        .inputs()
        .filter_map(UnknownInputPort::into_typed::<Audio>)
    {
        println!("\t{:?}", port)
    }
    for port in plugin
        .outputs()
        .filter_map(UnknownOutputPort::into_typed::<Audio>)
    {
        println!("\t{:?}", port)
    }
    for port in plugin
        .inputs()
        .filter_map(UnknownInputPort::into_typed::<Control>)
    {
        println!("\t{:?}", port)
    }
}
