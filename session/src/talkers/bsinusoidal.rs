use std::f32;
use std::f64::consts::PI;

use talker::audio_format::AudioFormat;
use talker::ctalker;
use talker::ear;
use talker::ear::Init;
use talker::identifier::Index;
use talker::talker::{CTalker, Talker, TalkerBase};
use talker::talker_handler::TalkerHandlerBase;
use talker::voice;

pub const MODEL: &str = "BSinusoidal";

pub struct Bsinusoidal {
    last_tick: i64,
    last_freq: f64,
    last_angle: f64,
}

const FREQ_EAR_INDEX: Index = 0;
const PHASE_EAR_INDEX: Index = 1;
const ROOF_EAR_INDEX: Index = 2;
const FLOOR_EAR_INDEX: Index = 3;

impl Bsinusoidal {
    pub fn new() -> Result<CTalker, failure::Error> {
        let mut base = TalkerBase::new("BSin", MODEL);

        base.add_ear(ear::cv(Some("freq"), 0., 20000., 440., &Init::DefValue)?);
        base.add_ear(ear::cv(Some("phase"), 0., 1000., 0., &Init::DefValue)?);
        base.add_ear(ear::cv(Some("roof"), -1000., 1000., 1., &Init::DefValue)?);
        base.add_ear(ear::cv(Some("floor"), -1000., 1000., 0., &Init::DefValue)?);

        base.add_voice(voice::cv(None, 0.));

        Ok(ctalker!(
            base,
            Self {
                last_tick: 0,
                last_freq: 0.,
                last_angle: 0.,
            }
        ))
    }

    pub fn descriptor() -> TalkerHandlerBase {
        TalkerHandlerBase::new("Oscillator", MODEL, "BSin")
    }
}

impl Talker for Bsinusoidal {
    fn talk(&mut self, base: &TalkerBase, port: usize, tick: i64, len: usize) -> usize {
        let ln = base.listen(tick, len);
        let freq_buf = base.ear_cv_buffer(FREQ_EAR_INDEX);
        let phase_buf = base.ear_cv_buffer(PHASE_EAR_INDEX);
        let roof_buf = base.ear_cv_buffer(ROOF_EAR_INDEX);
        let floor_buf = base.ear_cv_buffer(FLOOR_EAR_INDEX);
        let voice_buf = base.voice(port).audio_buffer();
        let c = AudioFormat::frequence_coef();

        let (mut last_freq, mut last_angle) = if self.last_tick == tick {
            (self.last_freq, self.last_angle)
        } else {
            let lf = -freq_buf[0] as f64;
            (lf, lf * c)
        };

        for i in 0..ln {
            let p = phase_buf[i] as f64 * PI;
            let rv = roof_buf[i] as f64;
            let fv = floor_buf[i] as f64;
            let a = last_angle + c * last_freq;

            let v = (a + p).sin();

            voice_buf[i] = ((((v * 0.5) + 0.5) * (rv - fv)) + fv) as f32;

            last_freq = freq_buf[i] as f64;
            last_angle = a;
        }

        self.last_freq = last_freq;
        self.last_angle = last_angle;
        self.last_tick = tick + ln as i64;

        ln
    }
}
