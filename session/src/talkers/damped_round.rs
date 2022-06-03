use std::f32;

use talker::audio_format::AudioFormat;
use talker::ctalker;
use talker::ear;
use talker::ear::Init;
use talker::talker::{CTalker, Talker, TalkerBase};
use talker::talker_handler::TalkerHandlerBase;
use talker::voice;

use tables::round;

pub const MODEL: &str = "Round";

pub struct Round {
    tab_len_on_sr: f32,
    last_tick: i64,
    last_pos: i64,
    last_phase: f32,
}

impl Round {
    pub fn new() -> Result<CTalker, failure::Error> {
        let mut base = TalkerBase::new("Round", MODEL);

        base.add_ear(ear::cv(Some("freq"), 0., 20000., 440., &Init::DefValue)?);
        base.add_ear(ear::audio(Some("phase"), -1., 1., 0., &Init::DefValue)?);
        base.add_ear(ear::audio(Some("gain"), -1., 1., 1., &Init::DefValue)?);

        base.add_voice(voice::audio(None, 0.));

        let tab_len_on_sr = (round::LEN as f64 / AudioFormat::sample_rate() as f64) as f32;

        Ok(ctalker!(
            base,
            Self {
                tab_len_on_sr,
                last_tick: 0,
                last_pos: 0,
                last_phase: 0.,
            }
        ))
    }

    pub fn descriptor() -> TalkerHandlerBase {
        TalkerHandlerBase::new("Oscillator", MODEL, "Round")
    }
}
const TAB_LEN: i64 = round::LEN as i64;

impl Talker for Round {
    fn talk(&mut self, base: &TalkerBase, port: usize, tick: i64, len: usize) -> usize {
        let ln = base.listen(tick, len);
        let freq_buf = base.ear_cv_buffer(0);
        let phase_buf = base.ear_audio_buffer(1);
        let gain_buf = base.ear_audio_buffer(2);
        let voice_buf = base.voice(port).audio_buffer();

        let phase_coef = round::LEN as f32 * 0.5;
        let tab_len_on_sr = self.tab_len_on_sr;
        let mut last_pos = 0;
        let mut last_phase = 0.;

        if self.last_tick == tick {
            last_pos = self.last_pos;
            last_phase = self.last_phase;
        }

        for i in 0..ln {
            let gain = gain_buf[i];

            if gain == 0. {
                last_pos = 0;

                if i > 0 {
                    voice_buf[i] = voice_buf[i - 1] * 0.95;
                } else {
                    voice_buf[i] = 0.;
                }
            } else {
                let phase = phase_buf[i];

                if phase != last_phase {
                    last_pos += ((phase - last_phase) * phase_coef) as i64;

                    if last_pos < 0 {
                        last_pos += TAB_LEN;
                    }
                }

                let pos = last_pos + (freq_buf[i] * tab_len_on_sr) as i64;
                let tab_idx = pos % TAB_LEN;

                voice_buf[i] = round::TAB[tab_idx as usize] * gain;
                last_pos = tab_idx;
                last_phase = phase;
            }
        }

        self.last_pos = last_pos;
        self.last_phase = last_phase;
        self.last_tick = tick + ln as i64;
        ln
    }
}
