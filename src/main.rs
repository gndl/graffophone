extern crate failure;
extern crate lilv;

use std::alloc::System;

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

//use std::collections::VecDeque;
mod audio_data;

mod playback_output;
use crate::audio_data::{AudioOutput, Interleaved};

use crate::playback_output::Playback;
use std::f64::consts::PI;

const CHANNELS: usize = 2;
const NUM_SECONDS: u64 = 9;
const SAMPLE_RATE: usize = 44_100;
const FRAMES_PER_SECOND: usize = 10;
const SAMPLES: usize = SAMPLE_RATE / FRAMES_PER_SECOND;

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

fn run() -> Result<(), failure::Error> {
    play_sine()
}

fn play_sine() -> Result<(), failure::Error> {
    let mut f = 22.5;
    let mut av = Vec::with_capacity(CHANNELS * SAMPLES);
    for _ in 0..CHANNELS * SAMPLES {
        av.push(0.);
    }

    let po = Playback::new()?;

    po.open()?;

    for _ in 0..NUM_SECONDS {
        for _ in 0..FRAMES_PER_SECOND {
            for i in 0..SAMPLES {
                let sample = ((i as f64 * PI * 2.0 * f) / SAMPLE_RATE as f64).sin() as f32;
                //                let sample = fuzz(sample);

                av[CHANNELS * i] = sample;
                av[CHANNELS * i + 1] = sample;
            }

            let ad = Interleaved::new(CHANNELS, SAMPLES, &av);
            po.write(ad)?;
        }
        f = 2. * f;
    }

    std::thread::sleep(std::time::Duration::from_secs(NUM_SECONDS));

    po.close()?;

    Ok(())
}
// the fuzz filter, when applied to all samples, will add some
// distortion
fn fuzz(input: f32) -> f32 {
    (0..4).fold(input, |acc, _| cubic_amplifier(acc))
}

fn cubic_amplifier(input: f32) -> f32 {
    // samples should be between -1.0 and 1.0
    if input < 0.0 {
        // if it's negative (-1.0 to 0), then adding 1.0 takes it to
        // the 0 to 1.0 range. If it's cubed, it still won't leave the
        // 0 to 1.0 range.
        let negated = input + 1.0;
        (negated * negated * negated) - 1.0
    } else {
        // if it's positive (0 to 1.0), then subtracting 1.0 takes it
        // to the -1.0 to 0 range. If it's cubed, it still won't leave
        // the -1.0 to 0 range.
        let negated = input - 1.0;
        (negated * negated * negated) + 1.0
    }
}
