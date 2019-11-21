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

use gpplugin::audio_format::AudioFormat;
use gpplugin::ear;
use gpplugin::ear::Ear;
use gpplugin::talker;
use gpplugin::talker::{Talker, TalkerBase};
use gpplugin::voice::Voice;

use lv2::core::FeatureBuffer;

pub struct Lv2Talker<'a> {
    base: talker::TalkerBase,
    instance: PluginInstance<'a>,
    inputPortHandlers: Vec<u32>,
    outputPortHandlers: Vec<u32>,
}

impl<'a> Lv2Talker<'a> {
    pub fn new<'w>(
        world: &'w World,
        features: &'w FeatureBuffer<'a>,
        uri: &String,
    ) -> Result<Lv2Talker<'a>, failure::Error> {
        let plugin = world.get_plugin_by_uri(uri.as_str()).unwrap();

        match plugin.resolve(features) {
            Ok(p) => match p.instantiate(AudioFormat::sample_rate() as f64) {
                Ok(mut instance) => {
                    let mut base = TalkerBase::new();
                    let mut inputPortHandlers = Vec::new();
                    let mut outputPortHandlers = Vec::new();

                    for port in plugin.inputs() {
                        match UnknownInputPort::as_typed::<Control>(&port) {
                            Some(p) => {
                                let w = ear::mk_word(Some(p.name().to_string()), None);
                                instance.connect_port(p.handle().clone(), w.value.clone());
                                inputPortHandlers.push(p.handle().index());
                                base.add_ear(Ear::EWord(w));
                            }
                            None => match UnknownInputPort::as_typed::<Audio>(&port) {
                                Some(p) => {
                                    let t = ear::mk_talk(Some(p.name().to_string()), None, None);
                                    inputPortHandlers.push(p.handle().index());
                                    base.add_ear(Ear::ETalk(t));
                                }
                                None => {
                                    eprintln!("Unmanaged input port type");
                                }
                            },
                        }
                    }

                    for port in plugin.outputs() {
                        match UnknownOutputPort::as_typed::<Audio>(&port) {
                            Some(p) => {
                                let vc = Voice::init(p.name().to_string());
                                base.add_voice(vc);
                                outputPortHandlers.push(p.handle().index());
                            }
                            None => {
                                eprintln!("Unmanaged output port type");
                            }
                        }
                    }
                    Ok(Self {
                        base,
                        instance,
                        inputPortHandlers,
                        outputPortHandlers,
                    })
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
