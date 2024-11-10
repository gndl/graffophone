use std::collections::VecDeque;
use std::f32;

use tables::fadein;
use tables::fadeout;
use tables::sinramp;
use tables::roundramp;
use tables::earlyramp;
use tables::lateramp;
use talkers::tseq::envelope;
use talkers::tseq::parser::PShape;
use talkers::tseq::sequence::SequenceEvents;

fn fadein_tick(start_tick: i64, end_tick: i64, fadein: bool) -> i64 {
    if fadein {
        i64::min(start_tick + fadein::LEN as i64, end_tick)
    } else {
        start_tick
    }
}

fn fadeout_tick(start_tick: i64, end_tick: i64, fadeout: bool) -> i64 {
    if fadeout {
        i64::max(start_tick, end_tick - fadeout::LEN as i64)
    } else {
        end_tick
    }
}

pub struct AudioEventBase {
    pub start_tick: i64,
    pub fadein_tick: i64,
    pub fadeout_tick: i64,
    pub end_tick: i64,
    pub envelope_index: usize,
}
impl AudioEventBase {
    pub fn new(
        start_tick: i64,
        fadein_tick: i64,
        fadeout_tick: i64,
        end_tick: i64,
        envelope_index: usize,
    ) -> Self {
        Self {
            start_tick,
            fadein_tick,
            fadeout_tick,
            end_tick,
            envelope_index,
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

    pub fn assign_buffer(
        &self,
        envelops: &Vec<Vec<f32>>,
        tick: i64,
        buf: &mut [f32],
        ofset: usize,
        len: usize,
    ) -> i64 {
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

        if self.base.envelope_index < envelops.len() {
            let envelope = envelops[self.base.envelope_index].as_slice();
            let mut envelope_idx = (tick - self.base.start_tick) as usize;

            for i in ofset..ofset + out_len {
                buf[i] = buf[i] * envelope[envelope_idx];
                envelope_idx += 1;
            }
        }
        out_end_t
    }
    pub fn fadein_buffer(&self, tick: i64, buf: &mut [f32], ofset: usize, len: usize) {
        let ln = usize::min(len, (self.base.fadein_tick - tick) as usize);
        let mut fadein_idx = (tick - self.base.start_tick) as usize;

        for i in ofset..ofset + ln {
            buf[i] = buf[i] * fadein::TAB[fadein_idx];
            fadein_idx += 1;
        }
    }

    pub fn fadeout_buffer(&self, tick: i64, buf: &mut [f32], ofset: usize, len: usize) {
        let fadeout_tick = self.base.fadeout_tick;

        let (pos, ln, mut fadeout_idx) = if tick < fadeout_tick {
            let fo_ofset = (fadeout_tick - tick) as usize;
            (ofset + fo_ofset, len - fo_ofset, 0)
        } else {
            (ofset, len, (tick - fadeout_tick) as usize)
        };

        for i in pos..pos + ln {
            buf[i] = buf[i] * fadeout::TAB[fadeout_idx];
            fadeout_idx += 1;
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

// AliasedCurveEvent
pub struct AliasedCurveEvent {
    table:&'static[f32],start_tick:usize,duration:usize,
    start_value: f32,
    dv: f32,
}
impl AliasedCurveEvent {
    pub fn new(table:&'static[f32],start_tick: i64, end_tick: i64, start_value: f32, end_value: f32) -> Self {
        Self {table,start_tick:start_tick as usize,duration:(end_tick - start_tick)as usize,
            start_value,
            dv: end_value - start_value,
        }
    }
}
impl AudioEventCore for AliasedCurveEvent {
    fn assign_buffer(
        &self,
        _base: &AudioEventBase,
        tick: i64,
        buf: &mut [f32],
        ofset: usize,
        len: usize,
    ) -> i64 {
        let start_tick = self.start_tick;
        let duration = self.duration;
        let tab_len = self.table.len();
        let mut t = tick as usize;

        for i in ofset..ofset + len {
            let tab_idx = ((t - start_tick) * tab_len ) / duration;
            buf[i] = self.table[tab_idx] * self.dv + self.start_value;
            t += 1;
        }
        t as i64
    }
}

// InterpolatedCurveEvent
pub struct InterpolatedCurveEvent {
    table:&'static[f32],
    start_value: f32,
    len_on_dt: f32,
    dv: f32,
}
impl InterpolatedCurveEvent {
    pub fn new(table:&'static[f32],start_tick: i64, end_tick: i64, start_value: f32, end_value: f32) -> Self {
        Self {table,
            start_value,
            len_on_dt: (table.len() - 1) as f32 / (end_tick - start_tick) as f32,
            dv: end_value - start_value,
        }
    }
}
impl AudioEventCore for InterpolatedCurveEvent {
    fn assign_buffer(
        &self,
        base: &AudioEventBase,
        tick: i64,
        buf: &mut [f32],
        ofset: usize,
        len: usize,
    ) -> i64 {
        let len_on_dt = self.len_on_dt;
        let mut pos = (tick - base.start_tick) as f32;

        for i in ofset..ofset + len {
            let tab_pos = pos * len_on_dt;
            let tab_idx = tab_pos as usize;
            let prev_c = self.table[tab_idx];
            let next_c = self.table[tab_idx + 1];
            let c = prev_c + (next_c - prev_c) * tab_pos.fract();
            buf[i] = c * self.dv + self.start_value;
            pos += 1.;
        }
        tick + len as i64
    }
}

pub fn create(
    start_tick: i64,
    end_tick: i64,
    start_value: f32,
    end_value: f32,
    transition: PShape,
    fadein: bool,
    fadeout: bool,
    envelop_index: usize,
) -> AudioEvent {
    let base = AudioEventBase::new(
        start_tick,
        fadein_tick(start_tick, end_tick, fadein),
        fadeout_tick(start_tick, end_tick, fadeout),
        end_tick,
        envelop_index,
    );

    let core: Box<dyn AudioEventCore> = match transition {
        PShape::None => Box::new(ConstantEvent::new(start_value)),
        PShape::Linear => Box::new(LinearEvent::new(
            start_tick,
            end_tick,
            start_value,
            end_value,
        )),
        PShape::Sin => Box::new(InterpolatedCurveEvent::new(&sinramp::TAB,
            start_tick,
            end_tick,
            start_value,
            end_value,
        )),
        PShape::Early => Box::new(InterpolatedCurveEvent::new(&earlyramp::TAB,
            start_tick,
            end_tick,
            start_value,
            end_value,
        )),
        PShape::Late => Box::new(InterpolatedCurveEvent::new(&lateramp::TAB,
            start_tick,
            end_tick,
            start_value,
            end_value,
        )),
        PShape::Round => Box::new(InterpolatedCurveEvent::new(&roundramp::TAB,
            start_tick,
            end_tick,
            start_value,
            end_value,
        )),
    };

    AudioEvent { base, core }
}

pub type AudioEvents = Vec<AudioEvent>;

pub fn create_from_sequences(harmonics_sequence_events: &VecDeque<SequenceEvents>, envelops: &Vec<Vec<f32>>) -> (VecDeque<AudioEvents>, VecDeque<AudioEvents>) {
    let mut harmonics_frequency_events = VecDeque::with_capacity(harmonics_sequence_events.len());
    let mut harmonics_velocity_events = VecDeque::with_capacity(harmonics_sequence_events.len());

    for harmonic_sequence_events in harmonics_sequence_events{
        let mut frequency_events = Vec::with_capacity(harmonic_sequence_events.len());
        let mut velocity_events = Vec::with_capacity(harmonic_sequence_events.len());

        for event in harmonic_sequence_events {
            frequency_events.push(create(
                event.start_tick,
                event.end_tick,
                event.start_frequency,
                event.end_frequency,
                event.frequency_transition,
                false,
                false,
                envelope::UNDEFINED,
            ));

            let mut velocity_end_tick = event.end_tick;
            let mut velocity_fadeout = event.fadeout;
            
            if event.envelop_index < envelops.len() {
                let env_len = envelops[event.envelop_index].len() as i64;
                let env_end_tick = event.start_tick + env_len;
                
                if env_end_tick < event.end_tick {
                    velocity_end_tick = env_end_tick;
                    velocity_fadeout = true;
                }
            }

            velocity_events.push(create(
                event.start_tick,
                velocity_end_tick,
                event.start_velocity,
                event.end_velocity,
                event.velocity_transition,
                event.fadein,
                velocity_fadeout,
                event.envelop_index,
            ));
        }
        harmonics_frequency_events.push_back(frequency_events);
        harmonics_velocity_events.push_back(velocity_events);
    }

    (harmonics_frequency_events, harmonics_velocity_events)
}
