use lilv::instance::PluginInstance;
use lv2::core::SharedFeatureBuffer;
extern crate failure;
extern crate gpplugin;
extern crate lilv;
extern crate lv2;

use std::alloc::System;
use std::f64::consts::PI;
use std::rc::Rc;

#[global_allocator]
static A: System = System;

use lilv::world::World;
/*
use lilv::plugin::Plugin;
use lilv::*;
use gpplugin::talker::Talker;
 */
use gpplugin::audio_format::AudioFormat;
use lilv::port::buffer::CellBuffer;
use lilv::port::buffer::VecBuffer;
use lilv::port::Port;
use lilv::port::TypedPort;
use lilv::port::{UnknownInputPort, UnknownOutputPort};
use lv2::core::ports::Audio;
use lv2::core::ports::Control;

mod audio_data;
mod playback_output;
mod plugins_manager;
mod talkers;

use crate::audio_data::{AudioOutput, Interleaved};
use crate::playback_output::Playback;
use crate::plugins_manager::PluginsManager;
use crate::talkers::abs_sine;
//use crate::talkers::lv2_talker;

const CHANNELS: usize = 2;
const NUM_SECONDS: u64 = 9;
const SAMPLE_RATE: usize = 44_100;
const FRAMES_PER_SECOND: usize = 10;
const SAMPLES: usize = SAMPLE_RATE / FRAMES_PER_SECOND;

fn main() {
    let world: World = World::new().unwrap();
    //    let mut talkers = Vec::new(); //: Vec<Box<dyn Talker>> = ,

    let mut pm = PluginsManager::new();
    pm.load_plugins( /*&world, &features*/);
    /*
    pm.run();
    match run(&world, pm.features_buffer()) {
        Ok(_) => {}
        e => {
            eprintln!("Example failed with the following: {:?}", e);
        }
    }
     */

    let mut talkers = Vec::new();
    let fuzzface_uri =
        "http://guitarix.sourceforge.net/plugins/gx_fuzzface_#_fuzzface_".to_string();

    match pm.make_talker(&fuzzface_uri, None) {
        Ok(fuzzface_tkr) => {
            talkers.push(fuzzface_tkr.clone());

            match pm.make_talker(&abs_sine::id().to_string(), None) {
                Ok(abs_sine_tkr) => {
                    talkers.push(abs_sine_tkr.clone());
                }
                Err(e) => {
                    eprintln!("Make talker failed: {:?}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("Make talker failed: {:?}", e);
        }
    }
}

fn run(world: &World, features: SharedFeatureBuffer) -> Result<(), failure::Error> {
    let mut f = 22.5;
    let mut av = Vec::with_capacity(CHANNELS * SAMPLES);
    for _ in 0..CHANNELS * SAMPLES {
        av.push(0.);
    }

    let fuzzface = world
        .get_plugin_by_uri("http://guitarix.sourceforge.net/plugins/gx_fuzzface_#_fuzzface_")
        .unwrap();

    let fuzz_ctrl_buf = Rc::new(CellBuffer::new(2f32));
    let level_ctrl_buf = Rc::new(CellBuffer::new(0.25f32));

    let control_bufs = fuzzface
        .inputs()
        .filter_map(UnknownInputPort::into_typed::<Control>)
        .map(|p| {
            if p.name().as_ref() == "FUZZ" {
                (p.handle(), Rc::clone(&fuzz_ctrl_buf))
            } else {
                (p.handle(), Rc::clone(&level_ctrl_buf))
            }
        })
        .collect::<Vec<_>>();

    let in_audio_buf = Rc::new(VecBuffer::new(0f32, SAMPLES));
    let out_audio_buf = Rc::new(VecBuffer::new(0f32, SAMPLES));

    let mut audio_bufs = fuzzface
        .inputs()
        .filter_map(UnknownInputPort::into_typed::<Audio>)
        .map(|p| (p.handle(), Rc::clone(&in_audio_buf)))
        .collect::<Vec<_>>();

    audio_bufs.extend(
        fuzzface
            .outputs()
            .filter_map(UnknownOutputPort::into_typed::<Audio>)
            .map(|p| (p.handle(), Rc::clone(&out_audio_buf))),
    );

    let mut fuzzface_inst =
        PluginInstance::new(&fuzzface, AudioFormat::sample_rate() as f64, features).unwrap();

    for buf in &control_bufs {
        fuzzface_inst.connect_port(buf.0.clone(), buf.1.clone())
    }

    for buf in &audio_bufs {
        fuzzface_inst.connect_port(buf.0.clone(), buf.1.clone())
    }

    let po = Playback::new()?;
    po.open()?;

    fuzzface_inst.activate();

    for _ in 0..NUM_SECONDS {
        for _ in 0..FRAMES_PER_SECOND {
            for i in 0..SAMPLES {
                let sample = ((i as f64 * PI * 2.0 * f) / SAMPLE_RATE as f64).sin() as f32;
                in_audio_buf.get()[i].set(sample);
            }

            fuzzface_inst.run(SAMPLES as u32);

            for i in 0..SAMPLES {
                let sample = out_audio_buf.get()[i].get();
                av[CHANNELS * i] = sample;
                av[CHANNELS * i + 1] = sample;
            }

            let ad = Interleaved::new(CHANNELS, SAMPLES, &av);
            po.write(ad)?;
        }
        f = 2. * f;
    }

    fuzzface_inst.deactivate();

    std::thread::sleep(std::time::Duration::from_secs(NUM_SECONDS));

    po.close()?;

    Ok(())
}
