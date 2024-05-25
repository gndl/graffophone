use std::collections::VecDeque;
use std::f32;

use talkers::tseq::binder::{self, Binder, Time, Velocity};
use talkers::tseq::envelope;
use talkers::tseq::parser::PFragment::Fragments;
use talkers::tseq::parser::PFragment::Part;
use talkers::tseq::parser::PFragment::SeqRef;
use talkers::tseq::parser::PPart;
use talkers::tseq::parser::PSequence;
use talkers::tseq::parser::PShape;
use talkers::tseq::parser::{PFragment, PPitchGap};

#[derive(Debug)]
pub struct SequenceEvent {
    pub start_tick: i64,
    pub end_tick: i64,
    pub start_frequency: f32,
    pub end_frequency: f32,
    pub frequency_transition: PShape,
    pub start_velocity: f32,
    pub end_velocity: f32,
    pub velocity_transition: PShape,
    pub fadein: bool,
    pub fadeout: bool,
    pub envelop_index: usize,
}

pub type SequenceEvents = Vec<SequenceEvent>;

struct Event {
    delay: Time,
    frequency: f32,
    frequency_transition: PShape,
    velocity: Velocity,
}
impl Event {
    pub fn new() -> Event {
        Self {
            delay: Time::Ticks(0),
            frequency: binder::DEFAULT_FREQUENCY,
            frequency_transition: PShape::None,
            velocity: Velocity::new(),
        }
    }
}
struct EventsBuilder {
    tick: i64,
    hit_start_tick: i64,
    hit_end_tick: i64,
    harmonic_count: usize,
    chord_events: Vec<Event>,
}

impl EventsBuilder {
    pub fn new() -> EventsBuilder {
        Self {
            tick: 0,
            hit_start_tick: 0,
            hit_end_tick: 0,
            harmonic_count: 0,
            chord_events: Vec::new(),
        }
    }

