/*
use lilv::port::Port;
use lilv::port::TypedPort;
use lilv::port::buffer::CellBuffer;
use lilv::port::buffer::VecBuffer;
use lilv::instance::{errors::MissingFeatureError, PluginInstance, ResolvedPlugin};
use lv2::core::{Feature, FeatureBuffer, FeatureSet};
use lv2::core::FeatureBuffer;
use std::cell::RefCell;
use std::rc::Rc;
*/

use lilv::instance::PluginInstance;
use lilv::plugin::Plugin;
use lilv::port::{UnknownInputPort, UnknownOutputPort};
use lilv::world::World;
use lv2::core::ports::Audio;
use lv2::core::ports::Control;

use gpplugin::audio_format;
use gpplugin::ear::Ear;
use gpplugin::talker;
use gpplugin::talker::{Talker, TalkerBase};

use lv2::core::FeatureBuffer;

pub struct Lv2Talker<'a> {
    base: talker::TalkerBase,
    ears: Vec<Ear>,
    instance: PluginInstance<'a>,
}

impl<'a> Lv2Talker<'a> {
    pub fn new<'w>(
        world: &'w World,
        features: &'w FeatureBuffer<'a>,
        uri: &String,
    ) -> Result<Lv2Talker<'a>, failure::Error> {
        let plugin = world.get_plugin_by_uri(uri.as_str()).unwrap();

        match plugin.resolve(features) {
            Ok(p) => match p.instantiate(audio_format::sample_rate() as f64) {
                Ok(instance) => Ok(Self {
                    base: TalkerBase::new(),
                    ears: Vec::new(),
                    instance: instance,
                }),
                _ => Err(failure::err_msg("PluginInstantiationError")),
            },
            _ => Err(failure::err_msg("MissingFeatureError")),
        }
    }
}

impl<'a> Talker for Lv2Talker<'a> {
    fn base<'b>(&'b self) -> &'b TalkerBase {
        &self.base
    }
    fn depends_of(&self, _id: u32) -> bool {
        true
    }
    fn ears<'e>(&'e self) -> &'e Vec<Ear> {
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
