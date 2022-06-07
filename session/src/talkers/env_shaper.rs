use std::f32;

use talker::audio_format::AudioFormat;
use talker::ctalker;
use talker::ear;
use talker::ear::Ear;
use talker::ear::Init;
use talker::ear::Set;
use talker::horn::PortType;
use talker::identifier::Index;
use talker::talker::{CTalker, RTalker, Talker, TalkerBase};
use talker::talker_handler::TalkerHandlerBase;
use talker::voice;

pub const MODEL: &str = "EnvShaper";

struct PlayerState {
    last_cv: f32,
    env_idx: usize,
}
impl PlayerState {
    pub fn new(last_cv: f32, env_idx: usize) -> Self {
        Self { last_cv, env_idx }
    }
}

pub struct EnvShaper {
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

impl EnvShaper {
    pub fn new() -> Result<CTalker, failure::Error> {
        let mut base = TalkerBase::new("EnvShaper", MODEL);

        base.add_ear(ear::audio(None, -1., 1., 1., &Init::DefValue)?);
        base.add_ear(ear::cv(Some("time"), 0., 3600., 0., &Init::DefValue)?);
        base.add_ear(ear::cv(Some("dur"), 0., 1800., 1., &Init::DefValue)?);
        base.add_ear(ear::audio(Some("a"), -1., 1., 0.5, &Init::DefValue)?);
        base.add_ear(ear::audio(Some("b"), -1., 1., 0.5, &Init::DefValue)?);
        //      base.add_ear(ear::cv(Some("trigger"), 0., 20000., 0., &Init::DefValue)?);
        //        base.add_ear(ear::audio(Some("gain"), -1., 1., 1., &Init::DefValue)?);

        let stem_set = Set::from_attributs(&vec![
            ("trigger", PortType::Cv, 0., 20000., 0., Init::DefValue),
            ("gain", PortType::Audio, -1., 1., 1., Init::DefValue),
        ])?;

        // let sets = vec![stem_set.clone()];
        // base.add_ear(Ear::new(Some("Players"), true, Some(stem_set), Some(sets)));
        base.add_ear(Ear::new(Some("Players"), true, Some(stem_set), None));

        //        base.add_voice(voice::audio(None, 0.));

        Ok(ctalker!(
            base,
            Self {
                start_tick: 0,
                duration: 1,
                a: 0.,
                b: 0.,
                env: vec![0.],
                players_states: Vec::new(),
                //                players_states: vec![PlayerState::new(0., 0)],
            }
        ))
    }

    pub fn descriptor() -> TalkerHandlerBase {
        TalkerHandlerBase::new("envelope", MODEL, "EnvShaper")
    }
}

impl Talker for EnvShaper {
    fn add_set_value_to_ear_update(
        &mut self,
        base: &TalkerBase,
        ear_idx: Index,
        hum_idx: Index,
        value: f32,
    ) -> Result<Option<TalkerBase>, failure::Error> {
        self.players_states.push(PlayerState::new(0., 0));

        let mut new_base = base.clone();

        new_base.ear(ear_idx).add_set_value(hum_idx, value)?;
        new_base.add_voice(voice::audio(None, 0.));

        Ok(Some(new_base))
    }
    fn add_set_voice_to_ear_update(
        &mut self,
        base: &TalkerBase,
        ear_idx: Index,
        hum_idx: Index,
        voice_talker: &RTalker,
        port: usize,
    ) -> Result<Option<TalkerBase>, failure::Error> {
        self.players_states.push(PlayerState::new(0., 0));

        let mut new_base = base.clone();

        new_base
            .ear(ear_idx)
            .add_set_voice(hum_idx, voice_talker, port)?;
        new_base.add_voice(voice::audio(None, 0.));

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

        new_base.ear(ear_idx).sup_set(set_idx)?;
        new_base.sup_voice(set_idx);

        Ok(Some(new_base))
    }

    fn talk(&mut self, base: &TalkerBase, port: usize, tick: i64, len: usize) -> usize {
        let ln = base.ear(PLAYERS_EAR_INDEX).listen_set(tick, len, port);
        let cv_buf = base.ear_set_hum_cv_buffer(PLAYERS_EAR_INDEX, port, TRIGGER_HUM_INDEX);
        let gain_buf = base.ear_set_hum_audio_buffer(PLAYERS_EAR_INDEX, port, GAIN_HUM_INDEX);

        let mut player_state = &mut self.players_states[port];
        let mut last_cv = player_state.last_cv;
        let mut env_idx = player_state.env_idx;

        let voice_buf = base.voice(port).audio_buffer();

        for i in 0..ln {
            let cv = cv_buf[i];

            if cv == 0. {
                if i == 0 {
                    voice_buf[0] = voice_buf[len - 1];
                } else {
                    voice_buf[i] = voice_buf[i - 1];
                }
            } else {
                if last_cv == 0. {
                    let t = tick + i as i64;
                    let sr = AudioFormat::sample_rate() as f64;

                    let l = base.ear(TIME_EAR_INDEX).listen(t, 1);
                    if l > 0 {
                        let start_tick = (base.ear_cv_buffer(TIME_EAR_INDEX)[0] as f64 * sr) as i64;

                        let l = base.ear(DURATION_EAR_INDEX).listen(t, 1);
                        if l > 0 {
                            let duration =
                                (base.ear_cv_buffer(DURATION_EAR_INDEX)[0] as f64 * sr) as usize;

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

                                        let chunk_size = AudioFormat::chunk_size();
                                        let nb_chunk = duration / chunk_size;
                                        let reminder = duration % chunk_size;

                                        let mut e_i = 0;
                                        let mut src_t = start_tick;

                                        for _ in 0..nb_chunk {
                                            base.ear(SRC_EAR_INDEX).listen(src_t, chunk_size);
                                            let src_buf = base.ear_cv_buffer(SRC_EAR_INDEX);

                                            for src_idx in 0..chunk_size {
                                                self.env[e_i] = a * src_buf[src_idx] + b;
                                                e_i += 1;
                                            }
                                            src_t += duration as i64;
                                        }

                                        if reminder > 0 {
                                            base.ear(SRC_EAR_INDEX).listen(src_t, reminder);
                                            let src_buf = base.ear_cv_buffer(SRC_EAR_INDEX);

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
                    voice_buf[i] = self.env[env_idx] * gain_buf[i];
                    env_idx += 1;
                } else {
                    if i == 0 {
                        voice_buf[0] = voice_buf[len - 1] * 0.9999;
                    } else {
                        voice_buf[i] = voice_buf[i - 1] * 0.9999;
                    }
                }
            }
            last_cv = cv;
        }

        player_state.last_cv = last_cv;
        player_state.env_idx = env_idx;
        //        self.players_states[port] = PlayerState::new(last_cv, env_idx);
        ln
    }
}