    pub fn create_part_events(
        &mut self,
        binder: &Binder,
        ticks_per_beat: f32,
        seq_envelop_index: usize,
        part: &PPart,
        harmonics_events: &mut VecDeque<Vec<SequenceEvent>>,
    ) -> Result<(), failure::Error> {
        let mut part_is_empty = true;
        let hitline = binder.fetch_hitline(part.hitline_id)?;
        let hitline_hits_count = hitline.hits.len();
        let hitline_ticks_count = binder::to_ticks(&hitline.duration, ticks_per_beat);
        let mut hitline_start_tick = self.tick;
        let mut mul = part.mul.unwrap_or(1.);
        let fadeout_pre_envelop = seq_envelop_index != envelope::UNDEFINED;

        if hitline_hits_count > 0 && mul > 0. {
            if let Some(pitchline_id) = part.pitchline_id {
                let (scale, pitchline) = binder.fetch_pitchline(pitchline_id)?;
                let pitchs_count = pitchline.len();

                if pitchs_count > 0 {
                    part_is_empty = false;

                    let chordline = binder.fetch_chordline(&part.chordline_id)?;
                    let chords_count = chordline.len();

                    let velocities = binder.fetch_velocityline(&part.velocityline_id)?;
                    let velocities_count = velocities.len();

                    let mut next_hit_idx = 0;
                    let mut next_pitch_idx = 0;
                    let mut next_chord_idx = 0;
                    let mut next_velocity_idx = 0;

                    let max_n = usize::max(hitline_hits_count, pitchs_count);

                    while mul > 0. {
                        let n = if mul < 1. {
                            ((max_n as f32) * mul) as usize
                        } else {
                            max_n
                        };

                        for _ in 0..n {
                            let next_hit = &hitline.hits[next_hit_idx];
                            let next_hit_start_tick = hitline_start_tick
                                + binder::to_ticks(&next_hit.position, ticks_per_beat);
                            let next_hit_end_tick = binder::option_to_ticks(
                                &next_hit.duration,
                                next_hit_start_tick,
                                ticks_per_beat,
                            );

                            let hit_ticks_count = (i64::min(self.hit_end_tick, next_hit_start_tick)
                                - self.hit_start_tick)
                                as f32;

                            let (next_pitch_frequency, next_pitch_transition) =
                                pitchline[next_pitch_idx];

                            let next_chord = &chordline[next_chord_idx];

                            if next_chord.len() > harmonics_events.len() {
                                for _ in harmonics_events.len()..next_chord.len() {
                                    harmonics_events.push_back(Vec::new());
                                    self.chord_events.push(Event::new());
                                }
                            }

                            let next_velocity = &velocities[next_velocity_idx];

                            let max_harmonic_count =
                                usize::max(self.harmonic_count, next_chord.len());

                            for harmonic_idx in 0..max_harmonic_count {
                                let harmonic_event = &mut self.chord_events[harmonic_idx];

                                let next_harmonic_idx = usize::min(harmonic_idx, next_chord.len() - 1);
                                let next_harmonic = &next_chord[next_harmonic_idx];

                                let freq_ratio = match &next_harmonic.pitch_gap {
                                    PPitchGap::FreqRatio(r) => *r,
                                    PPitchGap::Interval(i) => scale.frequency_ratio(*i),
                                };

                                let next_harmonic_frequency = next_pitch_frequency * freq_ratio;
                                let next_harmonic_velocity = next_harmonic.velocity.level * next_velocity.level;

                                if harmonic_idx < self.harmonic_count {
                                    let start_tick = self.hit_start_tick
                                        + binder::to_ticks(&harmonic_event.delay, hit_ticks_count);

                                    harmonics_events[harmonic_idx].push(
                                         SequenceEvent {
                                             start_tick,
                                             end_tick: self.hit_end_tick,
                                             start_frequency: harmonic_event.frequency,
                                             end_frequency: next_harmonic_frequency,
                                             frequency_transition: harmonic_event.frequency_transition,
                                             start_velocity: harmonic_event.velocity.level,
                                             end_velocity: next_harmonic_velocity,
                                             velocity_transition: harmonic_event.velocity.transition,
                                             fadein: harmonic_event.velocity.fadein,
                                             fadeout: harmonic_event.velocity.fadeout || fadeout_pre_envelop,
                                             envelop_index: harmonic_event.velocity.envelope_index,
                                         }
                                    );
                                }

                                if harmonic_idx < next_chord.len() {
                                    harmonic_event.delay = next_harmonic.delay;
                                    harmonic_event.frequency = next_harmonic_frequency;
                                    harmonic_event.frequency_transition = next_pitch_transition;
                                    
                                    let envelop_index= if next_harmonic.velocity.envelope_index != envelope::UNDEFINED {
                                        // The envelope defined at the chord level has priority over the envelope defined at the velocityline level
                                        next_harmonic.velocity.envelope_index
                                    } else if next_velocity.envelope_index != envelope::UNDEFINED {
                                        // The envelope defined at the velocityline level has priority over the envelope defined at the sequence level
                                        next_velocity.envelope_index
                                    } else {
                                        seq_envelop_index
                                    };
                                    harmonic_event.velocity = Velocity {
                                        envelope_index: envelop_index,
                                        level: next_harmonic_velocity,
                                        transition: if next_harmonic.velocity.transition
                                            == PShape::None
                                        {
                                            next_velocity.transition
                                        } else {
                                            next_harmonic.velocity.transition
                                        },
                                        fadein: next_harmonic.velocity.fadein
                                            || next_velocity.fadein,
                                        fadeout: next_harmonic.velocity.fadeout
                                            || next_velocity.fadeout,
                                    };
                                }
                            }
                            self.hit_start_tick = next_hit_start_tick;
                            self.hit_end_tick = next_hit_end_tick;
                            self.harmonic_count = next_chord.len();

                            next_hit_idx = if next_hit_idx < hitline_hits_count - 1 {
                                next_hit_idx + 1
                            } else {
                                hitline_start_tick += hitline_ticks_count;
                                0
                            };

                            next_pitch_idx = if next_pitch_idx < pitchs_count - 1 {
                                next_pitch_idx + 1
                            } else {
                                0
                            };

                            next_chord_idx = if next_chord_idx < chords_count - 1 {
                                next_chord_idx + 1
                            } else {
                                0
                            };

                            next_velocity_idx = if next_velocity_idx < velocities_count - 1 {
                                next_velocity_idx + 1
                            } else {
                                0
                            };
                        }
                        mul -= 1.;
                    }
                    if pitchs_count > hitline_hits_count && pitchs_count % hitline_hits_count != 0 {
                        hitline_start_tick += hitline_ticks_count;
                    }
                    self.tick = hitline_start_tick;
                }
            }
        }
        if part_is_empty {
            self.tick += (hitline_ticks_count as f32 * mul.ceil()) as i64;
        }
        Ok(())
    }

