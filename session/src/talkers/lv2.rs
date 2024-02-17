use std::cell::RefCell;

use livi;
use livi::PortIndex;
use livi::Plugin;

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
    control_inputs_indexes: Vec<(usize, PortIndex)>,
    control_outputs_indexes: Vec<(PortIndex, usize)>,
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
                show_plugin(&plugin);
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
                                        port.min_value.unwrap_or(audio_format::MIN_CONTROL),
                                        port.max_value.unwrap_or(audio_format::MAX_CONTROL),
                                        an_or(port.default_value, audio_format::DEF_CONTROL),
                                    )?;
                                    base.add_ear(ear);
                                    control_inputs_indexes.push((inputs_count, port.index));
                                    inputs_count = inputs_count + 1;
                                }
                                livi::PortType::ControlOutput => {
                                    let vc = voice::control(
                                        Some(&port.name),
                                        an_or(port.default_value, audio_format::DEF_CONTROL),
                                    );
                                    base.add_voice(vc);
                                    control_outputs_indexes.push((port.index, outputs_count));
                                    outputs_count = outputs_count + 1;
                                }
                                livi::PortType::AudioInput => {
                                    let ear = ear::audio(
                                        Some(&port.name),
                                        port.min_value.unwrap_or(audio_format::MIN_AUDIO),
                                        port.max_value.unwrap_or(audio_format::MAX_AUDIO),
                                        an_or(port.default_value, audio_format::DEF_AUDIO),
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
                                    let ear = ear::atom(Some(&port.name), Some(lv2_handler))?;
                                    base.add_ear(ear);
                                    atom_sequence_inputs_indexes.push(inputs_count);
                                    inputs_count = inputs_count + 1;
                                }
                                livi::PortType::AtomSequenceOutput => {
                                    let vc = voice::atom(Some(&port.name), Some(lv2_handler));
                                    base.add_voice(vc);
                                    atom_sequence_outputs_indexes.push(outputs_count);
                                    outputs_count = outputs_count + 1;
                                }
                                livi::PortType::CVInput => {
                                    let ear = ear::cv(
                                        Some(&port.name),
                                        port.min_value.unwrap_or(audio_format::MIN_CV),
                                        port.max_value.unwrap_or(audio_format::MAX_CV),
                                        an_or(port.default_value, audio_format::DEF_CV),
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

        for (ear_idx, port_idx) in &self.control_inputs_indexes {
            self.instance
                .borrow_mut()
                .set_control_input(*port_idx, base.ear(*ear_idx).get_control_value());
        }

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

        unsafe { self.instance.borrow_mut().run(ln, ports).unwrap() };

        for (port_idx, voice_idx) in &self.control_outputs_indexes {
            if let Some(value) = self.instance.borrow().control_output(*port_idx) {
                base.voice(*voice_idx).set_control_value(value);
            }
        }

        for voice in base.voices() {
            voice.set_tick_len(tick, ln);
        }

        ln
    }
}

fn show_plugin(plugin: &Plugin) {
    println!("plugin {} ({})", plugin.name(), plugin.uri());

    for classe in plugin.classes() {
        println!("\tclasse : {:?}", classe);
    }

   let lilv_plugin = plugin.raw();

    println!("\tbundle_uri : {:?}", lilv_plugin.bundle_uri());
    println!("\tlibrary_uri : {:?}", lilv_plugin.library_uri());

    for node in lilv_plugin.data_uris() {
       println!("\tdata_uri : {:?}", node.turtle_token());
    }

    for node in lilv_plugin.supported_features() {
       println!("\tsupported_feature : {:?}", node.turtle_token());
    }

    for node in lilv_plugin.required_features() {
       println!("\trequired_feature : {:?}", node.turtle_token());
    }

   if let Some(nodes) = lilv_plugin.extension_data() {
       for node in nodes {
           println!("\textension_data : {:?}", node.turtle_token());
       }
   }
    for port in plugin.ports() {
        println!("\tport : {:?}", port);
    }
}
/*
*/

use crate::feedback::Feedback;
use output::Output;

fn var_len_of(mut value: usize) -> Vec<u8> {
    let mut var_len_data: Vec<u8> = Vec::with_capacity(4);
    let mut buffer = value & 0x7f;

    loop {
        value >>= 7;

        if value == 0 {
            break;
        }
        buffer <<= 8;
        buffer += (value & 0x7f) | 0x80;
    }
    loop {
        var_len_data.push((buffer & 0xff) as u8);

        if (buffer & 0x80) != 0 {
            buffer >>= 8;
        } else {
            break;
        }
    }
    var_len_data
}

#[test]
fn test_var_len_of() {
    assert_eq!(var_len_of(0), vec![0]);
    assert_eq!(var_len_of(127), vec![127]);
    assert_eq!(var_len_of(128), vec![0x81, 0]);
    assert_eq!(var_len_of(0xC8), vec![0x81, 0x48]);
    assert_eq!(var_len_of(255), vec![0x81, 0x7f]);
    assert_eq!(var_len_of(256), vec![0x82, 0]);
    assert_eq!(var_len_of(0x100000), vec![0xC0, 0x80, 0]);
}

#[test]
fn test_fuildsynth_plugin() {
    let world = livi::World::new();
    let sample_rate = AudioFormat::sample_rate();

    let features = world.build_features(livi::FeaturesBuilder {
        min_block_length: 1,
        max_block_length: sample_rate, //4096,
    });

    let plugin = world
        .plugin_by_uri("urn:ardour:a-fluidsynth")
        .expect("Plugin not found.");

    show_plugin(&plugin);
    let mut instance = unsafe {
        plugin
            .instantiate(features.clone(), sample_rate as f64)
            .expect("Could not instantiate plugin.")
    };
}


fn run_midi() -> Result<(), failure::Error> {
    let world = livi::World::new();
    let sample_rate = AudioFormat::sample_rate();

    let features = world.build_features(livi::FeaturesBuilder {
        min_block_length: 1,
        max_block_length: sample_rate, //4096,
    });

    let plugin = world
        .plugin_by_uri("http://drobilla.net/plugins/mda/EPiano")
        .expect("Plugin not found.");

    let mut instance = unsafe {
        plugin
            .instantiate(features.clone(), sample_rate as f64)
            .expect("Could not instantiate plugin.")
    };

    let mut input = livi::event::LV2AtomSequence::new(&features, 1024);

    /*
    let mut header_chunk_data: Vec<u8> = vec![0x4D, 0x54, 0x68, 0x64, 0, 0, 0, 6, 0, 2, 0, 1];
    let mut time_div_data = var_len_of(sample_rate / 4);
    header_chunk_data.append(&mut time_div_data);
    let set_tempo_event_data = [255, 81, 0x03, 0xD0, 0x90];
    input.push_midi_event::<14>(0, features.midi_urid(), &header_chunk_data)?;
    input.push_midi_event::<5>(0, features.midi_urid(), &set_tempo_event_data)?;
*/

    let note_a_on_event = [0x90, 0x40, 0xff];
    let note_a_off_event = [0x80, 0x40, 0xff];
    let note_b_on_event = [0x90, 0x50, 0xff];
    let note_b_off_event = [0x80, 0x50, 0xff];

/*
        input.push_midi_event::<3>(0, features.midi_urid(), &note_a_on_event)?;
        input.push_midi_event::<3>(10000, features.midi_urid(), &note_a_off_event)?;
        input.push_midi_event::<3>(20000, features.midi_urid(), &note_a_on_event)?;
        input.push_midi_event::<3>(30000, features.midi_urid(), &note_a_off_event)?;
        input.push_midi_event::<3>(35000, features.midi_urid(), &note_a_on_event)?;
        input.push_midi_event::<3>(44000, features.midi_urid(), &note_a_off_event)?;
    */
    let mut outputs = vec![
        vec![0.0; features.max_block_length()], // For mda EPiano, this is the left channel.
        vec![0.0; features.max_block_length()], // For mda EPiano, this is the right channel.
    ];

    let mut feedback = Feedback::new(features.max_block_length())?;
    feedback.open()?;
    feedback.run()?;

        input.push_midi_event::<3>(0, features.midi_urid(), &note_a_on_event)?;
        input.push_midi_event::<3>(12000, features.midi_urid(), &note_a_off_event)?;
        input.push_midi_event::<3>(35000, features.midi_urid(), &note_b_on_event)?;
        input.push_midi_event::<3>(40000, features.midi_urid(), &note_b_off_event)?;

    for _ in 0..5 {
        let ports = livi::EmptyPortConnections::new()
            .with_atom_sequence_inputs(std::iter::once(&input))
            .with_audio_outputs(outputs.iter_mut().map(|output| output.as_mut_slice()));

        unsafe { instance.run(features.max_block_length(), ports)? };

        feedback.write(&outputs, features.max_block_length())?;
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
        input.clear();
//    std::thread::sleep(std::time::Duration::from_secs(1));

    feedback.close()?;
    Ok(())
}

#[test]
fn test_run_midi() {
    run_midi().unwrap();
}
