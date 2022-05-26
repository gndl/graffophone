use std::f32;

use tables::sinramp;
use talkers::tseq::parser::PTransition;

pub trait AudioEvent {
    fn start_tick(&self) -> i64;
    fn end_tick(&self) -> i64;
    fn assign_buffer(&self, t: i64, buf: &mut [f32], ofset: usize, len: usize) -> i64;
}
pub type RAudioEvent = Box<dyn AudioEvent>;

// ConstantEvent
pub struct ConstantEvent {
    start_tick: i64,
    end_tick: i64,
    value: f32,
}
impl ConstantEvent {
    pub fn new(start_tick: i64, end_tick: i64, value: f32) -> Self {
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
    fn assign_buffer(&self, tick: i64, buf: &mut [f32], ofset: usize, len: usize) -> i64 {
        let cur_len = usize::min((self.end_tick - tick) as usize, len);

        for i in ofset..ofset + cur_len {
            buf[i] = self.value;
        }
        tick + cur_len as i64
    }
}

// LinearEvent
pub struct LinearEvent {
    start_tick: i64,
    end_tick: i64,
    a: f64,
    b: f64,
}
impl LinearEvent {
    pub fn new(start_tick: i64, end_tick: i64, start_value: f32, end_value: f32) -> Self {
        let a = (end_value - start_value) as f64 / ((end_tick - start_tick) as f64);
        let b = (start_value as f64) - a * (start_tick as f64);

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
    fn assign_buffer(&self, tick: i64, buf: &mut [f32], ofset: usize, len: usize) -> i64 {
        let mut x = tick as f64;
        let cur_len = usize::min((self.end_tick - tick) as usize, len);

        for i in ofset..ofset + cur_len {
            buf[i] = (self.a * x + self.b) as f32;
            x += 1.;
        }
        tick + cur_len as i64
    }
}

// SinRampEvent
pub struct SinRampEvent {
    start_tick: i64,
    end_tick: i64,
    start_value: f32,
    len_on_dt: f64,
    dv: f32,
}
impl SinRampEvent {
    pub fn new(start_tick: i64, end_tick: i64, start_value: f32, end_value: f32) -> Self {
        Self {
            start_tick,
            end_tick,
            start_value,
            len_on_dt: sinramp::LEN as f64 / (end_tick - start_tick) as f64,
            dv: end_value - start_value,
        }
    }
}
impl AudioEvent for SinRampEvent {
    fn start_tick(&self) -> i64 {
        self.start_tick
    }
    fn end_tick(&self) -> i64 {
        self.end_tick
    }
    fn assign_buffer(&self, tick: i64, buf: &mut [f32], ofset: usize, len: usize) -> i64 {
        let start_tick = self.start_tick as f64;
        let mut t = tick as f64;
        let cur_len = usize::min((self.end_tick - tick) as usize, len);

        for i in ofset..ofset + cur_len {
            let tab_idx = ((t - start_tick) * self.len_on_dt) as usize;
            buf[i] = sinramp::TAB[tab_idx] * self.dv + self.start_value;
            t += 1.;
        }
        tick + cur_len as i64
    }
}

pub fn create(
    start_tick: i64,
    end_tick: i64,
    start_value: f32,
    end_value: f32,
    transition: PTransition,
) -> RAudioEvent {
    match transition {
        PTransition::None => Box::new(ConstantEvent::new(start_tick, end_tick, start_value)),
        PTransition::Linear => Box::new(LinearEvent::new(
            start_tick,
            end_tick,
            start_value,
            end_value,
        )),
        PTransition::Sin => Box::new(SinRampEvent::new(
            start_tick,
            end_tick,
            start_value,
            end_value,
        )),
        PTransition::Early => Box::new(SinRampEvent::new(
            start_tick,
            end_tick,
            start_value,
            end_value,
        )),
        PTransition::Late => Box::new(SinRampEvent::new(
            start_tick,
            end_tick,
            start_value,
            end_value,
        )),
        PTransition::Round => Box::new(SinRampEvent::new(
            start_tick,
            end_tick,
            start_value,
            end_value,
        )),
    }
}
