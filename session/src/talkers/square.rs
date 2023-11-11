use std::f32;

use talker::audio_format::AudioFormat;
use talker::ctalker;
use talker::ear;
use talker::ear::Init;
use talker::identifier::Index;
use talker::talker::{CTalker, Talker, TalkerBase};
use talker::talker_handler::TalkerHandlerBase;
use talker::voice;

pub const MODEL: &str = "Square";

pub struct Square {
    next_rising_edge_tick: i64,
    next_falling_edge_tick: i64,
    gain: f32,
}

const FREQ_EAR_INDEX: Index = 0;
const RATIO_EAR_INDEX: Index = 1;
const GAIN_EAR_INDEX: Index = 2;

impl Square {
    pub fn new() -> Result<CTalker, failure::Error> {
        let mut base = TalkerBase::new("Square", MODEL);

        base.add_ear(ear::cv(Some("freq"), 0., 20000., 440., &Init::DefValue)?);
        base.add_ear(ear::audio(Some("ratio"), -1., 1., 0., &Init::DefValue)?);
        base.add_ear(ear::audio(Some("gain"), -1., 1., 1., &Init::DefValue)?);

        base.add_voice(voice::audio(None, 0.));

        Ok(ctalker!(
            base,
            Self {
                next_rising_edge_tick: 0,
                next_falling_edge_tick: 0,
                gain: 0.
            }
        ))
    }

    pub fn descriptor() -> TalkerHandlerBase {
        TalkerHandlerBase::builtin("Oscillator", MODEL, "Square")
    }
}

impl Talker for Square {
    fn talk(&mut self, base: &TalkerBase, port: usize, tick: i64, len: usize) -> usize {
        let freq_ear = base.ear(FREQ_EAR_INDEX);
        let freq_buf = base.ear_cv_buffer(FREQ_EAR_INDEX);
        let ratio_ear = base.ear(RATIO_EAR_INDEX);
        let ratio_buf = base.ear_audio_buffer(RATIO_EAR_INDEX);
        let gain_ear = base.ear(GAIN_EAR_INDEX);
        let gain_buf = base.ear_audio_buffer(GAIN_EAR_INDEX);
        let voice_buf = base.voice(port).audio_buffer();
        let sample_rate = AudioFormat::sample_rate() as f32;

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

        while i < len {
            if i == next_rising_edge_idx {
                let tck = tick + i as i64;

                freq_ear.listen(tck, 1);
                let f = freq_buf[0];

                ratio_ear.listen(tck, 1);
                let r = ratio_buf[0];

                gain_ear.listen(tck, 1);
                gain = gain_buf[0];

                let p = sample_rate / f;

                next_rising_edge_idx = i + p as usize;
                next_falling_edge_idx = i + (p * (r * 0.5 + 0.5)) as usize;
            }

            let roof_end = usize::min(len, next_falling_edge_idx);

            for _ in i..roof_end {
                voice_buf[i] = gain;
                i += 1;
            }

            let floor_end = usize::min(len, next_rising_edge_idx);

            for _ in i..floor_end {
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
