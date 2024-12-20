use std::f32;

use talker::ctalker;
use talker::ear::Ear;
use talker::ear::Init;
use talker::ear::Set;
use talker::horn::PortType;
use talker::identifier::Index;
use talker::talker::{CTalker, Talker, TalkerBase};
use talker::talker_handler::TalkerHandlerBase;
use talker::voice;

pub const MODEL: &str = "DynamicModulator";

struct State {
    prev_output: f32,
}
impl State {
    pub fn new() -> Self {
        Self {
            prev_output: 0.,
        }
    }
}

const INPUT_EAR_INDEX: Index = 0;
const IN_HUM_INDEX: Index = 0;
const GAIN_HUM_INDEX: Index = 1;

pub struct DynamicModulator {
    states: Vec<State>,
}
impl DynamicModulator {
    pub fn new(mut base: TalkerBase) -> Result<CTalker, failure::Error> {
        let stem_set = Set::from_attributs(&vec![
            ("in", PortType::Audio, -1., 1., 0., Init::DefValue),
            ("gain", PortType::Audio, -1., 1., 1., Init::DefValue),
        ])?;

        base.add_ear(Ear::new(Some("inputs"), true, Some(stem_set), None));

        Ok(ctalker!(base, Self {
            states: Vec::new(),
        }))
    }

    pub fn descriptor() -> TalkerHandlerBase {
        TalkerHandlerBase::builtin("Modulator", MODEL, "Dynamic Modulator")
    }
}

impl Talker for DynamicModulator {
    fn add_set_to_ear_update(
        &mut self,
        base: &TalkerBase,
        ear_idx: Index,
        hum_idx: Index,
        entree: talker::ear::Entree,
    ) -> Result<Option<TalkerBase>, failure::Error> {
        let mut new_base = base.clone();
        new_base.ear(ear_idx).add_set(hum_idx, entree)?;

        if ear_idx == INPUT_EAR_INDEX {
            self.states.push(State::new());
            let mut voice = voice::audio(None, 0., base.buffer_len());
            voice.set_associated_ear_set(ear_idx, new_base.ear(ear_idx).sets_len() - 1);
            new_base.add_voice(voice);
        }
        Ok(Some(new_base))
    }
    fn sup_ear_set_update(
        &mut self,
        base: &TalkerBase,
        ear_idx: usize,
        set_idx: usize,
    ) -> Result<Option<TalkerBase>, failure::Error> {
        let mut new_base = base.clone();
        new_base.sup_ear_set_with_associated_voice(ear_idx, set_idx)?;

        if ear_idx == INPUT_EAR_INDEX {
            self.states.remove(set_idx);
        }

        Ok(Some(new_base))
    }

    fn talk(&mut self, base: &TalkerBase, port: usize, tick: i64, len: usize) -> usize {
        let ear = base.ear(INPUT_EAR_INDEX);
        let ln = ear.listen_set(tick, len, port);

        let input_buf = ear.get_set_hum_audio_buffer(port, IN_HUM_INDEX);
        let gain_buf = ear.get_set_hum_audio_buffer(port, GAIN_HUM_INDEX);

        let state = &mut self.states[port];

        let mut output = state.prev_output;

        let voice_buf = base.voice(port).audio_buffer();

        for i in 0..ln {
            let input = input_buf[i];
            let gain = gain_buf[i];

            output = output + ((input - output) * (gain + 1.));

            voice_buf[i] = output;
        }
        state.prev_output = output;

        ln
    }
}
