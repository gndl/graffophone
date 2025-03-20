use std::f32;

use talker::audio_format::AudioFormat;
use talker::ctalker;
use talker::ear;
use talker::ear::Ear;
use talker::ear::Init;
use talker::ear::Set;
use talker::horn::PortType;
use talker::identifier::Index;
use talker::talker::{CTalker, Talker, TalkerBase};
use talker::talker_handler::TalkerHandlerBase;
use talker::voice;

pub const MODEL: &str = "EnvelopeShaper";

struct PlayerState {
    last_trigger: f32,
    env_idx: usize,
    last_out_gain: f32,
}
impl PlayerState {
    pub fn new() -> Self {
        Self {
            last_trigger: 0.,
            env_idx: 0,
            last_out_gain: 0.,
        }
    }
}

pub struct EnvelopeShaper {
    sample_rate: f64,
    chunk_size: usize,
    start_tick: i64,
    duration: usize,
    a: f32,
    b: f32,
    env: Vec<f32>,
    players_states: Vec<PlayerState>,
}

const SRC_EAR_INDEX: Index = 0;
const TIME_EAR_INDEX: Index = 1;
const DURATION_EAR_INDEX: Index = 2;
const A_EAR_INDEX: Index = 3;
const B_EAR_INDEX: Index = 4;
const PLAYERS_EAR_INDEX: Index = 5;

const TRIGGER_HUM_INDEX: Index = 0;
const GAIN_HUM_INDEX: Index = 1;

impl EnvelopeShaper {
    pub fn new(mut base: TalkerBase) -> Result<CTalker, failure::Error> {
        base.add_ear(ear::audio(None, -1., 1., 1., &Init::DefValue)?);
        base.add_ear(ear::cv(Some("time"), 0., 3600., 0., &Init::DefValue)?);
        base.add_ear(ear::cv(Some("dur"), 0., 400., 1., &Init::DefValue)?);
        base.add_ear(ear::audio(Some("a"), -1., 1., 0.5, &Init::DefValue)?);
        base.add_ear(ear::audio(Some("b"), -1., 1., 0.5, &Init::DefValue)?);

        let stem_set = Set::from_attributs(&vec![
            ("trig", PortType::Cv, 0., 1., 0., Init::Empty),
            ("gain", PortType::Audio, -1., 1., 1., Init::Empty),
        ])?;

        // let sets = vec![stem_set.clone()];
        // base.add_ear(Ear::new(Some("Players"), true, Some(stem_set), Some(sets)));
        base.add_ear(Ear::new(Some("Players"), true, Some(stem_set), None));

        //        base.add_voice(voice::audio(None, 0.));

        Ok(ctalker!(
            base,
            Self {
                sample_rate: AudioFormat::sample_rate() as f64,
                chunk_size: AudioFormat::chunk_size(),
                start_tick: 0,
                duration: 1,
                a: 0.,
                b: 0.,
                env: vec![0.],
                players_states: Vec::new(),
            }
        ))
    }

    pub fn descriptor() -> TalkerHandlerBase {
        TalkerHandlerBase::builtin("Envelope", MODEL, "Envelope Shaper")
    }
}

impl Talker for EnvelopeShaper {
    fn add_set_to_ear_update(
        &mut self,
        base: &TalkerBase,
        ear_idx: Index,
        hum_idx: Index,
        entree: ear::Entree,
    ) -> Result<Option<TalkerBase>, failure::Error> {
        let mut new_base = base.clone();
        new_base.ear(ear_idx).add_set(hum_idx, entree)?;

        let mut voice = voice::audio(None, 0., base.buffer_len());
        voice.set_associated_ear_set(ear_idx, new_base.ear(ear_idx).sets_len() - 1);
        new_base.add_voice(voice);

        self.players_states.push(PlayerState::new());

        Ok(Some(new_base))
    }
    fn sup_ear_set_update(
        &mut self,
        base: &TalkerBase,
        ear_idx: usize,
        set_idx: usize,
    ) -> Result<Option<TalkerBase>, failure::Error> {
        self.players_states.remove(set_idx);

        let mut new_base = base.clone();

        new_base.sup_ear_set_with_associated_voice(ear_idx, set_idx)?;

        Ok(Some(new_base))
    }

