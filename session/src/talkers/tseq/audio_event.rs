use std::f32;

use tables::sinramp;
use talkers::tseq::parser::PTransition;

pub const DEFAULT_FREQUENCY: f32 = 0.;
pub const DEFAULT_VELOCITY: f32 = 1.;

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

fn fadein_tick(start_tick: i64, end_tick: i64, fadein: bool) -> i64 {
    if fadein {
        i64::min(start_tick + sinramp::VELOCITY_FADING_LEN as i64, end_tick)
    } else {
        start_tick
    }
}

fn fadeout_tick(start_tick: i64, end_tick: i64, fadeout: bool) -> i64 {
    if fadeout {
        i64::max(start_tick, end_tick - sinramp::VELOCITY_FADING_LEN as i64)
    } else {
        end_tick
    }
}
// ConstantEvent
pub struct FadingEvent {
    start_tick: i64,
    fadein_tick: i64,
    fadeout_tick: i64,
    end_tick: i64,
    value: f32,
}
impl FadingEvent {
    pub fn new(
        start_tick: i64,
        fadein_tick: i64,
        fadeout_tick: i64,
        end_tick: i64,
        value: f32,
    ) -> Self {
        Self {
            start_tick,
            fadein_tick,
            fadeout_tick,
            end_tick,
            value,
        }
    }
}
impl AudioEvent for FadingEvent {
    fn start_tick(&self) -> i64 {
        self.start_tick
    }
    fn end_tick(&self) -> i64 {
        self.end_tick
    }
    fn assign_buffer(&self, tick: i64, buf: &mut [f32], ofset: usize, len: usize) -> i64 {
        let mut t = tick;
        let end_tick = i64::min(tick + len as i64, self.end_tick);

        if tick < self.fadein_tick {
            let end_t = i64::min(self.fadein_tick, end_tick);
            let ln = (end_t - t) as usize;
            let mut fadein_idx = (t - self.start_tick) as usize;

            for i in ofset..ofset + ln {
                buf[i] = self.value * sinramp::VELOCITY_FADING_TAB[fadein_idx];
                fadein_idx += 1;
            }
            t += ln as i64;
        }

        if t < self.fadeout_tick && t < end_tick {
            let end_t = i64::min(self.fadeout_tick, end_tick);
            let ln = (end_t - t) as usize;
            let pos = ofset + (t - tick) as usize;

            for i in pos..pos + ln {
                buf[i] = self.value;
            }
            t += ln as i64;
        }

        if t < end_tick {
            let ln = (end_tick - t) as usize;
            let pos = ofset + (t - tick) as usize;
            let mut fadeout_idx = sinramp::VELOCITY_FADING_LEN - (t - self.fadeout_tick) as usize;

            for i in pos..pos + ln {
                fadeout_idx -= 1;
                buf[i] = self.value * sinramp::VELOCITY_FADING_TAB[fadeout_idx];
            }
        }
        end_tick
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

#[derive(Debug)]
pub struct AudioEventParameter {
    pub start_tick: i64,
    pub end_tick: i64,
    pub start_value: f32,
    end_value: f32,
    transition: PTransition,
    fadein: bool,
    fadeout: bool,
}

impl AudioEventParameter {
    pub fn new(
        start_tick: i64,
        end_tick: i64,
        start_value: f32,
        end_value: f32,
        transition: PTransition,
        fadein: bool,
        fadeout: bool,
    ) -> AudioEventParameter {
        Self {
            start_tick,
            end_tick,
            start_value,
            end_value,
            transition,
            fadein,
            fadeout,
        }
    }
}

pub fn create(
    start_tick: i64,
    end_tick: i64,
    start_value: f32,
    end_value: f32,
    transition: PTransition,
    fadein: bool,
    fadeout: bool,
) -> RAudioEvent {
    match transition {
        PTransition::None => {
            if fadein || fadeout {
                Box::new(FadingEvent::new(
                    start_tick,
                    fadein_tick(start_tick, end_tick, fadein),
                    fadeout_tick(start_tick, end_tick, fadeout),
                    end_tick,
                    start_value,
                ))
            } else {
                Box::new(ConstantEvent::new(start_tick, end_tick, start_value))
            }
        }
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

pub fn create_from_parameter(parameter: &AudioEventParameter) -> RAudioEvent {
    create(
        parameter.start_tick,
        parameter.end_tick,
        parameter.start_value,
        parameter.end_value,
        parameter.transition,
        parameter.fadein,
        parameter.fadeout,
    )
}
