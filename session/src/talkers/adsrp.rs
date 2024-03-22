use std::f32;

use talker::audio_format::AudioFormat;
use talker::ctalker;
use talker::ear::Ear;
use talker::ear::Init;
use talker::ear::Set;
use talker::horn::PortType;
use talker::identifier::Index;
use talker::talker::{CTalker, RTalker, Talker, TalkerBase};
use talker::talker_handler::TalkerHandlerBase;
use talker::voice;

pub const MODEL: &str = "ADSRp";

#[derive(PartialEq, Clone, Copy)]
enum PlayStep {
    OutsideNote,
    AtNoteStart,
    InNote,
}
struct PlayerState {
    step: PlayStep,
    segment_num: usize,
    envelope_tick: i64,
    next_env_point_tick: i64,
    next_env_point_level: f32,
    a: f32,
    b: f32,
    prev_level: f32,
}
impl PlayerState {
    pub fn new() -> Self {
        Self {
            step: PlayStep::OutsideNote,
            segment_num: 0,
            envelope_tick: 0,
            next_env_point_tick: 0,
            next_env_point_level: 0.,
            a: 1.,
            b: 0.,
            prev_level: 0.,
        }
    }
}

pub struct ADSRp {
    sample_rate: f32,
    players_states: Vec<PlayerState>,
}

const ENVELOPE_EAR_INDEX: Index = 0;
const DURATION_HUM_INDEX: Index = 0;
const LEVEL_HUM_INDEX: Index = 1;

const PLAYERS_EAR_INDEX: Index = 1;
const TRIGGER_HUM_INDEX: Index = 0;
const GAIN_HUM_INDEX: Index = 1;

impl ADSRp {
    pub fn new() -> Result<CTalker, failure::Error> {
        let mut base = TalkerBase::new(MODEL, MODEL);

        let env_stem_set = Set::from_attributs(&vec![
            ("dur", PortType::Cv, 0., 400., 0.2, Init::DefValue),
            ("level", PortType::Audio, -1., 1., 0.5, Init::DefValue),
        ])?;

        let attack = Set::from_attributs(&vec![
            ("dur", PortType::Cv, 0., 400., 0.01, Init::DefValue),
            ("level", PortType::Audio, -1., 1., 1., Init::DefValue),
        ])?;

        let decay = Set::from_attributs(&vec![
            ("dur", PortType::Cv, 0., 400., 0.2, Init::DefValue),
            ("level", PortType::Audio, -1., 1., 0.6, Init::DefValue),
        ])?;

        let sustain = Set::from_attributs(&vec![
            ("dur", PortType::Cv, 0., 400., 0.6, Init::DefValue),
            ("level", PortType::Audio, -1., 1., 0.4, Init::DefValue),
        ])?;

        let release = Set::from_attributs(&vec![
            ("dur", PortType::Cv, 0., 400., 0.2, Init::DefValue),
            ("level", PortType::Audio, -1., 1., 0.0, Init::DefValue),
        ])?;

        let sets = vec![attack, decay, sustain, release];
        base.add_ear(Ear::new(
            Some("Envelope"),
            true,
            Some(env_stem_set),
            Some(sets),
        ));

        let player_stem_set = Set::from_attributs(&vec![
            ("trig", PortType::Cv, 0., 1., 0., Init::DefValue),
            ("gain", PortType::Audio, -1., 1., 1., Init::DefValue),
        ])?;

        base.add_ear(Ear::new(Some("Players"), true, Some(player_stem_set), None));

        Ok(ctalker!(
            base,
            Self {
                sample_rate: AudioFormat::sample_rate() as f32,
                players_states: Vec::new(),
            }
        ))
    }

    pub fn descriptor() -> TalkerHandlerBase {
        TalkerHandlerBase::builtin("Envelope", MODEL, MODEL)
    }
}

