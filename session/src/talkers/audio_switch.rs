use talker::audio_format::AudioFormat;
use talker::ctalker;
use talker::dsp;
use talker::ear::{self, Ear, Init, Set};
use talker::horn::PortType;
use talker::identifier::Index;
use talker::talker::{CTalker, Talker, TalkerBase};
use talker::talker_handler::TalkerHandlerBase;

pub const MODEL: &str = "AudioSwitch";

#[derive(Debug)]
pub struct AudioSwitch {
    sample_rate: usize, input_index: usize, input_times: usize, vm2: f32, vm1: f32,
}

const TRIGGER_EAR_INDEX: Index = 0;
const INPUTS_EAR_INDEX: Index = 1;
const INPUT_HUM_INDEX: Index = 0;
const TIMES_HUM_INDEX: Index = 1;

impl AudioSwitch {
    pub fn new(mut base: TalkerBase) -> Result<CTalker, failure::Error> {
        base.add_ear(ear::cv(Some("trig"), 0., 1., 0., &Init::DefValue)?);

        let in_stem_set = Set::from_attributs(&vec![
            ("", PortType::Audio, -1., 1., 0., Init::DefValue),
            ("times", PortType::Control, 1., 100., 1., Init::DefValue),
        ])?;
        base.add_ear(Ear::new(None, true, Some(in_stem_set), None));

        base.add_audio_voice(None, 0.);

        Ok(ctalker!(
            base,
            Self {
                sample_rate: AudioFormat::sample_rate(), input_index: 0, input_times: 1, vm2: 0., vm1: 0.,
            }
        ))
    }

    pub fn descriptor() -> TalkerHandlerBase {
        TalkerHandlerBase::builtin("Sequencer", MODEL, "Audio Switch")
    }
}

impl Talker for AudioSwitch {
    fn activate(&mut self) {
        self.sample_rate = AudioFormat::sample_rate();
        self.input_index = 0;
        self.input_times = 1;
        self.vm2 = 0.;
        self.vm1 = 0.;
    }

    fn sup_ear_set_update(
        &mut self,
        base: &TalkerBase,
        ear_idx: usize,
        set_idx: usize,
    ) -> Result<Option<TalkerBase>, failure::Error> {
        base.ear(ear_idx).sup_set(set_idx)?;

        if self.input_index > 0 && self.input_index >= base.ear(ear_idx).sets_len() {
            self.input_index -= 1;
        }

        Ok(None)
    }

    fn talk(&mut self, base: &TalkerBase, port: usize, tick: i64, len: usize) -> usize {
        let trigger_ear = base.ear(TRIGGER_EAR_INDEX);
        let inputs_ear = base.ear(INPUTS_EAR_INDEX);
        let inputs_count = inputs_ear.sets_len();

        let voice_buf = base.voice(port).audio_buffer();

        if inputs_count == 0 {
            voice_buf.fill(0.);
            len
        }
        else {
            let ln = trigger_ear.listen(tick, len);
            let fade_edge = ln - dsp::fade_len(self.sample_rate);

            let trigger_buf = trigger_ear.get_cv_buffer();

            let mut in_start_idx = 0;
            let last_input_index = inputs_count - 1;
            let mut commuting = false;

            if tick == 0 {
                let _ = inputs_ear.listen_set_hum(0, 1, self.input_index, TIMES_HUM_INDEX);
                self.input_times = inputs_ear.get_set_hum_control_value(self.input_index, TIMES_HUM_INDEX) as usize;
            }

            for trig_idx in 0..ln {
                if trigger_buf[trig_idx] > 0.5 && (trig_idx > 0 || tick > 0) {
                    self.input_times -= 1;

                    if self.input_times == 0 {
                        let idx = trig_idx.min(fade_edge);

                        if idx > in_start_idx {
                            let in_tick = tick + in_start_idx as i64;
                            let in_len = idx - in_start_idx;

                            let in_ln = inputs_ear.listen_set_hum(in_tick, in_len, self.input_index, INPUT_HUM_INDEX);
                            let input_buf = inputs_ear.get_set_hum_audio_buffer(self.input_index, INPUT_HUM_INDEX);

                            for i in 0..in_ln {
                                voice_buf[in_start_idx + i] = input_buf[i];
                            }

                            if commuting {
                                dsp::recoveryless_fade_buffer(self.sample_rate, voice_buf, in_start_idx, self.vm2, self.vm1);
                            }
                            self.vm1 = voice_buf[idx - 1];
                            self.vm2 = if idx > 1 { voice_buf[idx - 2] } else { self.vm1 };
                        }
                        in_start_idx = idx;

                        self.input_index = if self.input_index < last_input_index { self.input_index + 1 } else { 0 };
                        commuting = true;

                        let _ = inputs_ear.listen_set_hum(tick + trig_idx as i64, 1, self.input_index, TIMES_HUM_INDEX);
                        self.input_times = inputs_ear.get_set_hum_control_value(self.input_index, TIMES_HUM_INDEX) as usize;
                    }
                }
            }

            let in_tick = tick + in_start_idx as i64;
            let in_len = ln - in_start_idx;

            let in_ln = inputs_ear.listen_set_hum(in_tick, in_len, self.input_index, INPUT_HUM_INDEX);
            let input_buf = inputs_ear.get_set_hum_audio_buffer(self.input_index, INPUT_HUM_INDEX);

            for i in 0..in_ln {
                voice_buf[in_start_idx + i] = input_buf[i];
            }

            if commuting {
                dsp::recoveryless_fade_buffer(self.sample_rate, voice_buf, in_start_idx, self.vm2, self.vm1);
            }
            let idx = in_start_idx + in_ln;
            self.vm1 = if idx > 0 { voice_buf[idx - 1] } else { self.vm1 };
            self.vm2 = if idx > 1 { voice_buf[idx - 2] } else { self.vm2 };

            ln
        }
    }
}
