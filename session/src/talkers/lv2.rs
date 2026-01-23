use std::ffi::CStr;
use std::collections::HashMap;

use livi::{self, PortIndex, PortType, Plugin};
use livi::event::{LV2AtomEventBuilder, LV2AtomSequence};

use talker::audio_format;
use talker::audio_format::AudioFormat;
use talker::ctalker;
use talker::ear;
use talker::ear::Init;
use talker::horn::{AtomBuf, AudioBuf, CvBuf, MAtomBuf, MAudioBuf, MCvBuf};
use talker::identifier::{Identifiable, Index};
use talker::lv2_handler::{self, Lv2Handler};
use talker::talker::{CTalker, Talker, TalkerBase};
use talker::data::Data;

const ATOM_SEQUENCE_CAPACITY: usize = 65536;


fn an_or(v: f32, def: f32) -> f32 {
    if v.is_nan() {
        def
    } else {
        v
    }
}

struct Idxs {tkr_port: usize, plugin_port: usize}
pub struct Lv2 {
    uri: String,
    urid_float_protocol: u32,
    urid_event_transfer: u32,
    urid_atom_transfer: u32,
    atom_sequence: LV2AtomSequence,
    atom_sequences: Vec<LV2AtomSequence>,
    control_inputs_indexes: Vec<Idxs>,
    control_outputs_indexes: Vec<Idxs>,
    audio_inputs_indexes: Vec<Idxs>,
    audio_outputs_indexes: Vec<Idxs>,
    atom_sequence_inputs_indexes: Vec<Idxs>,
    atom_sequence_outputs_indexes: Vec<Idxs>,
    cv_inputs_indexes: Vec<Idxs>,
    cv_outputs_indexes: Vec<Idxs>,
    instance: livi::Instance,
}

