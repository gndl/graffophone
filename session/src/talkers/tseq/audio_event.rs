use std::f32;

//use talker::audio_format::AudioFormat;
use talkers::tseq::parser::PProgression;

pub trait AudioEvent {
    fn start_tick(&self) -> i64;
    fn end_tick(&self) -> i64;
    fn assign_buffer(&self, t: i64, buf: &mut [f32], ofset: usize, len: usize) -> i64;
}
pub type RAudioEvent = Box<dyn AudioEvent>;

pub struct ConstantEvent {
    start_tick: i64,
    end_tick: i64,
    value: f32,
}
impl ConstantEvent {
    pub fn new(start_tick: i64, end_tick: i64, value: f32) -> Self {
        println!("tseq::assign_constant : {}", value);
        Self {
            start_tick,
            end_tick,
            value,
        }
    }
}
impl AudioEvent for ConstantEvent {
    fn start_tick(&self) -> i64 {
        self.start_tick
    }
    fn end_tick(&self) -> i64 {
        self.end_tick
    }
    fn assign_buffer(&self, t: i64, buf: &mut [f32], ofset: usize, len: usize) -> i64 {
        let cur_len = usize::min((self.end_tick - t) as usize, len);

        for i in ofset..ofset + cur_len {
            buf[i] = self.value;
        }
        t + cur_len as i64
    }
}

pub struct LinearEvent {
    start_tick: i64,
    end_tick: i64,
    a: f32,
    b: f32,
}
impl LinearEvent {
    pub fn new(start_tick: i64, end_tick: i64, start_value: f32, end_value: f32) -> Self {
        let a = (end_value - start_value) / ((end_tick - start_tick) as f32);
        let b = start_value - a * start_tick as f32;
        println!("LinearEvent : y = {} * x + {}", a, b);

        Self {
            start_tick,
            end_tick,
            a,
            b,
        }
    }
}
impl AudioEvent for LinearEvent {
    fn start_tick(&self) -> i64 {
        self.start_tick
    }
    fn end_tick(&self) -> i64 {
        self.end_tick
    }
    fn assign_buffer(&self, t: i64, buf: &mut [f32], ofset: usize, len: usize) -> i64 {
        let mut x = t as f32;
        let cur_len = usize::min((self.end_tick - t) as usize, len);

        for i in ofset..ofset + cur_len {
            buf[i] = self.a * x + self.b;
            x += 1.;
        }
        t + cur_len as i64
    }
}

pub fn create(
    progression: PProgression,
    start_tick: i64,
    end_tick: i64,
    start_value: f32,
    end_value: f32,
) -> RAudioEvent {
    match progression {
        PProgression::None => Box::new(ConstantEvent::new(start_tick, end_tick, start_value)),
        PProgression::Linear => Box::new(LinearEvent::new(
            start_tick,
            end_tick,
            start_value,
            end_value,
        )),
        PProgression::Cosin => Box::new(LinearEvent::new(
            start_tick,
            end_tick,
            start_value,
            end_value,
        )),
        PProgression::Early => Box::new(LinearEvent::new(
            start_tick,
            end_tick,
            start_value,
            end_value,
        )),
        PProgression::Late => Box::new(LinearEvent::new(
            start_tick,
            end_tick,
            start_value,
            end_value,
        )),
    }
}
