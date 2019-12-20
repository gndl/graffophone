use lilv::instance::PluginInstance;
use lilv::plugin::Plugin;
use lilv::port::Port;
use lilv::port::TypedPort;
use lilv::port::{UnknownInputPort, UnknownOutputPort};
use lilv::world::World;
use lv2::core::ports::{Audio, Control, CV};
use std::cell::RefCell;
use std::rc::Rc;

use gpplugin::audio_format::AudioFormat;
use gpplugin::ear;
use gpplugin::ear::Ear;
use gpplugin::horn;
use gpplugin::talker;
use gpplugin::talker::{MTalker, Talker, TalkerBase};
use gpplugin::voice;
use gpplugin::voice::Voice;

use lv2::core::SharedFeatureBuffer;

pub struct Lv2Talker {
    base: talker::TalkerBase,
    model: String,
    instance: PluginInstance,
    input_port_handlers: Vec<u32>,
    output_port_handlers: Vec<u32>,
    activated: bool,
}

impl Lv2Talker {
    pub fn new(
        world: &World,
        features: SharedFeatureBuffer,
        uri: &String,
    ) -> Result<MTalker, failure::Error> {
        let plugin = world.get_plugin_by_uri(uri.as_str()).unwrap();

        match PluginInstance::new(&plugin, AudioFormat::sample_rate() as f64, features) {
            Ok(mut instance) => {
                let mut base = TalkerBase::new();
                let mut input_port_handlers = Vec::new();
                let mut output_port_handlers = Vec::new();

                for port in plugin.inputs() {
                    match UnknownInputPort::as_typed::<Control>(&port) {
                        Some(p) => {
                            let ear = ear::control(Some(p.name().to_string()), None);
                            //                                instance.connect_port(p.handle().clone(), w.value.clone());
                            base.add_ear(ear);
                            input_port_handlers.push(p.handle().index());
                        }
                        None => match UnknownInputPort::as_typed::<Audio>(&port) {
                            Some(p) => {
                                let ear = ear::audio(Some(p.name().to_string()), None, None);
                                base.add_ear(ear);
                                input_port_handlers.push(p.handle().index());
                            }
                            None => match UnknownInputPort::as_typed::<CV>(&port) {
                                Some(p) => {
                                    let ear = ear::cv(Some(p.name().to_string()), None, None);
                                    base.add_ear(ear);
                                    input_port_handlers.push(p.handle().index());
                                }
                                None => {
                                    eprintln!("Unmanaged input port type");
                                }
                            },
                        },
                    }
                }

                for port in plugin.outputs() {
                    match UnknownOutputPort::as_typed::<Audio>(&port) {
                        Some(p) => {
                            let buf = horn::audio_buf(None, None);
                            instance.connect_port(p.handle().clone(), buf.clone());
                            let vc = voice::audio(Some(p.name().to_string()), None, Some(buf));
                            base.add_voice(vc);
                            output_port_handlers.push(p.handle().index());
                        }
                        None => match UnknownOutputPort::as_typed::<Control>(&port) {
                            Some(p) => {
                                let buf = horn::control_buf(None);
                                instance.connect_port(p.handle().clone(), buf.clone());
                                let vc =
                                    voice::control(Some(p.name().to_string()), None, Some(buf));
                                base.add_voice(vc);
                                output_port_handlers.push(p.handle().index());
                            }
                            None => match UnknownOutputPort::as_typed::<CV>(&port) {
                                Some(p) => {
                                    let buf = horn::cv_buf(None, None);
                                    instance.connect_port(p.handle().clone(), buf.clone());
                                    let vc = voice::cv(Some(p.name().to_string()), None, Some(buf));
                                    base.add_voice(vc);
                                    output_port_handlers.push(p.handle().index());
                                }
                                None => {
                                    eprintln!("Unmanaged output port type");
                                }
                            },
                        },
                    }
                }
                Ok(Rc::new(RefCell::new(Self {
                    base,
                    model: plugin.name().to_str().to_string(),
                    instance,
                    input_port_handlers,
                    output_port_handlers,
                    activated: false,
                })))
            }
            _ => Err(failure::err_msg("PluginInstantiationError")),
        }
    }
}

impl Talker for Lv2Talker {
    fn base<'b>(&'b self) -> &'b TalkerBase {
        &self.base
    }
    fn model(&self) -> &str {
        self.model.as_str()
    }

    fn talk(&mut self, _port: usize, tick: i64, len: usize) -> usize {
        let mut ln = len;

        for ear in self.ears() {
            ln = ear::listen(ear, tick, ln);
        }
        self.instance.run(ln as u32);
        ln
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
