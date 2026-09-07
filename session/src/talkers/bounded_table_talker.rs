use std::f32;

use talker::audio_format::AudioFormat;
use talker::ear;
use talker::ear::Init;
use talker::identifier::Index;
use talker::talker::TalkerBase;

pub struct BoundedTableTalker {
    freq_ear_index: Index,
    phase_ear_index: Index,
    roof_ear_index: Index,
    floor_ear_index: Index,
    tab_len: f32,
    tab_len_on_sr: f32,
    last_tick: i64,
    last_pos: f32,
    last_phase: f32,
}

impl BoundedTableTalker {
    pub fn new(base: &mut TalkerBase, tab_len: usize) -> Result<BoundedTableTalker, failure::Error> {
        let freq_ear_index = base.add_ear(ear::cv(Some("freq"), 0., 20000., 440., &Init::DefValue)?);
        let phase_ear_index = base.add_ear(ear::audio(Some("phase"), -1., 2., 0., &Init::DefValue)?);
        let roof_ear_index = base.add_ear(ear::cv(Some("roof"), -1000., 20000., 1., &Init::DefValue)?);
        let floor_ear_index = base.add_ear(ear::cv(Some("floor"), -1000., 20000., 0., &Init::DefValue)?);

        base.add_cv_voice(Some("cv"), 0.);
        base.add_audio_voice(Some("au"), 0.);

        let tab_len_on_sr = (tab_len as f64 / AudioFormat::sample_rate() as f64) as f32;

        Ok(Self {
            freq_ear_index,
            phase_ear_index,
            roof_ear_index,
            floor_ear_index,
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
        let freq_buf = base.ear_cv_buffer(self.freq_ear_index);
        let phase_buf = base.ear_audio_buffer(self.phase_ear_index);
        let roof_buf = base.ear_cv_buffer(self.roof_ear_index);
        let floor_buf = base.ear_cv_buffer(self.floor_ear_index);

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

            let rv = roof_buf[i] as f64;
            let fv = floor_buf[i] as f64;

            voice_buf[i] = ((((v as f64 * 0.5) + 0.5) * (rv - fv)) + fv) as f32;

            last_pos = pos;
            last_phase = phase;
        }

        self.last_pos = last_pos;
        self.last_phase = last_phase;
        self.last_tick = tick + ln as i64;

        ln
    }
}
