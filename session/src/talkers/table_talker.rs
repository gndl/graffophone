use std::f32;

use talker::audio_format::AudioFormat;
use talker::ear;
use talker::ear::Init;
use talker::talker::TalkerBase;

pub struct TableTalker {
    tab_len: f32,
    tab_len_on_sr: f32,
    last_tick: i64,
    last_pos: f32,
    last_phase: f32,
}

impl TableTalker {
    pub fn new(base: &mut TalkerBase, tab_len: usize) -> Result<TableTalker, failure::Error> {
        base.add_ear(ear::cv(Some("freq"), 0., 20000., 440., &Init::DefValue)?);
        base.add_ear(ear::audio(Some("phase"), -1., 2., 0., &Init::DefValue)?);
        base.add_ear(ear::audio(Some("gain"), -1., 1., 1., &Init::DefValue)?);

        base.add_audio_voice(None, 0.);

        let tab_len_on_sr = (tab_len as f64 / AudioFormat::sample_rate() as f64) as f32;

        Ok(Self {
            tab_len: tab_len as f32,
            tab_len_on_sr,
            last_tick: 0,
            last_pos: 0.,
            last_phase: 0.,
        })
    }

    pub fn talk(
        &mut self,
        base: &TalkerBase,
        port: usize,
        tick: i64,
        len: usize,
        tab: &[f32],
    ) -> usize {
        let ln = base.listen(tick, len);
        let freq_buf = base.ear_cv_buffer(0);
        let phase_buf = base.ear_audio_buffer(1);
        let gain_buf = base.ear_audio_buffer(2);
        let voice_buf = base.voice(port).audio_buffer();

        let phase_coef = self.tab_len * 0.5;
        let tab_len_on_sr = self.tab_len_on_sr;
        let mut last_pos = 0.;
        let mut last_phase = 0.;

        if self.last_tick == tick {
            last_pos = self.last_pos;
            last_phase = self.last_phase;
        }

        for i in 0..ln {
            let phase = phase_buf[i];

            if phase != last_phase {
                last_pos += (phase - last_phase) * phase_coef;

                if last_pos < 0. {
                    last_pos += self.tab_len;
                }
            }

            let freq = freq_buf[i];
            let pos = (last_pos + freq * tab_len_on_sr) % self.tab_len;
            let tab_idx = pos as usize;

            let v = if freq < 1. {
                let pv = tab[tab_idx];
                ((tab[tab_idx + 1] - pv) * pos.fract()) + pv
            } else {
                tab[tab_idx]
            };

            voice_buf[i] = v * gain_buf[i];
            last_pos = pos;
            last_phase = phase;
        }

        self.last_pos = last_pos;
        self.last_phase = last_phase;
        self.last_tick = tick + ln as i64;
        ln
    }
}
