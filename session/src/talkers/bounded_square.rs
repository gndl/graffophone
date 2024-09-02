use std::f32;

use talker::audio_format::AudioFormat;
use talker::ctalker;
use talker::dsp;
use talker::ear;
use talker::ear::Init;
use talker::identifier::Identifiable;
use talker::identifier::Index;
use talker::talker::{CTalker, Talker, TalkerBase};
use talker::talker_handler::TalkerHandlerBase;

pub const MODEL: &str = "BoundedSquare";

pub struct BoundedSquare {
    next_rising_edge_tick: i64,
    next_falling_edge_tick: i64,
}

const FREQ_EAR_INDEX: Index = 0;
const RATIO_EAR_INDEX: Index = 1;
const ROOF_EAR_INDEX: Index = 2;
const FLOOR_EAR_INDEX: Index = 3;

const AUDIO_VOICE_PORT: usize = 1;

impl BoundedSquare {
    pub fn new(mut base: TalkerBase) -> Result<CTalker, failure::Error> {
        base.add_ear(ear::cv(Some("freq"), 0., 20000., 440., &Init::DefValue)?);
        base.add_ear(ear::audio(Some("ratio"), -1., 1., 0., &Init::DefValue)?);
        base.add_ear(ear::cv(Some("roof"), -1000., 1000., 1., &Init::DefValue)?);
        base.add_ear(ear::cv(Some("floor"), -1000., 1000., 0., &Init::DefValue)?);

        base.add_cv_voice(Some("cv"), 0.);
        base.add_audio_voice(Some("au"), 0.);

        Ok(ctalker!(
            base,
            Self {
                next_rising_edge_tick: 0,
                next_falling_edge_tick: 0,
            }
        ))
    }

    pub fn descriptor() -> TalkerHandlerBase {
        TalkerHandlerBase::builtin("Oscillator", MODEL, "Bounded Square")
    }
}

impl Talker for BoundedSquare {
    fn talk(&mut self, base: &TalkerBase, port: usize, tick: i64, len: usize) -> usize {
        let freq_ear = base.ear(FREQ_EAR_INDEX);
        let freq_buf = base.ear_cv_buffer(FREQ_EAR_INDEX);
        let ratio_ear = base.ear(RATIO_EAR_INDEX);
        let ratio_buf = base.ear_audio_buffer(RATIO_EAR_INDEX);
        let roof_ear = base.ear(ROOF_EAR_INDEX);
        let roof_buf = base.ear_cv_buffer(ROOF_EAR_INDEX);
        let floor_ear = base.ear(FLOOR_EAR_INDEX);
        let floor_buf = base.ear_cv_buffer(FLOOR_EAR_INDEX);
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

        let mut i: usize = 0;

        while i < len {
            let t = tick + i as i64;

            if i == next_rising_edge_idx {
                freq_ear.listen(t, 1);
                let f = freq_buf[0];
                ratio_ear.listen(t, 1);
                let r = ratio_buf[0];
                let p = sample_rate / f;

                next_rising_edge_idx = i + p as usize;
                next_falling_edge_idx = i + (p * (r * 0.5 + 0.5)) as usize;
            }

            let roof_end = usize::min(len, next_falling_edge_idx);

            if i < roof_end {
                let roof_len = roof_end - i;
                let obtained_roof_len = roof_ear.listen(t, roof_len);

                for j in 0..obtained_roof_len {
                    voice_buf[i] = roof_buf[j];
                    i += 1;
                }
                if obtained_roof_len < roof_len {
                    break;
                }
            }

            let floor_end = usize::min(len, next_rising_edge_idx);

            if i < floor_end {
                let floor_len = floor_end - i;
                let obtained_floor_len =
                    floor_ear.listen(tick + next_falling_edge_idx as i64, floor_len);

                for j in 0..obtained_floor_len {
                    voice_buf[i] = floor_buf[j];
                    i += 1;
                }
                if obtained_floor_len < floor_len {
                    break;
                }
            }
        }

        if port == AUDIO_VOICE_PORT {
            dsp::audioize_buffer_by_clipping(voice_buf, 0, i);
        }

        self.next_rising_edge_tick = next_rising_edge_idx as i64 + tick;
        self.next_falling_edge_tick = next_falling_edge_idx as i64 + tick;

        i
    }
}