    fn talk(&mut self, base: &TalkerBase, port: usize, tick: i64, len: usize) -> usize {
        let ln = base.ear(PLAYERS_EAR_INDEX).listen_set(tick, len, port);
        let trigger_buf = base.ear_set_hum_cv_buffer(PLAYERS_EAR_INDEX, port, TRIGGER_HUM_INDEX);
        let gain_buf = base.ear_set_hum_audio_buffer(PLAYERS_EAR_INDEX, port, GAIN_HUM_INDEX);

        let player_state = &mut self.players_states[port];
        let mut last_trigger = player_state.last_trigger;
        let mut env_idx = player_state.env_idx;
        let mut last_out_gain = player_state.last_out_gain;

        let voice_buf = base.voice(port).audio_buffer();

        for i in 0..ln {
            let trigger = trigger_buf[i];

            if trigger != 0. && last_trigger == 0. {
                let t = tick + i as i64;

                let l = base.ear(TIME_EAR_INDEX).listen(t, 1);
                if l > 0 {
                    let start_tick = (base.ear_cv_buffer(TIME_EAR_INDEX)[0] as f64 * self.sample_rate) as i64;

                    let l = base.ear(DURATION_EAR_INDEX).listen(t, 1);
                    if l > 0 {
                        let duration =
                            (base.ear_cv_buffer(DURATION_EAR_INDEX)[0] as f64 * self.sample_rate) as usize;

                        let l = base.ear(A_EAR_INDEX).listen(t, 1);
                        if l > 0 {
                            let a = base.ear_cv_buffer(A_EAR_INDEX)[0];

                            let l = base.ear(B_EAR_INDEX).listen(t, 1);
                            if l > 0 {
                                let b = base.ear_cv_buffer(B_EAR_INDEX)[0];

                                if self.start_tick != start_tick
                                    || self.duration < duration
                                    || self.a != a
                                    || self.b != b
                                {
                                    if self.env.len() < duration {
                                        self.env.resize(duration, 0.);
                                    }

                                    let nb_chunk = duration / self.chunk_size;
                                    let reminder = duration % self.chunk_size;

                                    let mut e_i = 0;
                                    let mut src_t = start_tick;
                                    let chunk_size = self.chunk_size as i64;

                                    for _ in 0..nb_chunk {
                                        base.ear(SRC_EAR_INDEX).listen(src_t, self.chunk_size);
                                        let src_buf = base.ear_audio_buffer(SRC_EAR_INDEX);

                                        for src_idx in 0..self.chunk_size {
                                            self.env[e_i] = a * src_buf[src_idx] + b;
                                            e_i += 1;
                                        }
                                        src_t += chunk_size;
                                    }

                                    if reminder > 0 {
                                        base.ear(SRC_EAR_INDEX).listen(src_t, reminder);
                                        let src_buf = base.ear_audio_buffer(SRC_EAR_INDEX);

                                        for src_idx in 0..reminder {
                                            self.env[e_i] = a * src_buf[src_idx] + b;
                                            e_i += 1;
                                        }
                                    }
                                    self.start_tick = start_tick;
                                    self.duration = duration;
                                    self.a = a;
                                    self.b = b;
                                }
                            }
                        }
                    }
                }
                env_idx = 0;
            }

            if env_idx < self.duration {
                last_out_gain = self.env[env_idx] * gain_buf[i];
                env_idx += 1;
            } else {
                last_out_gain = last_out_gain * 0.9999;
            }
            voice_buf[i] = last_out_gain;
            last_trigger = trigger;
        }

        player_state.last_trigger = last_trigger;
        player_state.env_idx = env_idx;
        player_state.last_out_gain = last_out_gain;
        ln
    }
}
