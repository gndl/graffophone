use std::sync::atomic::{AtomicUsize, Ordering};

const CHANNELS: usize = 2;
const DEFAULT_SAMPLE_RATE: usize = 44_100;
static SAMPLE_RATE: AtomicUsize = AtomicUsize::new(DEFAULT_SAMPLE_RATE);
const FRAMES_PER_SECOND: usize = 10;
const CHUNK_SIZE: usize = 512;

pub struct AudioFormat {
    pub sample_rate: usize, // samples per second (in a single channel)
    pub channels: usize,    // number of audio channels
}

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
