use std::f32;

use tables::sinramp;
use talkers::tseq::parser::PTransition;

pub const DEFAULT_FREQUENCY: f32 = 0.;
pub const DEFAULT_VELOCITY: f32 = 1.;

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

pub struct AudioEventBase {
    pub start_tick: i64,
    pub fadein_tick: i64,
    pub fadeout_tick: i64,
    pub end_tick: i64,
}
impl AudioEventBase {
    pub fn new(start_tick: i64, fadein_tick: i64, fadeout_tick: i64, end_tick: i64) -> Self {
        Self {
            start_tick,
            fadein_tick,
            fadeout_tick,
            end_tick,
        }
    }
}

pub trait AudioEventCore {
    fn assign_buffer(
        &self,
        base: &AudioEventBase,
        t: i64,
        buf: &mut [f32],
        ofset: usize,
        len: usize,
    ) -> i64;
}

pub struct AudioEvent {
    pub base: AudioEventBase,
    pub core: Box<dyn AudioEventCore>,
}

impl AudioEvent {
    pub fn start_tick(&self) -> i64 {
        self.base.start_tick
    }
    pub fn fadein_tick(&self) -> i64 {
        self.base.fadein_tick
    }
    pub fn fadeout_tick(&self) -> i64 {
        self.base.fadeout_tick
    }
    pub fn end_tick(&self) -> i64 {
        self.base.end_tick
    }
    pub fn assign_buffer(&self, tick: i64, buf: &mut [f32], ofset: usize, len: usize) -> i64 {
        let out_len = usize::min((self.base.end_tick - tick) as usize, len);

        let out_end_t = self
            .core
            .assign_buffer(&self.base, tick, buf, ofset, out_len);

        if tick < self.base.fadein_tick {
            self.fadein_buffer(tick, buf, ofset, out_len);
        }

        if out_end_t > self.base.fadeout_tick {
            self.fadeout_buffer(tick, buf, ofset, out_len);
        }

        out_end_t
    }
    pub fn fadein_buffer(&self, tick: i64, buf: &mut [f32], ofset: usize, len: usize) {
        let ln = usize::min(len, (self.base.fadein_tick - tick) as usize);
        let mut fadein_idx = (tick - self.base.start_tick) as usize;

        for i in ofset..ofset + ln {
            buf[i] = buf[i] * sinramp::VELOCITY_FADING_TAB[fadein_idx];
            fadein_idx += 1;
        }
    }

    pub fn fadeout_buffer(&self, tick: i64, buf: &mut [f32], ofset: usize, len: usize) {
        let fadeout_tick = self.base.fadeout_tick;
        let (pos, ln, mut fadeout_idx) = if tick < fadeout_tick {
            let fo_ofset = (fadeout_tick - tick) as usize;
            (
                ofset + fo_ofset,
                len - fo_ofset,
                sinramp::VELOCITY_FADING_LEN,
            )
        } else {
            (
                ofset,
                len,
                sinramp::VELOCITY_FADING_LEN - (tick - fadeout_tick) as usize,
            )
        };

        for i in pos..pos + ln {
            fadeout_idx -= 1;
            buf[i] = buf[i] * sinramp::VELOCITY_FADING_TAB[fadeout_idx];
        }
    }
}

// ConstantEvent
pub struct ConstantEvent {
    value: f32,
}
impl ConstantEvent {
    pub fn new(value: f32) -> Self {
        Self { value }
    }
}
impl AudioEventCore for ConstantEvent {
    fn assign_buffer(
        &self,
        _: &AudioEventBase,
        tick: i64,
        buf: &mut [f32],
        ofset: usize,
        len: usize,
    ) -> i64 {
        for i in ofset..ofset + len {
            buf[i] = self.value;
        }
        tick + len as i64
    }
}

// LinearEvent
pub struct LinearEvent {
    a: f64,
    b: f64,
}
impl LinearEvent {
    pub fn new(start_tick: i64, end_tick: i64, start_value: f32, end_value: f32) -> Self {
        let a = (end_value - start_value) as f64 / ((end_tick - start_tick) as f64);
        let b = (start_value as f64) - a * (start_tick as f64);

        Self { a, b }
    }
}
impl AudioEventCore for LinearEvent {
    fn assign_buffer(
        &self,
        _: &AudioEventBase,
        tick: i64,
        buf: &mut [f32],
        ofset: usize,
        len: usize,
    ) -> i64 {
        let mut x = tick as f64;

        for i in ofset..ofset + len {
            buf[i] = (self.a * x + self.b) as f32;
            x += 1.;
        }
        tick + len as i64
    }
}

// SinRampEvent
pub struct SinRampEvent {
    start_value: f32,
    len_on_dt: f64,
    dv: f32,
}
impl SinRampEvent {
    pub fn new(start_tick: i64, end_tick: i64, start_value: f32, end_value: f32) -> Self {
        Self {
            start_value,
            len_on_dt: sinramp::LEN as f64 / (end_tick - start_tick) as f64,
            dv: end_value - start_value,
        }
    }
}
impl AudioEventCore for SinRampEvent {
    fn assign_buffer(
        &self,
        base: &AudioEventBase,
        tick: i64,
        buf: &mut [f32],
        ofset: usize,
        len: usize,
    ) -> i64 {
        let start_tick = base.start_tick as f64;
        let mut t = tick as f64;

        for i in ofset..ofset + len {
            let tab_idx = ((t - start_tick) * self.len_on_dt) as usize;
            buf[i] = sinramp::TAB[tab_idx] * self.dv + self.start_value;
            t += 1.;
        }
        tick + len as i64
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
) -> AudioEvent {
    let base = AudioEventBase::new(
        start_tick,
        fadein_tick(start_tick, end_tick, fadein),
        fadeout_tick(start_tick, end_tick, fadeout),
        end_tick,
    );

    let core: Box<dyn AudioEventCore> = match transition {
        PTransition::None => Box::new(ConstantEvent::new(start_value)),
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
    };

    AudioEvent { base, core }
}

pub fn create_from_parameter(parameter: &AudioEventParameter) -> AudioEvent {
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
