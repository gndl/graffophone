use talker::audio_format::AudioFormat;
use talker::ctalker;
use talker::dsp;
use talker::ear;
use talker::ear::Init;
use talker::identifier::Index;
use talker::talker::{CTalker, Talker, TalkerBase};
use talker::talker_handler::TalkerHandlerBase;

pub const MODEL: &str = "BoundedSquare";

pub struct BoundedSquare {
    sample_rate: f64,
    up: bool,
    period_part_end_idx: usize,
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
        base.add_ear(ear::cv(Some("roof"), -1000., 20000., 1., &Init::DefValue)?);
        base.add_ear(ear::cv(Some("floor"), -1000., 20000., 0., &Init::DefValue)?);

        base.add_cv_voice(Some("cv"), 0.);
        base.add_audio_voice(Some("au"), 0.);

        Ok(ctalker!(
            base,
            Self {
                sample_rate: AudioFormat::sample_rate() as f64,
                up: false,
                period_part_end_idx: 0,
            }
        ))
    }

    pub fn descriptor() -> TalkerHandlerBase {
        TalkerHandlerBase::builtin("Oscillator", MODEL, "Bounded Square")
    }
}

impl Talker for BoundedSquare {
    fn activate(&mut self) {
        self.up = false;
        self.period_part_end_idx = 0;
    }

    fn talk(&mut self, base: &TalkerBase, port: usize, tick: i64, len: usize) -> usize {
        let ln = base.listen(tick, len);
        let freq_buf = base.ear_cv_buffer(FREQ_EAR_INDEX);
        let ratio_buf = base.ear_audio_buffer(RATIO_EAR_INDEX);
        let roof_buf = base.ear_cv_buffer(ROOF_EAR_INDEX);
        let floor_buf = base.ear_cv_buffer(FLOOR_EAR_INDEX);
        let voice_buf = base.voice(port).audio_buffer();

        let mut up = self.up;
        let mut period_part_end_idx = self.period_part_end_idx;

        let mut gain_buf = roof_buf;
        let mut i: usize = 0;

        while i < ln {
            if i == period_part_end_idx {
                let freq = f64::EPSILON.max(freq_buf[i] as f64);
                let ratio = (ratio_buf[i] as f64 + 1.) * 0.5;
                let period = self.sample_rate / freq;

                if up {
                    gain_buf = floor_buf;
                    period_part_end_idx = i + (period * (1. - ratio)) as usize;
                }
                else {
                    gain_buf = roof_buf;
                    period_part_end_idx = i + (period * ratio) as usize;
                }
                up = !up;
            }

            let end_idx = ln.min(period_part_end_idx);

            while i < end_idx {
                voice_buf[i] = gain_buf[i];
                i += 1;
            }
        }

        if port == AUDIO_VOICE_PORT {
            dsp::audioize_buffer_by_clipping(voice_buf, 0, i);
        }

        self.up = up;
        self.period_part_end_idx = period_part_end_idx - ln;

        ln
    }
}