impl Lv2 {
    pub fn new(lv2_handler: &Lv2Handler, uri: &str, mut base: TalkerBase) -> Result<CTalker, failure::Error> {
        let urid_float_protocol = lv2_handler.features.urid(CStr::from_bytes_with_nul(lv2_sys::LV2_UI__floatProtocol).unwrap());
        let urid_event_transfer = lv2_handler.features.urid(CStr::from_bytes_with_nul(lv2_sys::LV2_ATOM__eventTransfer).unwrap());
        let urid_atom_transfer = lv2_handler.features.urid(CStr::from_bytes_with_nul(lv2_sys::LV2_ATOM__atomTransfer).unwrap());

        let atom_sequence = LV2AtomSequence::new(&lv2_handler.features, ATOM_SEQUENCE_CAPACITY);
        let mut atom_sequences = Vec::new();

        match lv2_handler.world.plugin_by_uri(uri) {
            Some(plugin) => {

                show_plugin(&plugin);

                if lv2_handler.plugin_ui_supported(&plugin) {
                    base.set_data(Data::UI);
                }

                match unsafe {
                    plugin.instantiate(
                        lv2_handler.features.clone(),
                        AudioFormat::sample_rate() as f64,
                    )
                } {
                    Ok(instance) => {
                        base.set_name(&plugin.name());
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
                                    control_inputs_indexes.push(Idxs{tkr_port: inputs_count, plugin_port: port.index.0});
                                    inputs_count = inputs_count + 1;
                                }
                                livi::PortType::ControlOutput => {
                                    base.add_control_voice(Some(&port.name), an_or(port.default_value, audio_format::DEF_CONTROL));
                                    control_outputs_indexes.push(Idxs{tkr_port: outputs_count, plugin_port: port.index.0});
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
                                    audio_inputs_indexes.push(Idxs{tkr_port: inputs_count, plugin_port: port.index.0});
                                    inputs_count = inputs_count + 1;
                                }
                                livi::PortType::AudioOutput => {
                                    base.add_audio_voice(Some(&port.name), 0.);
                                    audio_outputs_indexes.push(Idxs{tkr_port: outputs_count, plugin_port: port.index.0});
                                    outputs_count = outputs_count + 1;
                                }
                                livi::PortType::AtomSequenceInput => {
                                    atom_sequences.push(LV2AtomSequence::new(&lv2_handler.features, ATOM_SEQUENCE_CAPACITY));

                                    let ear = ear::atom(Some(&port.name), Some(lv2_handler))?;
                                    base.add_ear(ear);
                                    atom_sequence_inputs_indexes.push(Idxs{tkr_port: inputs_count, plugin_port: port.index.0});
                                    inputs_count = inputs_count + 1;
                                }
                                livi::PortType::AtomSequenceOutput => {
                                    base.add_atom_voice(Some(&port.name), Some(lv2_handler));
                                    atom_sequence_outputs_indexes.push(Idxs{tkr_port: outputs_count, plugin_port: port.index.0});
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
                                    cv_inputs_indexes.push(Idxs{tkr_port: inputs_count, plugin_port: port.index.0});
                                    inputs_count = inputs_count + 1;
                                }
                                livi::PortType::CVOutput => {
                                    base.add_cv_voice(Some(&port.name), 0.);
                                    cv_outputs_indexes.push(Idxs{tkr_port: outputs_count, plugin_port: port.index.0});
                                    outputs_count = outputs_count + 1;
                                }
                            }
                        }
                        Ok(ctalker!(
                            base,
                            Self {
                                uri: uri.to_string(),
                                urid_float_protocol,
                                urid_event_transfer,
                                urid_atom_transfer,
                                atom_sequence,
                                atom_sequences,
                                control_inputs_indexes,
                                control_outputs_indexes,
                                audio_inputs_indexes,
                                audio_outputs_indexes,
                                atom_sequence_inputs_indexes,
                                atom_sequence_outputs_indexes,
                                cv_inputs_indexes,
                                cv_outputs_indexes,
                                instance,
                            }
                        ))
                    }
                    _ => Err(failure::err_msg("PluginInstantiationError")),
                }
            }
            None => Err(failure::err_msg(format!("LV2 plugin {} not found.", uri))),
        }
    }

    fn connect_ports(&mut self, base: &TalkerBase) {
        let livi_active_instance = self.instance.raw_mut();
        let livi_instance = livi_active_instance.instance_mut();

        unsafe {
            for idx in &self.audio_inputs_indexes {
                livi_instance.connect_port(idx.plugin_port, base.ear(idx.tkr_port).get_audio_buffer().as_ptr());
            }
            for idx in &self.audio_outputs_indexes {
                livi_instance.connect_port(idx.plugin_port, base.voice(idx.tkr_port).audio_buffer().as_mut_ptr());
            }

            for idx in &self.control_inputs_indexes {
                livi_instance.connect_port(idx.plugin_port, base.ear(idx.tkr_port).get_control_buffer().as_ptr());
            }

            for idx in &self.cv_inputs_indexes {
                livi_instance.connect_port(idx.plugin_port, base.ear(idx.tkr_port).get_cv_buffer().as_ptr());
            }
            for idx in &self.cv_outputs_indexes {
                livi_instance.connect_port(idx.plugin_port, base.voice(idx.tkr_port).cv_buffer().as_mut_ptr());
            }

            for idx in &self.atom_sequence_inputs_indexes {
                livi_instance.connect_port(idx.plugin_port, base.ear(idx.tkr_port).get_atom_buffer().as_ptr());
            }
            for idx in &self.atom_sequence_outputs_indexes {
                base.voice(idx.tkr_port).atom_buffer().clear_as_chunk();
                livi_instance.connect_port(idx.plugin_port, base.voice(idx.tkr_port).atom_buffer().as_mut_ptr());
            }
        }
    }


    fn talk0(&mut self, base: &TalkerBase, _port: usize, tick: i64, len: usize) -> usize {
        let ln = base.listen(tick, len);

        for idx in &self.control_inputs_indexes {
            self.instance
                .set_control_input(PortIndex(idx.plugin_port), base.ear(idx.tkr_port).get_control_value());
        }

        let audio_inputs: Vec<AudioBuf> = self
            .audio_inputs_indexes
            .iter()
            .map(|idx| base.ear(idx.tkr_port).get_audio_buffer())
            .collect();
        let audio_outputs: Vec<MAudioBuf> = self
            .audio_outputs_indexes
            .iter()
            .map(|idx| base.voice(idx.tkr_port).audio_buffer())
            .collect();

        let cv_inputs: Vec<CvBuf> = self
            .cv_inputs_indexes
            .iter()
            .map(|idx| base.ear(idx.tkr_port).get_cv_buffer())
            .collect();
        let cv_outputs: Vec<MCvBuf> = self
            .cv_outputs_indexes
            .iter()
            .map(|idx| base.voice(idx.tkr_port).cv_buffer())
            .collect();

        let atom_sequence_inputs: Vec<AtomBuf> = self
            .atom_sequence_inputs_indexes
            .iter()
            .map(|idx| base.ear(idx.tkr_port).get_atom_buffer())
            .collect();
        let atom_sequence_outputs: Vec<MAtomBuf> = self
            .atom_sequence_outputs_indexes
            .iter()
            .map(|idx| base.voice(idx.tkr_port).atom_buffer())
            .collect();

        let ports = livi::EmptyPortConnections::new()
            .with_audio_inputs(audio_inputs.into_iter())
            .with_audio_outputs(audio_outputs.into_iter())
            .with_atom_sequence_inputs(atom_sequence_inputs.into_iter())
            .with_atom_sequence_outputs(atom_sequence_outputs.into_iter())
            .with_cv_inputs(cv_inputs.into_iter())
            .with_cv_outputs(cv_outputs.into_iter());

        unsafe { self.instance.run(ln, ports).unwrap() };

        for idx in &self.control_outputs_indexes {
            if let Some(value) = self.instance.control_output(PortIndex(idx.plugin_port)) {
                base.voice(idx.tkr_port).set_control_value(value);
            }
        }

        for voice in base.voices() {
            voice.set_tick_len(tick, ln);
        }

        ln
    }
}

