use std::f64::consts::PI;
use std::sync::atomic::{AtomicUsize, Ordering};

const CHANNELS: usize = 2;
const DEFAULT_SAMPLE_RATE: usize = 44_100;
static SAMPLE_RATE: AtomicUsize = AtomicUsize::new(DEFAULT_SAMPLE_RATE);
//const FRAMES_PER_SECOND: usize = 10;
const DEFAULT_CHUNK_SIZE: usize = 4410;
static CHUNK_SIZE: AtomicUsize = AtomicUsize::new(DEFAULT_CHUNK_SIZE);

pub struct AudioFormat {
    pub sample_rate: usize, // samples per second (in a single channel)
    pub channels: usize,    // number of audio channels
}

impl AudioFormat {
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
        CHUNK_SIZE.load(Ordering::Relaxed)
    }
    pub fn set_chunk_size(chunk_size: usize) {
        CHUNK_SIZE.store(chunk_size, Ordering::Relaxed);
    }
    pub fn frequence_coef() -> f64 {
        (PI * 2.0) / SAMPLE_RATE.load(Ordering::Relaxed) as f64
    }
}
