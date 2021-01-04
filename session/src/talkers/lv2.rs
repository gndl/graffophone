use std::cell::RefCell;
use std::rc::Rc;

use talker::audio_format::AudioFormat;
use talker::ear;
use talker::horn;
use talker::horn::{AudioBuf, ControlBuf, CvBuf, Horn};
use talker::talker;
use talker::talker::{RTalker, Talker, TalkerBase};
use talker::voice;

use crate::lv2_resources::Lv2Resources;

fn instantiate(
    plugin: &lilv::Plugin,
    features: *const *const lv2_raw::LV2Feature,
) -> Result<lilv::Instance, failure::Error> {
    unsafe {
        match plugin.instantiate(AudioFormat::sample_rate() as f64, features) {
            Some(instance) => Ok(instance),
            None => Err(failure::err_msg("PluginInstantiationError")),
        }
    }
}

pub struct Lv2 {
    base: talker::TalkerBase,
    model: String,
    instance: lilv::Instance,
    input_port_handlers: Vec<usize>,
    output_port_handlers: Vec<usize>,
}

impl Lv2 {
    pub fn new(
        lv2_resources: &Lv2Resources,
        features: *const *const lv2_raw::LV2Feature,
        plugin: &lilv::Plugin,
    ) -> Result<RTalker, failure::Error> {
        show_plugin(plugin);

        let mut instance = instantiate(plugin, features)?;

        let name_node = plugin.name();
        let name_turtle_token = name_node.turtle_token();
        let name = name_node.as_str().unwrap_or(&name_turtle_token);
        let uri_node = plugin.uri();
        let uri_turtle_token = uri_node.turtle_token();
        let model = uri_node.as_str().unwrap_or(&uri_turtle_token);

        let mut base = TalkerBase::new(name, model);
        let mut input_port_handlers = Vec::new();
        let mut output_port_handlers = Vec::new();

        let num_ports = plugin.num_ports();
        let mut mins = vec![f32::NAN; num_ports];
        let mut maxes = vec![f32::NAN; num_ports];
        let mut defaults = vec![f32::NAN; num_ports];

        plugin
            .port_ranges_float(
                mins.as_mut_slice(),
                maxes.as_mut_slice(),
                defaults.as_mut_slice(),
            )
            .map_err(|_| failure::err_msg(format!("Lv2 plugin port ranges error")))?;

        for port_index in 0..num_ports {
            unsafe { instance.connect_port(port_index, &mut (std::ptr::null_mut() as *mut f32)) };
            match plugin.port_by_index(port_index) {
                Some(port) => {
                    let port_symbol = port.symbol();
                    let port_tag = port_symbol.as_str();

                    let default = if defaults[port_index].is_nan() {
                        None
                    } else {
                        Some(defaults[port_index])
                    };

                    if lv2_resources.is_input(&port) {
                        if lv2_resources.is_audio(&port) {
                            let ear = ear::audio(port_tag, default, None);
                            base.add_ear(ear);
                            input_port_handlers.push(port_index);
                        } else if lv2_resources.is_control(&port) {
                            let ear = ear::control(port_tag, default);
                            base.add_ear(ear);
                            input_port_handlers.push(port_index);
                        } else if lv2_resources.is_atom(&port) {
                            let ear = ear::cv(port_tag, default, None);
                            base.add_ear(ear);
                            input_port_handlers.push(port_index);
                        } else {
                            eprintln!("Unmanaged input port type");
                        }
                    } else {
                        if lv2_resources.is_audio(&port) {
                            let mut buf = horn::audio_buf(None, None);
                            unsafe {
                                instance.connect_port(
                                    port_index,
                                    &mut buf.borrow_mut().as_mut_slice().as_mut_ptr(),
                                )
                            };

                            let vc = voice::audio(port_tag, None, Some(buf));
                            base.add_voice(vc);
                            output_port_handlers.push(port_index);
                        } else if lv2_resources.is_control(&port) {
                            let mut buf = horn::control_buf(None);
                            unsafe {
                                instance.connect_port(
                                    port_index,
                                    &mut buf.borrow_mut().as_mut_slice().as_mut_ptr(),
                                )
                            };

                            let vc = voice::control(port_tag, None, Some(buf));
                            base.add_voice(vc);
                            output_port_handlers.push(port_index);
                        } else if lv2_resources.is_atom(&port) {
                            let mut buf = horn::cv_buf(None, None);
                            unsafe {
                                instance.connect_port(
                                    port_index,
                                    &mut buf.borrow_mut().as_mut_slice().as_mut_ptr(),
                                )
                            };

                            let vc = voice::cv(port_tag, None, Some(buf));
                            base.add_voice(vc);
                            output_port_handlers.push(port_index);
                        } else {
                            eprintln!("Unmanaged output port type");
                        }
                    }
                }
                None => eprintln!("Unmanaged port index"),
            }
        }

        Ok(Rc::new(RefCell::new(Self {
            base,
            model: model.to_string(),
            instance,
            input_port_handlers,
            output_port_handlers,
        })))
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
        let mut audio_buffers: Vec<(usize, AudioBuf)> = Vec::new();
        let mut control_buffers: Vec<(usize, ControlBuf)> = Vec::new();
        let mut cv_buffers: Vec<(usize, CvBuf)> = Vec::new();

        for (i, ear) in self.ears().iter().enumerate() {
            if let Some(port_index) = self.input_port_handlers.get(i) {
                ear.visit_horn(
                    0,
                    |horn, _| match horn {
                        Horn::Audio(buf) => {
                            audio_buffers.push((*port_index, buf.clone()));
                        }
                        Horn::Control(buf) => {
                            control_buffers.push((*port_index, buf.clone()));
                        }
                        Horn::Cv(buf) => {
                            cv_buffers.push((*port_index, buf.clone()));
                        }
                    },
                    (),
                );
            }
        }

        for (port_index, buf) in audio_buffers {
            unsafe {
                self.instance.connect_port(
                    port_index,
                    &mut buf.borrow_mut().as_mut_slice().as_mut_ptr(),
                )
            };
        }
        for (port_index, buf) in control_buffers {
            unsafe {
                self.instance.connect_port(
                    port_index,
                    &mut &buf.borrow_mut().as_mut_slice().as_mut_ptr(),
                )
            };
        }
        for (port_index, buf) in cv_buffers {
            unsafe {
                self.instance.connect_port(
                    port_index,
                    &mut &buf.borrow_mut().as_mut_slice().as_mut_ptr(),
                )
            };
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
        self.instance.run(ln);
        ln
    }
}

fn show_plugin(plugin: &lilv::Plugin) {
    //    println!("> {:?}", plugin);
    /*
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
    */
}