impl Talker for Lv2 {
    fn activate(&mut self) {}
    fn deactivate(&mut self) {}

    fn set_indexed_data(&mut self, base: &TalkerBase, port_index: Index, protocol: u32, data: &Vec<u8>) -> Result<(), failure::Error> {

        if protocol == self.urid_event_transfer || protocol == self.urid_atom_transfer {
            if protocol == self.urid_event_transfer {
                println!("set_indexed_data event_transfer");
            }
            else {
                println!("set_indexed_data atom_transfer");
            }
            self.connect_ports(base);
            
            let livi_active_instance = self.instance.raw_mut();
            let livi_instance = livi_active_instance.instance_mut();

            self.atom_sequence.clear();

            let header = unsafe { (data.as_ptr() as *const lv2_raw::LV2Atom).as_ref().unwrap() };

            let content = &data[std::mem::size_of::<lv2_raw::LV2Atom>()..];

            let event: LV2AtomEventBuilder<ATOM_SEQUENCE_CAPACITY> = LV2AtomEventBuilder::new(0, header.mytype, content).unwrap();
            self.atom_sequence.push_event(&event).unwrap();

            unsafe{ livi_instance.connect_port(port_index, self.atom_sequence.as_ptr()); }
            unsafe{ livi_active_instance.run(2); }
            let _ = self.instance.run_worker();
        }
        Ok(())
    }

    fn read_port_events(&mut self, base: &TalkerBase) -> Result<Vec<(u32, u32, Vec<u8>)>, failure::Error> {

        let mut port_events = Vec::new();

        for idx in &self.atom_sequence_outputs_indexes {
            let atom_buf = base.voice(idx.tkr_port).atom_buffer();

            for ev in atom_buf.iter() {
                let header_size = std::mem::size_of::<lv2_raw::LV2Atom>();
                let buffer_size = header_size + ev.data.len();
                let mut buffer = Vec::with_capacity(buffer_size as usize);

                let header = unsafe{ std::slice::from_raw_parts((&ev.event.body) as *const lv2_raw::LV2Atom as *const u8, header_size) };
                buffer.extend_from_slice(header);
                buffer.extend_from_slice(ev.data);

                port_events.push((idx.plugin_port as u32, self.urid_event_transfer, buffer));
            }

            atom_buf.clear();
        }

        Ok(port_events)
    }

    fn talk(&mut self, base: &TalkerBase, _port: usize, tick: i64, len: usize) -> usize {
        let ln = base.listen(tick, len);

        self.connect_ports(base);

        let livi_active_instance = self.instance.raw_mut();

        unsafe{ livi_active_instance.run(ln); }

        let _ = self.instance.run_worker();

        for idx in &self.control_outputs_indexes {
            if let Some(value) = self.instance.control_output(PortIndex(idx.plugin_port)) {
                base.voice(idx.tkr_port).set_control_value(value);
            }
        }

        for voice in base.voices() {
            voice.set_tick_len(tick, ln);
        }

        ln
    }

