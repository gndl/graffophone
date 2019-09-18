extern crate lilv;

use std::alloc::System;
use std::io;

#[global_allocator]
static A: System = System;

/*
mod lilv;
use lilv::world::World;
use lilv::*;
use lilv::port::{ UnknownInputPort, UnknownOutputPort };
use lilv::port::TypedPort;
use lilv::port::buffer::CellBuffer;
use lv2::core::ports::Audio;
use lv2::core::ports::Control;
use lv2::core::{Feature, FeatureSet, FeatureBuffer};
use lilv::port::buffer::VecBuffer;

struct MyFeatureSet {
    hard_rt_capable: ::lv2::core::features::HardRTCapable
}

impl MyFeatureSet {
    pub fn new() -> Self {
        Self {
            hard_rt_capable: ::lv2::core::features::HardRTCapable
        }
    }
}

impl<'a> FeatureSet<'a> for MyFeatureSet {
    fn to_list(&self) -> FeatureBuffer {
        FeatureBuffer::from_vec(vec![
            Feature::descriptor(&self.hard_rt_capable)
        ])
    }
}
*/

extern crate portaudio;

use portaudio as pa;
//use std::collections::VecDeque;
use std::f64::consts::PI;

const CHANNELS: i32 = 2;
const NUM_SECONDS: i32 = 5;
const SAMPLE_RATE: f64 = 44_100.0;
const FRAMES_PER_BUFFER: u32 = 64;
const TABLE_SIZE: usize = 200;

fn main() {
    //    println!("lilv_plugins_size: {}", lilv_sys::lilv_plugins_size(plugins));
    let world = lilv::world::World::new().unwrap();

    println!("Print plugins start");

    for plugin in world.plugins() {
        println!("{}", plugin.name());
        //        PluginInstance
    }
    println!("Print plugins end");

    match run() {
        Ok(_) => {}
        e => {
            eprintln!("Example failed with the following: {:?}", e);
        }
    }
}

fn run() -> Result<(), pa::Error> {
    let pa = pa::PortAudio::new()?;

    let device_index = select_device(&pa)?;
    play_sine(&pa, device_index)?;
    run_blocking()
}

const INTERLEAVED: bool = true;

fn select_device<'a>(pa: &'a pa::PortAudio) -> Result<pa::DeviceIndex, pa::Error> {
    for device in pa.devices()? {
        let (idx, info) = device?;
        println!("{} {}", idx.0, info.name);
    }
    println!("Please input your device.");

    let mut dev_idx = String::new();

    io::stdin()
        .read_line(&mut dev_idx)
        .expect("Failed to read line");

    match dev_idx.trim().parse() {
        Ok(idx) => Ok(pa::DeviceIndex(idx)),
        Err(_) => Err(pa::Error::InvalidDevice),
    }
}

fn play_sine<'a>(pa: &'a pa::PortAudio, device_index: pa::DeviceIndex) -> Result<(), pa::Error> {
    // Initialise sinusoidal wavetable.
    let mut sine = [0.0; TABLE_SIZE];
    for i in 0..TABLE_SIZE {
        sine[i] = (i as f64 / TABLE_SIZE as f64 * PI * 2.0).sin() as f32;
    }
    let mut left_phase = 0;
    let mut right_phase = 0;

    println!("let device_info = pa.device_info(device_index)?;");
    let device_info = pa.device_info(device_index)?;
    println!("Default output device info: {:#?}", &device_info);

    // Construct the output stream parameters.
    let latency = device_info.default_high_output_latency;
    let output_params = pa::StreamParameters::new(device_index, CHANNELS, INTERLEAVED, latency);

    // Construct the settings with which we'll open our duplex stream.
    let settings = pa::OutputStreamSettings::new(output_params, SAMPLE_RATE, FRAMES_PER_BUFFER);
    /*
        let mut settings =
            pa.default_output_stream_settings(CHANNELS, SAMPLE_RATE, FRAMES_PER_BUFFER)?;
        // we won't output out of range samples so don't bother clipping them.
        settings.flags = pa::stream_flags::CLIP_OFF;
    */
    // This routine will be called by the PortAudio engine when audio is needed. It may called at
    // interrupt level on some machines so don't do anything that could mess up the system like
    // dynamic resource allocation or IO.
    let callback = move |pa::OutputStreamCallbackArgs { buffer, frames, .. }| {
        let mut idx = 0;
        for _ in 0..frames {
            buffer[idx] = sine[left_phase];
            buffer[idx + 1] = sine[right_phase];
            left_phase += 1;
            if left_phase >= TABLE_SIZE {
                left_phase -= TABLE_SIZE;
            }
            right_phase += 3;
            if right_phase >= TABLE_SIZE {
                right_phase -= TABLE_SIZE;
            }
            idx += 2;
        }
        pa::Continue
    };

    println!("{:#?}", &settings);

    println!("pa.open_non_blocking_stream");
    let mut stream = pa.open_non_blocking_stream(settings, callback)?;

    println!("stream.start()");
    stream.start()?;

    println!("Play for {} seconds.", NUM_SECONDS);
    pa.sleep(NUM_SECONDS * 1_000);

    println!("stream.stop()");
    stream.stop()?;
    stream.close()?;

    println!("Test finished.");

    Ok(())
}

fn run_blocking() -> Result<(), pa::Error> {
    // Initialise sinusoidal wavetable.
    let mut sine = [0.0; TABLE_SIZE];
    let f = 440.0;

    for i in 0..TABLE_SIZE {
        sine[i] = ((i as f64 * PI * 2.0 * f) / SAMPLE_RATE).sin() as f32;
    }

    let pa = pa::PortAudio::new()?;

    println!("let pa = pa::PortAudio::new()");

    let settings = pa.default_output_stream_settings(CHANNELS, SAMPLE_RATE, FRAMES_PER_BUFFER)?;
    println!("let settings = pa.default_output_stream_settings(CHANNELS, SAMPLE_RATE, FRAMES)?;");
    // we won't output out of range samples so don't bother clipping them.
    //    settings.flags = pa::stream_flags::CLIP_OFF;
    /*
        let def_output = pa.default_output_device();
        let device_info = pa.device_info(def_output);
        println!("Default output device info: {:#?}", &device_info);

        // Construct the output stream parameters.
        let latency = device_info.default_low_output_latency;
        let output_params =
            pa::StreamParameters::<f32>::new(def_output, CHANNELS, INTERLEAVED, latency);

        // Construct the settings with which we'll open our duplex stream.
        let settings = pa::OutputStreamSettings::new(output_params, SAMPLE_RATE, FRAMES);
    */
    let mut stream = pa.open_blocking_stream(settings)?;
    println!("let mut stream = pa.open_blocking_stream(settings)?;");

    stream.start()?;
    println!("stream.start()?;");

    'stream: loop {
        println!("'stream: loop ");
        stream.write(TABLE_SIZE as u32 * 2, |output| {
            println!("stream.write(TABLE_SIZE as u32 * 2, |output|");
            let buffer_frames = output.len() / CHANNELS as usize;
            for i in 0..buffer_frames {
                println!("output[{}] = sine[{}]", i * 2, i);

                output[i * 2] = sine[i];
                output[i * 2 + 1] = sine[i];
            }
        })?;
    }
    /*
    stream.stop()?;
    stream.close()?;
    Ok(())
     */
}
