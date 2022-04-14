use std::cell::RefCell;

use livi;

use talker::audio_format;
use talker::audio_format::AudioFormat;
use talker::ctalker;
use talker::ear;
use talker::ear::Init;
use talker::horn::{AtomBuf, AudioBuf, CvBuf, MAtomBuf, MAudioBuf, MCvBuf};
use talker::lv2_handler::Lv2Handler;
use talker::talker::{CTalker, Talker, TalkerBase};
use talker::voice;

fn an_or(v: f32, def: f32) -> f32 {
    if v.is_nan() {
        def
    } else {
        v
    }
}

pub struct Lv2 {
    control_inputs_indexes: Vec<usize>,
    control_outputs_indexes: Vec<usize>,
    audio_inputs_indexes: Vec<usize>,
    audio_outputs_indexes: Vec<usize>,
    atom_sequence_inputs_indexes: Vec<usize>,
    atom_sequence_outputs_indexes: Vec<usize>,
    cv_inputs_indexes: Vec<usize>,
    cv_outputs_indexes: Vec<usize>,
    instance: RefCell<livi::Instance>,
}

impl Lv2 {
    pub fn new(lv2_handler: &Lv2Handler, uri: &str) -> Result<CTalker, failure::Error> {
        match lv2_handler.world.plugin_by_uri(uri) {
            Some(plugin) => {
                match unsafe {
                    plugin.instantiate(
                        lv2_handler.features.clone(),
                        AudioFormat::sample_rate() as f64,
                    )
                } {
                    Ok(instance) => {
                        let mut base = TalkerBase::new(&plugin.name(), uri);
                        let mut inputs_count = 0;
                        let mut outputs_count = 0;
                        let mut control_inputs_indexes = Vec::new();
                        let mut control_outputs_indexes = Vec::new();
                        let mut audio_inputs_indexes = Vec::new();
                        let mut audio_outputs_indexes = Vec::new();
                        let mut atom_sequence_inputs_indexes = Vec::new();
                        let mut atom_sequence_outputs_indexes = Vec::new();
                        let mut cv_inputs_indexes = Vec::new();
                        let mut cv_outputs_indexes = Vec::new();

                        for port in plugin.ports() {
                            match port.port_type {
                                livi::PortType::ControlInput => {
                                    let ear = ear::control(
                                        Some(&port.name),
                                        audio_format::MIN_CONTROL,
                                        audio_format::MAX_CONTROL,
                                        an_or(port.default_value, audio_format::DEF_CONTROL),
                                    )?;
                                    base.add_ear(ear);
                                    control_inputs_indexes.push(inputs_count);
                                    inputs_count = inputs_count + 1;
                                }
                                livi::PortType::ControlOutput => {
                                    let vc = voice::control(
                                        Some(&port.name),
                                        an_or(port.default_value, audio_format::DEF_CONTROL),
                                    );
                                    base.add_voice(vc);
                                    control_outputs_indexes.push(outputs_count);
                                    outputs_count = outputs_count + 1;
                                }
                                livi::PortType::AudioInput => {
                                    let ear = ear::audio(
                                        Some(&port.name),
                                        audio_format::MIN_AUDIO,
                                        audio_format::MAX_AUDIO,
                                        audio_format::DEF_AUDIO,
                                        &Init::DefValue,
                                    )?;
                                    base.add_ear(ear);
                                    audio_inputs_indexes.push(inputs_count);
                                    inputs_count = inputs_count + 1;
                                }
                                livi::PortType::AudioOutput => {
                                    let vc = voice::audio(Some(&port.name), 0.);
                                    base.add_voice(vc);
                                    audio_outputs_indexes.push(outputs_count);
                                    outputs_count = outputs_count + 1;
                                }
                                livi::PortType::AtomSequenceInput => {
                                    let ear = ear::atom(Some(lv2_handler), Some(&port.name))?;
                                    base.add_ear(ear);
                                    atom_sequence_inputs_indexes.push(inputs_count);
                                    inputs_count = inputs_count + 1;
                                }
                                livi::PortType::AtomSequenceOutput => {
                                    let vc = voice::atom(Some(lv2_handler), Some(&port.name));
                                    base.add_voice(vc);
                                    atom_sequence_outputs_indexes.push(outputs_count);
                                    outputs_count = outputs_count + 1;
                                }
                                livi::PortType::CVInput => {
                                    let ear = ear::cv(
                                        Some(&port.name),
                                        audio_format::MIN_CV,
                                        audio_format::MAX_CV,
                                        audio_format::DEF_CV,
                                        &Init::DefValue,
                                    )?;
                                    base.add_ear(ear);
                                    cv_inputs_indexes.push(inputs_count);
                                    inputs_count = inputs_count + 1;
                                }
                                livi::PortType::CVOutput => {
                                    let vc = voice::cv(Some(&port.name), 0.);
                                    base.add_voice(vc);
                                    cv_outputs_indexes.push(outputs_count);
                                    outputs_count = outputs_count + 1;
                                }
                            }
                        }
                        Ok(ctalker!(
                            base,
                            Self {
                                control_inputs_indexes,
                                control_outputs_indexes,
                                audio_inputs_indexes,
                                audio_outputs_indexes,
                                atom_sequence_inputs_indexes,
                                atom_sequence_outputs_indexes,
                                cv_inputs_indexes,
                                cv_outputs_indexes,
                                instance: RefCell::new(instance),
                            }
                        ))
                    }
                    _ => Err(failure::err_msg("PluginInstantiationError")),
                }
            }
            None => Err(failure::err_msg(format!("LV2 plugin {} not found.", uri))),
        }
    }
}