    fn state(&mut self) -> Result<Option<String>, failure::Error> {
        lv2_handler::visit(|lv2_handler| {
            match lv2_handler.world.plugin_by_uri(&self.uri) {
                Some(plugin) => {
                    Ok(self.instance.state_string(
                        &plugin,
                        Some("/home/gndl/tmp/file"),
                        Some("/home/gndl/tmp/copy"),
                        Some("/home/gndl/tmp/link"),
                        Some("/home/gndl/tmp/save"),
                        None,
                        lv2_sys::LV2_State_Flags::LV2_STATE_IS_POD))
                }
                None => Err(failure::err_msg(format!("LV2 plugin {} not found.", &self.uri))),
            }
        })
    }

    fn set_state(&self, state_string: &str) -> Result<(), failure::Error> {
        
        lv2_handler::visit(|lv2_handler| {
            let ostate = lv2_handler.world.new_state_from_string(
                lv2_handler.features.as_ref(),
                state_string);

            if let Some(state) = ostate {
                self.instance.restore_state(
                    &state,
                    None,
                    lv2_sys::LV2_State_Flags::LV2_STATE_IS_POD,
                );
                Ok(())
            }
            else {
                Err(failure::err_msg(format!("LV2 plugin {} instance state restoration failed.", &self.uri)))
            }
        })
    }
}

pub fn get_bundle_uri(plugin_uri: &str) -> Result<String, failure::Error> {
    lv2_handler::visit(|lv2_handler| {
        match lv2_handler.world.plugin_by_uri(plugin_uri) {
            Some(plugin) => {
                let bundle_node = plugin.raw().bundle_uri();
                let bundle_uri = bundle_node.as_uri().unwrap_or("");
                Ok(bundle_uri.to_string())
            }
            None => Ok("".to_string()),
        }
    })
}

pub fn get_ears_indexes(plugin_uri: &str) -> Result<Vec<usize>, failure::Error> {
    lv2_handler::visit(|lv2_handler| {
        match lv2_handler.world.plugin_by_uri(plugin_uri) {
            Some(plugin) => {
                let mut ears_indexes: Vec<usize> = vec![0; plugin.ports().count()];
                let mut ear_idx = 0;

                for port in plugin.ports() {
                    match port.port_type {
                        PortType::ControlInput | PortType::AudioInput | PortType::AtomSequenceInput | PortType::CVInput => {
                            ears_indexes[port.index.0] = ear_idx;
                            ear_idx += 1;
                        },
                        _ => (),
                    }
                }
                Ok(ears_indexes)
            }
            None => Err(failure::err_msg(format!("LV2 plugin {} not found.", plugin_uri))),
        }
    })
}

pub fn get_port_symbol_indexes(plugin_uri: &str) -> Result<HashMap<String, u32>, failure::Error> {
    lv2_handler::visit(|lv2_handler| {
        match lv2_handler.world.plugin_by_uri(plugin_uri) {
            Some(plugin) => {
                let mut port_symbol_indexes = HashMap::new();

                for port in plugin.ports() {
                    port_symbol_indexes.insert(port.symbol, port.index.0 as u32);
                }
                Ok(port_symbol_indexes)
            }
            None => Err(failure::err_msg(format!("LV2 plugin {} not found.", plugin_uri))),
        }
    })
}

pub fn show_plugin(plugin: &Plugin) {
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

    if let Some(uis) = lilv_plugin.uis() {
        for ui in uis {
            println!("\tUI plugin {:?} :", ui.uri().turtle_token());
            println!("\t\tbinary_uri : {:?}", ui.binary_uri().map_or("None".to_string(), |n| n.turtle_token()));
            println!("\t\tbundle_uri : {:?}", ui.bundle_uri().map_or("None".to_string(), |n| n.turtle_token()));

            println!("\t\tclasses :");
            for classe in ui.classes() {
                println!("\t\t\t{:?}", classe.turtle_token());
            }
        }
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

    let _instance = unsafe {
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
