use std::f64::consts::PI;
use std::sync::atomic::{AtomicUsize, Ordering};

const CHANNELS: usize = 2;
const DEFAULT_SAMPLE_RATE: usize = 48000; // 44_100;
static SAMPLE_RATE: AtomicUsize = AtomicUsize::new(DEFAULT_SAMPLE_RATE);
//const FRAMES_PER_SECOND: usize = 10;
pub const DEFAULT_CHUNK_SIZE: usize = 2048; //4410;
static DYNAMIC_CHUNK_SIZE: AtomicUsize = AtomicUsize::new(DEFAULT_CHUNK_SIZE);

pub const MIN_AUDIO: f32 = -0.99999;
pub const MAX_AUDIO: f32 = 0.99999;
pub const DEF_AUDIO: f32 = 0.;
pub const MIN_CONTROL: f32 = 0.;
pub const MAX_CONTROL: f32 = 20000.;
pub const DEF_CONTROL: f32 = 0.;
pub const MIN_CV: f32 = 0.;
pub const MAX_CV: f32 = 20000.;
pub const DEF_CV: f32 = 0.;

pub struct AudioFormat {
    pub sample_rate: usize, // samples per second (in a single channel)
    pub channels: usize,    // number of audio channels
}

impl AudioFormat {
    pub const CHUNK_SIZE: usize = 2048; //4410;
    pub const MIN_AUDIO: f32 = MIN_AUDIO;
    pub const MAX_AUDIO: f32 = MAX_AUDIO;
    pub const DEF_AUDIO: f32 = DEF_AUDIO;

    pub fn default() -> AudioFormat {
        AudioFormat {
            sample_rate: DEFAULT_SAMPLE_RATE,
            channels: CHANNELS,
        }
    }
    pub fn sample_rate() -> usize {
        SAMPLE_RATE.load(Ordering::Relaxed)
    }
    pub fn set_sample_rate(sample_rate: usize) {
        SAMPLE_RATE.store(sample_rate, Ordering::Relaxed);
    }
    pub fn chunk_size() -> usize {
        DYNAMIC_CHUNK_SIZE.load(Ordering::Relaxed)
    }
    pub fn set_chunk_size(chunk_size: usize) {
        DYNAMIC_CHUNK_SIZE.store(chunk_size, Ordering::Relaxed);
    }
    pub fn frequence_coef() -> f64 {
        (PI * 2.0) / SAMPLE_RATE.load(Ordering::Relaxed) as f64
    }
}