impl Talker for ADSRp {
    fn add_set_with_value_to_ear_update(
        &mut self,
        base: &TalkerBase,
        ear_idx: Index,
        hum_idx: Index,
        value: f32,
    ) -> Result<Option<TalkerBase>, failure::Error> {
        let mut new_base = base.clone();
        new_base.ear(ear_idx).add_set_with_value(hum_idx, value)?;

        if ear_idx == PLAYERS_EAR_INDEX {
            self.players_states.push(PlayerState::new());
            let mut voice = voice::audio(None, 0.);
            voice.set_associated_ear_set(ear_idx, new_base.ear(ear_idx).sets_len() - 1);
            new_base.add_voice(voice);
        }
        Ok(Some(new_base))
    }
    fn add_set_with_voice_to_ear_update(
        &mut self,
        base: &TalkerBase,
        ear_idx: Index,
        hum_idx: Index,
        voice_talker: &RTalker,
        port: usize,
    ) -> Result<Option<TalkerBase>, failure::Error> {
        let mut new_base = base.clone();
        new_base
            .ear(ear_idx)
            .add_set_with_voice(hum_idx, voice_talker, port)?;

        if ear_idx == PLAYERS_EAR_INDEX {
            self.players_states.push(PlayerState::new());
            let mut voice = voice::audio(None, 0.);
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

        if ear_idx == PLAYERS_EAR_INDEX {
            self.players_states.remove(set_idx);
        }

        Ok(Some(new_base))
    }

    fn talk(&mut self, base: &TalkerBase, port: usize, tick: i64, len: usize) -> usize {
        let envelope_ear = base.ear(ENVELOPE_EAR_INDEX);
        let player_ear = base.ear(PLAYERS_EAR_INDEX);
        let ln = player_ear.listen_set(tick, len, port);
        let trigger_buf = player_ear.get_set_hum_cv_buffer(port, TRIGGER_HUM_INDEX);
        let gain_buf = player_ear.get_set_hum_audio_buffer(port, GAIN_HUM_INDEX);

        let player_state = &mut self.players_states[port];

        let mut step = player_state.step;
        let mut segment_num = player_state.segment_num;
        let mut envelope_tick = player_state.envelope_tick;
        let mut next_env_point_tick = player_state.next_env_point_tick;
        let mut next_env_point_level = player_state.next_env_point_level;
        let mut a = player_state.a;
        let mut b = player_state.b;
        let mut prev_level = player_state.prev_level;

        let mut new_segment_num = segment_num;
        let env_segment_count = envelope_ear.sets_len();

        let voice_buf = base.voice(port).audio_buffer();
        /*
                if envelope_tick < prev_env_point_t {
                    new_segment_num = 0;
                    next_env_point_t = 0;

                    while new_segment_num < env_segment_count {
                        let duration_buf =
                            envelope_ear.get_set_hum_cv_buffer(new_segment_num, DURATION_HUM_INDEX);

                        prev_env_point_t = next_env_point_t;
                        let dur = (self.sample_rate * duration_buf[0]) as i64;

                        next_env_point_t += if dur > 0 { dur } else { 1 };

                        if envelope_tick >= prev_env_point_t && envelope_tick < next_env_point_t {
                            break;
                        }
                        new_segment_num += 1;
                    }
                }
        */
        let mut idx: usize = 0;

        while idx < ln {
            if step == PlayStep::OutsideNote {
                while idx < ln {
                    if trigger_buf[idx] == 0. {
                        voice_buf[idx] = 0.;
                    } else {
                        step = PlayStep::AtNoteStart;
                        break;
                    }
                    idx += 1;
                }
            } else {
                let next_note_search_start_idx = if step == PlayStep::AtNoteStart {
                    step = PlayStep::InNote;
                    segment_num = usize::MAX;
                    new_segment_num = 0;
                    next_env_point_tick = 0;
                    next_env_point_level = prev_level;
                    envelope_tick = 0;
                    idx + 1
                } else {
                    if envelope_tick >= next_env_point_tick && new_segment_num < env_segment_count {
                        new_segment_num += 1;
                    }
                    idx
                };

                if new_segment_num == env_segment_count {
                    step = PlayStep::OutsideNote;
                } else {
                    if new_segment_num != segment_num {
                        envelope_ear.listen_set(tick + idx as i64, 1, new_segment_num);
                        let duration_buf =
                            envelope_ear.get_set_hum_cv_buffer(new_segment_num, DURATION_HUM_INDEX);
                        let level_buf =
                            envelope_ear.get_set_hum_audio_buffer(new_segment_num, LEVEL_HUM_INDEX);

                        let prev_env_point_tick = next_env_point_tick;
                        let prev_env_point_level = next_env_point_level;

                        let dur = (self.sample_rate * duration_buf[0]) as i64;

                        next_env_point_tick += if dur > 0 { dur } else { 1 };
                        next_env_point_level = level_buf[0];

                        a = (next_env_point_level - prev_env_point_level)
                            / (next_env_point_tick - prev_env_point_tick) as f32;
                        b = next_env_point_level - a * next_env_point_tick as f32;

                        segment_num = new_segment_num;
                    }

                    let mut stop_idx =
                        usize::min(ln, idx + (next_env_point_tick - envelope_tick) as usize);

                    for i in next_note_search_start_idx..stop_idx {
                        if trigger_buf[i] != 0. {
                            step = PlayStep::AtNoteStart;
                            stop_idx = i;
                            break;
                        }
                    }

                    while idx < stop_idx {
                        prev_level = (a * (envelope_tick as f32) + b) * gain_buf[idx];
                        voice_buf[idx] = prev_level;
                        envelope_tick += 1;
                        idx += 1;
                    }
                }
            }
        }

        player_state.step = step;
        player_state.segment_num = segment_num;
        player_state.envelope_tick = envelope_tick;
        player_state.next_env_point_tick = next_env_point_tick;
        player_state.next_env_point_level = next_env_point_level;
        player_state.a = a;
        player_state.b = b;
        player_state.prev_level = prev_level;

        ln
    }
}