impl Talker for Lv2 {
    fn activate(&mut self) {}
    fn deactivate(&mut self) {}

    fn talk(&mut self, base: &TalkerBase, _port: usize, tick: i64, len: usize) -> usize {
        let ln = base.listen(tick, len);
        /*
                let control_inputs: Vec<f32> = self
                    .control_inputs_indexes
                    .iter()
                    .map(|i| base.ear(*i).get_control_value())
                    .collect();
                let mut control_outputs: Vec<f32> = self
                    .control_outputs_indexes
                    .iter()
                    .map(|i| base.voice(*i).control_buffer()[0])
                    .collect();
        */
        let audio_inputs: Vec<AudioBuf> = self
            .audio_inputs_indexes
            .iter()
            .map(|i| base.ear(*i).get_audio_buffer())
            .collect();
        let audio_outputs: Vec<MAudioBuf> = self
            .audio_outputs_indexes
            .iter()
            .map(|i| base.voice(*i).audio_buffer())
            .collect();

        let cv_inputs: Vec<CvBuf> = self
            .cv_inputs_indexes
            .iter()
            .map(|i| base.ear(*i).get_cv_buffer())
            .collect();
        let cv_outputs: Vec<MCvBuf> = self
            .cv_outputs_indexes
            .iter()
            .map(|i| base.voice(*i).cv_buffer())
            .collect();

        let atom_sequence_inputs: Vec<AtomBuf> = self
            .atom_sequence_inputs_indexes
            .iter()
            .map(|i| base.ear(*i).get_atom_buffer())
            .collect();
        let atom_sequence_outputs: Vec<MAtomBuf> = self
            .atom_sequence_outputs_indexes
            .iter()
            .map(|i| base.voice(*i).atom_buffer())
            .collect();

        let ports = livi::EmptyPortConnections::new()
            .with_audio_inputs(audio_inputs.into_iter())
            .with_audio_outputs(audio_outputs.into_iter())
            .with_atom_sequence_inputs(atom_sequence_inputs.into_iter())
            .with_atom_sequence_outputs(atom_sequence_outputs.into_iter())
            .with_cv_inputs(cv_inputs.into_iter())
            .with_cv_outputs(cv_outputs.into_iter());
        /*
                    .with_control_inputs(control_inputs.iter())
                    .with_control_outputs(control_outputs.iter_mut())
        */

        unsafe { self.instance.borrow_mut().run(ln, ports).unwrap() };
        /*
                for (i, port_idx) in self.control_outputs_indexes.iter().enumerate() {
                    base.voice(*port_idx).set_control_value(control_outputs[i]);
                }
        */
        for voice in base.voices() {
            voice.set_tick_len(tick, ln);
        }

        ln
    }
}
/*
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
*/
