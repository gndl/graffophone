use std::f32;

use talker::audio_format::AudioFormat;
use talker::ctalker;
use talker::ear::Ear;
use talker::ear::Init;
use talker::ear::Set;
use talker::horn::PortType;
use talker::identifier::Index;
use talker::talker::{CTalker, Talker, TalkerBase};
use talker::talker_handler::TalkerHandlerBase;
use talker::voice;

pub const MODEL: &str = "SpeedModulator";

#[derive(Debug, PartialEq, Clone, Copy)]
struct Point {
    speed_tick: i64,
    input_position: f64,
    last_input_tick: i64,
    last_y1: f32,
    last_y2: f32,
}
impl Point {
    pub fn new() -> Self {
        Self {speed_tick: 0, input_position: 0., last_input_tick: -1, last_y1: 0., last_y2: 0.}
    }
}

struct State {
    points: Vec<Point>,
}
impl State {
    pub fn new() -> Self {
        Self {
            points: vec![Point::new()],
        }
    }
}

const INPUT_EAR_INDEX: Index = 0;
const IN_HUM_INDEX: Index = 0;
const SPEED_HUM_INDEX: Index = 1;
const NEUTRAL_HUM_INDEX: Index = 2;

pub struct SpeedModulator {
    chunk_size: usize,
    states: Vec<State>,
}
impl SpeedModulator {
    pub fn new(mut base: TalkerBase) -> Result<CTalker, failure::Error> {
        let stem_set = Set::from_attributs(&vec![
            ("in", PortType::Audio, -1., 1., 0., Init::DefValue),
            ("speed", PortType::Cv, -1., 999., 1., Init::DefValue),
            ("neutral", PortType::Control, 0., 1., 1., Init::DefValue),
        ])?;

        base.add_ear(Ear::new(Some("inputs"), true, Some(stem_set), None));

        Ok(ctalker!(base, Self {
            chunk_size: AudioFormat::chunk_size(),
            states: Vec::new(),
        }))
    }

    pub fn descriptor() -> TalkerHandlerBase {
        TalkerHandlerBase::builtin("Modulator", MODEL, "Speed Modulator")
    }
}

impl Talker for SpeedModulator {
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
        let state = &mut self.states[port];
        let mut starting_point = Point::new();

        // Search for parameters of the requested time
        let mut points_idx = state.points.len();

        while points_idx > 0 {
            points_idx -= 1;
            
            let p = &state.points[points_idx];
            
            if tick >= p.speed_tick {
                starting_point = *p;

                if points_idx < state.points.len() - 1 {
                    state.points.truncate(points_idx + 1);
                }
                break;
            }
        }

        let ear = base.ear(INPUT_EAR_INDEX);
        let speed_buf = ear.get_set_hum_cv_buffer(port, SPEED_HUM_INDEX);

        ear.listen_set_hum(tick, 1, port, NEUTRAL_HUM_INDEX);
        let neutral_buf = ear.get_set_hum_control_buffer(port, NEUTRAL_HUM_INDEX);
        let speed_step_gap = 1. - neutral_buf[0] as f64;
        
        // If the requested time portion is not directly following the previous one, we compute the input corresponding time.
        if starting_point.speed_tick < tick {

            let mut rl = (tick - starting_point.speed_tick) as usize;
            let mut spd_t = starting_point.speed_tick;
            let mut spd_idx = 0;

            while rl > 0 {
                let l = self.chunk_size.min(rl);
                let spd_len = ear.listen_set_hum(spd_t, l, port, SPEED_HUM_INDEX);
                spd_idx = 0;

                while spd_idx < spd_len {
                    starting_point.input_position += speed_buf[spd_idx] as f64 + speed_step_gap;
                    spd_idx += 1;
                }

                spd_t += spd_len as i64;
                rl -= spd_len;
            }
            let prev_p = speed_buf[spd_idx - 1] as f64 + speed_step_gap;
            starting_point.last_input_tick = (starting_point.input_position - prev_p).floor() as i64 + 1;
        }

        // Reading the speed input of the requested time portion
        let speed_len = ear.listen_set_hum(tick, len, port, SPEED_HUM_INDEX);

        // Calculates the duration to read on the input.
        let mut last_in_pos = starting_point.input_position;

        for i in 0..(speed_len - 1) {
            last_in_pos += speed_buf[i] as f64 + speed_step_gap;
        }

        let in_pos_over = last_in_pos + speed_buf[speed_len - 1] as f64 + speed_step_gap;

        let mut input_tick = starting_point.last_input_tick + 1;
        let last_input_tick = last_in_pos.floor() as i64 + 1;
        let input_len = (last_input_tick - starting_point.last_input_tick) as usize;

        let input_buf = ear.get_set_hum_audio_buffer(port, IN_HUM_INDEX);

        let voice_buf = base.voice(port).audio_buffer();

        // copy of speed buffer because it can be modified by the input listening call
        let mut speed_work_buf = vec![0.; speed_buf.len()];
        speed_work_buf.clone_from_slice(speed_buf);

        let mut y1 = starting_point.last_y1;
        let mut y2 = starting_point.last_y2;
    
        if input_len > 1 {
            let mut in_rem_len = input_len;
            let mut in_end = -1.;
            
            let mut it = starting_point.last_input_tick as f64;
            let mut ip = starting_point.input_position;
            let mut t = 0;
            
            while ip <= it {
                voice_buf[t] = y1 + (y2 - y1) * ip.fract() as f32;
                ip += speed_work_buf[t] as f64 + speed_step_gap;
                t += 1;
            }

            while t < speed_len {

                if ip >= in_end {
                    let l = self.chunk_size.min(in_rem_len);
                    let rl = ear.listen_set_hum(input_tick, l, port, IN_HUM_INDEX);
                    it = input_tick as f64;
                    in_end = it + rl as f64 - 1.;
                    input_tick += rl as i64;
                    in_rem_len -= rl;
                    y1 = y2;
                    y2 = input_buf[0];
                }
                
                if ip >= it {
                    let x1 = (ip - it).floor() as usize;
                    y1 = input_buf[x1];
                    y2 = input_buf[x1 + 1];
                }

                let ii_frac = ip.fract();

                if ii_frac == 0. {
                    voice_buf[t] = y1;
                }
                else {
                    voice_buf[t] = y1 + (y2 - y1) * ii_frac as f32;
                }

                ip += speed_work_buf[t] as f64 + speed_step_gap;
                t += 1;
            }
        }
        else {
            ear.listen_set_hum(input_tick, 1, port, IN_HUM_INDEX) as f64;
            y1 = input_buf[0];
            y2 = y1;
            voice_buf.fill(y1);
        }

        // Memorizing parameters of the next time
        starting_point.speed_tick = tick + speed_len as i64;
        starting_point.input_position = in_pos_over;
        starting_point.last_input_tick = last_input_tick;
        starting_point.last_y1 = y1;
        starting_point.last_y2 = y2;
        
        state.points.push(starting_point);

        speed_len
    }
}
