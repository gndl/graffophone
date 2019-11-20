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
use lilv::port::Port;
use lilv::port::TypedPort;
use lilv::port::{UnknownInputPort, UnknownOutputPort};
use lilv::world::World;
use lv2::core::ports::Audio;
use lv2::core::ports::Control;

use gpplugin::audio_format;
use gpplugin::ear;
use gpplugin::ear::Ear;
use gpplugin::talker;
use gpplugin::talker::{Talker, TalkerBase};

use lv2::core::FeatureBuffer;

pub struct Lv2Talker<'a> {
    base: talker::TalkerBase,
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
                Ok(mut instance) => {
                    let mut base = TalkerBase::new();

                    for port in plugin.inputs() {
                        match UnknownInputPort::as_typed::<Control>(&port) {
                            Some(cp) => {
                                let w = ear::mk_word(Some(cp.name().to_string()), None);
                                instance.connect_port(cp.handle().clone(), w.value.clone());
                                base.add_ear(Ear::EWord(w));
                            }
                            None => match UnknownInputPort::as_typed::<Audio>(&port) {
                                Some(ap) => {
                                    let t = ear::mk_talk(Some(ap.name().to_string()), None, None);
                                    base.add_ear(Ear::ETalk(t));
                                }
                                None => {
                                    eprintln!("Type port inconnu");
                                }
                            },
                        }
                    }
                    Ok(Self { base, instance })
                }
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
