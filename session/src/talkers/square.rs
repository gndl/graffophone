use std::f32;

use talker::audio_format::AudioFormat;
use talker::ctalker;
use talker::ear;
use talker::ear::Init;
use talker::identifier::Index;
use talker::talker::{CTalker, Talker, TalkerBase};
use talker::talker_handler::TalkerHandlerBase;

pub const MODEL: &str = "Square";

pub struct Square {
    sample_rate: f64,
    next_rising_edge_tick: i64,
    next_falling_edge_tick: i64,
    gain: f32,
}

const FREQ_EAR_INDEX: Index = 0;
const RATIO_EAR_INDEX: Index = 1;
const GAIN_EAR_INDEX: Index = 2;

impl Square {
    pub fn new(mut base: TalkerBase) -> Result<CTalker, failure::Error> {
        base.add_ear(ear::cv(Some("freq"), 0., 20000., 440., &Init::DefValue)?);
        base.add_ear(ear::audio(Some("ratio"), -1., 1., 0., &Init::DefValue)?);
        base.add_ear(ear::cv(Some("gain"), -1., 4., 1., &Init::DefValue)?);

        base.add_audio_voice(None, 0.);

        Ok(ctalker!(
            base,
            Self {
                sample_rate: AudioFormat::sample_rate() as f64,
                next_rising_edge_tick: 0,
                next_falling_edge_tick: 0,
                gain: 0.
            }
        ))
    }

    pub fn descriptor() -> TalkerHandlerBase {
        TalkerHandlerBase::builtin("Oscillator", MODEL, MODEL)
    }
}

impl Talker for Square {
    fn activate(&mut self) {
        self.next_rising_edge_tick = 0;
        self.next_falling_edge_tick = 0;
    }

    fn talk(&mut self, base: &TalkerBase, port: usize, tick: i64, len: usize) -> usize {
        let ln = base.listen(tick, len);
        let freq_buf = base.ear_cv_buffer(FREQ_EAR_INDEX);
        let ratio_buf = base.ear_audio_buffer(RATIO_EAR_INDEX);
        let gain_buf = base.ear_cv_buffer(GAIN_EAR_INDEX);
        let voice_buf = base.voice(port).audio_buffer();

        let mut next_rising_edge_idx = if self.next_rising_edge_tick < tick {
            0
        } else {
            (self.next_rising_edge_tick - tick) as usize
        };
        let mut next_falling_edge_idx = if self.next_falling_edge_tick < tick {
            0
        } else {
            (self.next_falling_edge_tick - tick) as usize
        };
        let mut gain = self.gain;

        let mut i: usize = 0;

        while i < ln {
            if i == next_rising_edge_idx {
                let freq = f64::EPSILON.max(freq_buf[i] as f64);
                let ratio = (ratio_buf[i] as f64 + 1.) * 0.5;
                gain = gain_buf[i];

                let period = self.sample_rate / freq;

                next_rising_edge_idx = i + period as usize;
                next_falling_edge_idx = i + (period * ratio) as usize;
            }

            let roof_end = ln.min(next_falling_edge_idx);

            while i < roof_end {
                voice_buf[i] = gain;
                i += 1;
            }

            let floor_end = ln.min(next_rising_edge_idx);

            while i < floor_end {
                voice_buf[i] = -gain;
                i += 1;
            }
        }

        self.next_rising_edge_tick = next_rising_edge_idx as i64 + tick;
        self.next_falling_edge_tick = next_falling_edge_idx as i64 + tick;
        self.gain = gain;

        i
    }
}
