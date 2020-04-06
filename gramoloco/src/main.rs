#![allow(dead_code, unused_variables, unused_imports)]
extern crate failure;
extern crate gramotor;

use std::alloc::System;
use std::cell::RefCell;
use std::f64::consts::PI;
use std::fs;
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

use granode::audio_format::AudioFormat;
use granode::talker::Talker;

use gramotor::event_bus::EventBus;
use gramotor::gramotor::Gramotor;
use gramotor::mixer::Mixer;
use gramotor::output::Output;
use gramotor::playback::Playback;
use gramotor::player::Player;
use gramotor::session::{RSession, Session};
use gramotor::talkers::second_degree_frequency_progression;
use gramotor::talkers::second_degree_frequency_progression::SecondDegreeFrequencyProgression;
use gramotor::talkers::sinusoidal;
use gramotor::talkers::sinusoidal::Sinusoidal;
use gramotor::track::Track;

const CHANNELS: usize = 2;
const NUM_SECONDS: u64 = 9;
const SAMPLE_RATE: usize = 44_100;
const FRAMES_PER_SECOND: usize = 10;
const SAMPLES: usize = SAMPLE_RATE / FRAMES_PER_SECOND;

const GSR: &str = "
Sinusoidal 1#Sinusoidal_1 
> frequence 440
> phase 0

track 2#track_2
> I 1#Sinusoidal_1:O
> gain 1

mixer 5#mixer_5
> volume 1
> track 2#track_2
";

fn main() {
    {
    let mut motor = Gramotor::new();
    motor.init_session(GSR.to_string());
    for _ in 0..5 {
        let _ = motor.play();
    std::thread::sleep(std::time::Duration::from_secs(2));
        let _ = motor.pause();
    std::thread::sleep(std::time::Duration::from_secs(2));
    }
        let _ = motor.play();
    std::thread::sleep(std::time::Duration::from_secs(2));
        let _ = motor.stop();
    std::thread::sleep(std::time::Duration::from_secs(50));
    }
    let bus = EventBus::new_ref();
    let session = Session::new_ref(None, None, None, None, None);

    /*
    session.borrow().run();
    match run(&world, session.borrow().features_buffer()) {
        Ok(_) => {}
        e => {
            eprintln!("Example failed with the following: {:?}", e);
        }
    }

    match play_sin(&session) {
        Ok(_) => {}
        e => {
            eprintln!("Example failed with the following: {:?}", e);
        }
    }
     */
    match load_save_sessions() {
        Ok(_) => {}
        e => {
            eprintln!("Example failed with the following: {:?}", e);
        }
    }

    match play("play_sin.gsr") {
        Ok(_) => {}
        e => {
            eprintln!("playing play_sin.gsr failed : {:?}", e);
        }
    }
}

fn play(filename: &str) -> Result<(), failure::Error> {
    let session_description = String::from_utf8(fs::read(filename)?)?;
    let mut player = Player::new(session_description)?;
    player.start()?;
    std::thread::sleep(std::time::Duration::from_secs(1));

    for _ in 0..5 {
        std::thread::sleep(std::time::Duration::from_secs(1));
        player.pause()?;
        std::thread::sleep(std::time::Duration::from_secs(1));
        player.play()?;
    }

    player.stop()?;
    player.exit()?;

    Ok(())
}

fn load_save_sessions() -> Result<(), failure::Error> {
    let mut lss = Session::load_file("play_sin.gsr")?;
    lss.save_as("play_sin_dst.gsr")?;
    Ok(())
}

fn play_fuzz(session: &RSession) -> Result<(), failure::Error> {
    let fuzzface_uri = "http://guitarix.sourceforge.net/plugins/gx_fuzzface_#_fuzzface_";

    let fuzzface_tkr = session
        .borrow_mut()
        .add_talker(fuzzface_uri, None, None)?;

    let abs_sine_tkr = session
        .borrow_mut()
        .add_talker(sinusoidal::MODEL, None, None)?;

    fuzzface_tkr
        .borrow_mut()
        .set_ear_voice_by_tag("In", &abs_sine_tkr, 0)?;
    fuzzface_tkr
        .borrow_mut()
        .set_ear_value_by_tag("FUZZ", 2f32)?;
    fuzzface_tkr
        .borrow_mut()
        .set_ear_value_by_tag("LEVEL", 0.25f32)?;

    session.borrow().activate_talkers();

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

    session.borrow().deactivate_talkers();

    std::thread::sleep(std::time::Duration::from_secs(secs));
    Ok(())
}

fn play_sin(session: &RSession) -> Result<(), failure::Error> {
    let tkr = session
        .borrow_mut()
        .add_talker(sinusoidal::MODEL, None, None)?;

    session.borrow().activate_talkers();

    let mut po = Playback::new(CHANNELS, SAMPLES)?;
    po.open()?;
    let audio_buf = tkr.borrow_mut().voice(0).borrow().audio_buffer().unwrap();
    let mut tick: i64 = 0;
    let len = AudioFormat::chunk_size();
    let nb_iter = 10;
    let secs = ((nb_iter * len) / SAMPLE_RATE) as u64;
    println!("Will play sinusoidal for {} seconds", secs);

    for _ in 0..nb_iter {
        let ln = tkr.borrow_mut().talk(0, tick, len);
        po.write_mono(&audio_buf, ln)?;
        tick += ln as i64;
    }

    std::thread::sleep(std::time::Duration::from_secs(secs));
    session.borrow().deactivate_talkers();

    let track = Track::new();
    track.set_ear_voice_by_index(0, &tkr, 0)?;
    let rmixer = Mixer::new_ref(None, None);
    //    let mut mixer = rmixer.borrow_mut();
    rmixer.borrow_mut().add_track(track);
    rmixer.borrow_mut().add_output(Rc::new(RefCell::new(po)));
    session.borrow_mut().add_mixer(rmixer);
    session.borrow_mut().save_as("play_sin.gsr")?;
    Ok(())
}

fn play_progressive_sinusoidale(
    session: &RSession,
) -> Result<(), failure::Error> {
    let tkr = session.borrow_mut().add_talker(
        second_degree_frequency_progression::MODEL,
        None,
        None,
    )?;
    session.borrow().activate_talkers();

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

    session.borrow().deactivate_talkers();

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