    pub fn create_fragment_events(
        &mut self,
        binder: &Binder,
        ticks_per_beat: f32,
        envelop_index: usize,
        fragment: &PFragment,
        harmonics_events: &mut VecDeque<Vec<SequenceEvent>>,
    ) -> Result<(), failure::Error> {
        match fragment {
            Part(part) => {
                self.create_part_events(
                    binder,
                    ticks_per_beat,
                    envelop_index,
                    part,
                    harmonics_events,
                )?;
            }
            SeqRef(seqref) => {
                let seq = binder.fetch_sequence(&seqref.id)?;
                let mul = seqref.mul.unwrap_or(1);

                for _ in 0..mul {
                    self.create_events(
                        binder,
                        ticks_per_beat,
                        envelop_index,
                        seq,
                        harmonics_events,
                    )?;
                }
            }
            Fragments((fragments, mul)) => {
                for _ in 0..*mul {
                    for fragment in fragments {
                        self.create_fragment_events(
                            binder,
                            ticks_per_beat,
                            envelop_index,
                            fragment,
                            harmonics_events,
                        )?;
                    }
                }
            }
        }
        Ok(())
    }

    pub fn create_events(
        &mut self,
        binder: &Binder,
        ticks_per_beat: f32,
        envelop_index: usize,
        sequence: &PSequence,
        harmonics_events: &mut VecDeque<Vec<SequenceEvent>>,
    ) -> Result<(), failure::Error> {
        let ticks_per_beat = match sequence.beat {
            Some(id) => binder.ticks_per_minute / (binder.fetch_beat(id)?),
            None => ticks_per_beat,
        };

        let envelop_index = match sequence.envelope_id {
            Some(id) => binder.fetch_envelop_index(id)?,
            None => envelop_index,
        };

        for fragment in &sequence.fragments {
            self.create_fragment_events(
                binder,
                ticks_per_beat,
                envelop_index,
                fragment,
                harmonics_events,
            )?;
        }

        Ok(())
    }

    pub fn create_last_events(
        &self,
        harmonics_events: &mut VecDeque<Vec<SequenceEvent>>,
    ) {
        let end_tick = if self.hit_end_tick == binder::UNDEFINED_TICKS {
            self.tick
        } else {
            self.hit_end_tick
        };
        let hit_ticks_count = (end_tick - self.hit_start_tick) as f32;

        for harmonic_idx in 0..self.harmonic_count {
            let harmonic_event = &self.chord_events[harmonic_idx];

            let start_tick =
                self.hit_start_tick + binder::to_ticks(&harmonic_event.delay, hit_ticks_count);

            if harmonic_event.frequency > 0. {
                harmonics_events[harmonic_idx].push(
                    SequenceEvent {
                        start_tick,
                        end_tick,
                        start_frequency: harmonic_event.frequency,
                        end_frequency: harmonic_event.frequency,
                        frequency_transition: PShape::None,
                        start_velocity: harmonic_event.velocity.level,
                        end_velocity: harmonic_event.velocity.level,
                        velocity_transition: PShape::None,
                        fadein: harmonic_event.velocity.fadein,
                        fadeout: true,
                        envelop_index: harmonic_event.velocity.envelope_index,
                    }
                );
            }
        }
    }
}

fn limit_overflowing_durations(
    harmonics_events: &mut VecDeque<Vec<SequenceEvent>>,
) {
    for harmonic_events in harmonics_events {
        for idx in 0..harmonic_events.len() - 1 {
            if harmonic_events[idx].end_tick > harmonic_events[idx + 1].start_tick {
                harmonic_events[idx].end_tick = harmonic_events[idx + 1].start_tick;
            }
        }
    }
}

pub fn create_events(
    binder: &Binder,
    sequence: &PSequence,
) -> Result<VecDeque<SequenceEvents>, failure::Error> {
    let mut builder = EventsBuilder::new();
    let mut harmonics_events = VecDeque::with_capacity(6);

    let ticks_per_beat = binder.ticks_per_minute / binder.default_bpm as f32;

    builder.create_events(
        binder,
        ticks_per_beat,
        envelope::UNDEFINED,
        sequence,
        &mut harmonics_events,
    )?;

    builder.create_last_events(&mut harmonics_events);

    limit_overflowing_durations(&mut harmonics_events);

    Ok(harmonics_events)
}

pub struct EventReminder {
    pub initialized: bool,
    pub index: usize,
    pub last_value: f32,
}

impl EventReminder {
    pub fn new() -> EventReminder {
        Self {
            initialized: false,
            index: 0,
            last_value: 0.,
        }
    }
}
