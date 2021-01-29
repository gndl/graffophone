use lilv::instance::PluginInstance;
use lilv::plugin::Plugin;
use lilv::port::Port;
use lilv::port::TypedPort;
use lilv::port::{UnknownInputPort, UnknownOutputPort};
use lilv::world::World;
use lv2::core::ports::{Audio, Control, CV};
use std::cell::RefCell;
use std::rc::Rc;

use talker::audio_format::AudioFormat;
use talker::ear;
use talker::horn;
use talker::horn::{AudioBuf, ControlBuf, CvBuf, Horn};
use talker::talker;
use talker::talker::{RTalker, Talker, TalkerBase};
use talker::voice;

use lv2::core::SharedFeatureBuffer;

pub struct Lv2 {
    base: talker::TalkerBase,
    model: String,
    instance: PluginInstance,
    input_port_handlers: Vec<u32>,
    output_port_handlers: Vec<u32>,
}

impl Lv2 {
    pub fn new(
        world: &World,
        features: SharedFeatureBuffer,
        uri: &str,
    ) -> Result<RTalker, failure::Error> {
        let plugin = world.get_plugin_by_uri(uri).unwrap();
        show_plugin(&plugin);
        match PluginInstance::new(&plugin, AudioFormat::sample_rate() as f64, features) {
            Ok(mut instance) => {
                let mut base = TalkerBase::new(plugin.name().to_str(), uri);
                let mut input_port_handlers = Vec::new();
                let mut output_port_handlers = Vec::new();

                let (min_values, max_values, def_values) = plugin.all_port_ranges_float();

                for port in plugin.inputs() {
                    let port_index = port.index() as usize;
                    let min_val = min_values[port_index];
                    let max_val = max_values[port_index];
                    let def_val = def_values[port_index];

                    match UnknownInputPort::as_typed::<Control>(&port) {
                        Some(p) => {
                            let ear = ear::control(
                                Some(&p.name().to_string()),
                                min_val,
                                max_val,
                                def_val,
                            );
                            base.add_ear(ear);
                            input_port_handlers.push(p.handle().index());
                        }
                        None => match UnknownInputPort::as_typed::<Audio>(&port) {
                            Some(p) => {
                                let ear = ear::audio(
                                    Some(&p.name().to_string()),
                                    min_val,
                                    max_val,
                                    def_val,
                                    None,
                                );
                                base.add_ear(ear);
                                input_port_handlers.push(p.handle().index());
                            }
                            None => match UnknownInputPort::as_typed::<CV>(&port) {
                                Some(p) => {
                                    let ear = ear::cv(
                                        Some(&p.name().to_string()),
                                        min_val,
                                        max_val,
                                        def_val,
                                        None,
                                    );
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
                            let buf = horn::audio_buf(0., None);
                            instance.connect_port(p.handle().clone(), buf.clone());
                            let vc = voice::audio(Some(&p.name().to_string()), 0., Some(buf));
                            base.add_voice(vc);
                            output_port_handlers.push(p.handle().index());
                        }
                        None => match UnknownOutputPort::as_typed::<Control>(&port) {
                            Some(p) => {
                                let buf = horn::control_buf(0.);
                                instance.connect_port(p.handle().clone(), buf.clone());
                                let vc = voice::control(Some(&p.name().to_string()), 0., Some(buf));
                                base.add_voice(vc);
                                output_port_handlers.push(p.handle().index());
                            }
                            None => match UnknownOutputPort::as_typed::<CV>(&port) {
                                Some(p) => {
                                    let buf = horn::cv_buf(0., None);
                                    instance.connect_port(p.handle().clone(), buf.clone());
                                    let vc = voice::cv(Some(&p.name().to_string()), 0., Some(buf));
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
                    model: uri.to_string(), //plugin.name().to_str().to_string(),
                    instance,
                    input_port_handlers,
                    output_port_handlers,
                })))
            }
            _ => Err(failure::err_msg("PluginInstantiationError")),
        }
    }
}

impl Talker for Lv2 {
    fn base<'b>(&'b self) -> &'b TalkerBase {
        &self.base
    }
    fn model(&self) -> &str {
        self.model.as_str()
    }
    fn activate(&mut self) {
        let mut audio_buffers: Vec<(&u32, AudioBuf)> = Vec::new();
        let mut control_buffers: Vec<(&u32, ControlBuf)> = Vec::new();
        let mut cv_buffers: Vec<(&u32, CvBuf)> = Vec::new();

        for (i, ear) in self.ears().iter().enumerate() {
            if let Some(port_index) = self.input_port_handlers.get(i) {
                ear.visit_horn(|horn| match horn {
                    Horn::Audio(buf) => {
                        audio_buffers.push((port_index, buf.clone()));
                    }
                    Horn::Control(buf) => {
                        control_buffers.push((port_index, buf.clone()));
                    }
                    Horn::Cv(buf) => {
                        cv_buffers.push((port_index, buf.clone()));
                    }
                });
            }
        }
        for (port_index, buf) in audio_buffers {
            self.instance
                .connect_port_index::<Audio, AudioBuf>(*port_index, buf.clone());
        }
        for (port_index, buf) in control_buffers {
            self.instance
                .connect_port_index::<Control, ControlBuf>(*port_index, buf.clone());
        }
        for (port_index, buf) in cv_buffers {
            self.instance
                .connect_port_index::<CV, CvBuf>(*port_index, buf.clone());
        }
        self.instance.activate()
    }
    fn deactivate(&mut self) {
        self.instance.deactivate()
    }

    fn talk(&mut self, _port: usize, tick: i64, len: usize) -> usize {
        let mut ln = len;

        for ear in self.ears() {
            ln = ear.listen(tick, ln);
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
