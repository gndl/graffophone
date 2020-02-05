extern crate failure;
extern crate gpplugin;
extern crate lilv;
extern crate lv2;

use std::alloc::System;
use std::f64::consts::PI;
use std::rc::Rc;

#[global_allocator]
static A: System = System;

use lv2::core::ports::Audio;
use lv2::core::ports::Control;
use lv2::core::SharedFeatureBuffer;

use lilv::instance::PluginInstance;
use lilv::port::buffer::CellBuffer;
use lilv::port::buffer::VecBuffer;
use lilv::port::Port;
use lilv::port::TypedPort;
use lilv::port::{UnknownInputPort, UnknownOutputPort};
use lilv::world::World;

use gpplugin::audio_format::AudioFormat;

mod audio_data;
mod curve_controler;
mod event_bus;
mod graph_controler;
mod mixer;
mod playback_output;
mod plugins_manager;
mod session;
mod session_controler;
mod state;
mod talkers;
mod track;

use crate::audio_data::AudioOutput;
use crate::event_bus::EventBus;
use crate::playback_output::Playback;
use crate::plugins_manager::PluginsManager;
use crate::session::Session;
use crate::session_controler::SessionControler;
use crate::talkers::abs_sine::AbsSine;
use crate::talkers::second_degree_frequency_progression::SecondDegreeFrequencyProgression;
use crate::talkers::sinusoidal::Sinusoidal;

const CHANNELS: usize = 2;
const NUM_SECONDS: u64 = 9;
const SAMPLE_RATE: usize = 44_100;
const FRAMES_PER_SECOND: usize = 10;
const SAMPLES: usize = SAMPLE_RATE / FRAMES_PER_SECOND;

fn main() {
    let bus = EventBus::new_ref();
    let session = Session::new_ref("".to_string());
    let controler = SessionControler::new(session, bus);
    //    let world: World = World::new().unwrap();
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

    match play_sin(&pm) {
        //    match play_progressive_sinusoidale(&pm) {
        Ok(_) => {}
        e => {
            eprintln!("Example failed with the following: {:?}", e);
        }
    }
}

fn play(pm: &PluginsManager) -> Result<(), failure::Error> {
    let mut talkers = Vec::new();
    let fuzzface_uri =
        "http://guitarix.sourceforge.net/plugins/gx_fuzzface_#_fuzzface_".to_string();

    let fuzzface_tkr = pm.make_talker(&fuzzface_uri, None)?;
    talkers.push(fuzzface_tkr.clone());

    let abs_sine_tkr = pm.make_talker(&Sinusoidal::id().to_string(), None)?;
    talkers.push(abs_sine_tkr.clone());

    fuzzface_tkr
        .borrow_mut()
        .set_ear_voice_by_tag(&"In".to_string(), &abs_sine_tkr, 0);
    fuzzface_tkr
        .borrow_mut()
        .set_ear_value_by_tag(&"FUZZ".to_string(), 2f32);
    fuzzface_tkr
        .borrow_mut()
        .set_ear_value_by_tag(&"LEVEL".to_string(), 0.25f32);

    for tkr in &talkers {
        tkr.borrow_mut().activate();
    }

    let mut po = Playback::new(CHANNELS, SAMPLES)?;
    po.open()?;
    let audio_buf = fuzzface_tkr
        .borrow_mut()
        .voice(0)
        .borrow()
        .audio_buffer()
        .unwrap();
    let mut tick: i64 = 0;
    let len = AudioFormat::chunk_size();
    let nb_iter = 2000;
    let secs = ((nb_iter * len) / SAMPLE_RATE) as u64;
    println!("Will play fuzzed abs sinusoidal for {} seconds", secs);

    for _ in 0..nb_iter {
        let ln = fuzzface_tkr.borrow_mut().talk(0, tick, len);
        po.write_mono(&audio_buf, ln)?;
        tick += ln as i64;
    }
    for tkr in &talkers {
        tkr.borrow_mut().deactivate();
    }
    std::thread::sleep(std::time::Duration::from_secs(secs));
    Ok(())
}

fn play_sin(pm: &PluginsManager) -> Result<(), failure::Error> {
    let tkr = pm.make_talker(&Sinusoidal::id().to_string(), None)?;
    tkr.borrow_mut().activate();

    let mut po = Playback::new(CHANNELS, SAMPLES)?;
    po.open()?;
    let audio_buf = tkr.borrow_mut().voice(0).borrow().audio_buffer().unwrap();
    let mut tick: i64 = 0;
    let len = AudioFormat::chunk_size();
    let nb_iter = 100;
    let secs = ((nb_iter * len) / SAMPLE_RATE) as u64;
    println!("Will play sinusoidal for {} seconds", secs);

    for _ in 0..nb_iter {
        let ln = tkr.borrow_mut().talk(0, tick, len);
        po.write_mono(&audio_buf, ln)?;
        tick += ln as i64;
    }

    tkr.borrow_mut().deactivate();

    std::thread::sleep(std::time::Duration::from_secs(secs));
    Ok(())
}

fn play_progressive_sinusoidale(pm: &PluginsManager) -> Result<(), failure::Error> {
    let tkr = pm.make_talker(&SecondDegreeFrequencyProgression::id().to_string(), None)?;
    tkr.borrow_mut().activate();

    let mut po = Playback::new(CHANNELS, SAMPLES)?;
    po.open()?;
    let audio_buf = tkr.borrow_mut().voice(0).borrow().audio_buffer().unwrap();
    let mut tick: i64 = 0;
    let len = AudioFormat::chunk_size();
    let nb_iter = 2000;
    let secs = ((nb_iter * len) / SAMPLE_RATE) as u64;
    println!("Will play sinusoidal for {} seconds", secs);

    for _ in 0..nb_iter {
        let ln = tkr.borrow_mut().talk(0, tick, len);
        po.write_mono(&audio_buf, ln)?;
        tick += ln as i64;
    }

    tkr.borrow_mut().deactivate();

    std::thread::sleep(std::time::Duration::from_secs(secs));
    Ok(())
}

fn run(world: &World, features: SharedFeatureBuffer) -> Result<(), failure::Error> {
    let mut f = 22.5;

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

    let mut po = Playback::new(CHANNELS, SAMPLES)?;
    po.open()?;

    fuzzface_inst.activate();

    for _ in 0..NUM_SECONDS {
        for _ in 0..FRAMES_PER_SECOND {
            for i in 0..SAMPLES {
                let sample = ((i as f64 * PI * 2.0 * f) / SAMPLE_RATE as f64).sin() as f32;
                in_audio_buf.get()[i].set(sample);
            }

            fuzzface_inst.run(SAMPLES as u32);

            po.write_mono(&out_audio_buf, SAMPLES)?;
        }
        f = 2. * f;
    }

    fuzzface_inst.deactivate();

    std::thread::sleep(std::time::Duration::from_secs(NUM_SECONDS));

    po.close()?;

    Ok(())
}
