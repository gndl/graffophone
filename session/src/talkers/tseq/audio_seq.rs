use std::collections::VecDeque;
use std::f32;

use talkers::tseq::audio_event;
use talkers::tseq::audio_event::AudioEventParameter;
use talkers::tseq::audio_event::RAudioEvent;
use talkers::tseq::binder;
use talkers::tseq::binder::Binder;
use talkers::tseq::binder::Time;
use talkers::tseq::parser::PFragment::Fragments;
use talkers::tseq::parser::PFragment::Part;
use talkers::tseq::parser::PFragment::SeqRef;
use talkers::tseq::parser::PPart;
use talkers::tseq::parser::PSequence;
use talkers::tseq::parser::PTransition;
use talkers::tseq::parser::{PFragment, PVelocity};

pub type AudioEvents = Vec<RAudioEvent>;

struct Event {
    delay: Time,
    frequency: f32,
    frequency_transition: PTransition,
    velocity: PVelocity,
}
impl Event {
    pub fn new() -> Event {
        Self {
            delay: Time::Ticks(0),
            frequency: audio_event::DEFAULT_FREQUENCY,
            frequency_transition: PTransition::None,
            velocity: PVelocity {
                value: audio_event::DEFAULT_VELOCITY,
                fadein: false,
                fadeout: false,
                transition: PTransition::None,
            },
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
        part: &PPart,
        harmonics_frequency_events: &mut VecDeque<Vec<AudioEventParameter>>,
        harmonics_velocity_events: &mut VecDeque<Vec<AudioEventParameter>>,
    ) -> Result<(), failure::Error> {
        let mut part_is_empty = true;
        let hitline = binder.fetch_hitline(part.hitline_id)?;
        let hitline_hits_count = hitline.hits.len();
        let hitline_ticks_count = binder::to_ticks(&hitline.duration, ticks_per_beat);
        let mut hitline_start_tick = self.tick;
        let mut mul = part.mul.unwrap_or(1.);

        if hitline_hits_count > 0 && mul > 0. {
            if let Some(pitchline_id) = part.pitchline_id {
                let pitchline = binder.fetch_pitchline(pitchline_id)?;
                let pitchs_count = pitchline.len();

                if pitchs_count > 0 {
                    part_is_empty = false;

                    let chordline = binder.fetch_chordline(&part.chordline_id)?;
                    let chords_count = chordline.len();

                    let velocityline = binder.fetch_velocityline(&part.velocityline_id)?;
                    let velocities_count = velocityline.velocities.len();

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

                            if next_chord.len() > harmonics_frequency_events.len() {
                                for _ in harmonics_frequency_events.len()..next_chord.len() {
                                    harmonics_frequency_events.push_back(Vec::new());
                                    harmonics_velocity_events.push_back(Vec::new());
                                    self.chord_events.push(Event::new());
                                }
                            }

                            let next_velocity = &velocityline.velocities[next_velocity_idx];

                            let max_harmonic_count =
                                usize::max(self.harmonic_count, next_chord.len());

                            for harmonic_idx in 0..max_harmonic_count {
                                let harmonic_event = &mut self.chord_events[harmonic_idx];

                                let next_harmonic_idx =
                                    usize::min(harmonic_idx, next_chord.len() - 1);
                                let next_harmonic = &next_chord[next_harmonic_idx];

                                let next_harmonic_frequency =
                                    next_pitch_frequency * next_harmonic.freq_ratio;

                                let next_harmonic_velocity =
                                    next_harmonic.velocity.value * next_velocity.value;

                                if harmonic_idx < self.harmonic_count {
                                    let start_tick = self.hit_start_tick
                                        + binder::to_ticks(&harmonic_event.delay, hit_ticks_count);

                                    harmonics_frequency_events[harmonic_idx].push(
                                        audio_event::AudioEventParameter::new(
                                            start_tick,
                                            self.hit_end_tick,
                                            harmonic_event.frequency,
                                            next_harmonic_frequency,
                                            harmonic_event.frequency_transition,
                                            false,
                                            false,
                                        ),
                                    );

                                    harmonics_velocity_events[harmonic_idx].push(
                                        audio_event::AudioEventParameter::new(
                                            start_tick,
                                            self.hit_end_tick,
                                            harmonic_event.velocity.value,
                                            next_harmonic_velocity,
                                            harmonic_event.velocity.transition,
                                            harmonic_event.velocity.fadein,
                                            harmonic_event.velocity.fadeout,
                                        ),
                                    );
                                }

                                if harmonic_idx < next_chord.len() {
                                    harmonic_event.delay = next_harmonic.delay;
                                    harmonic_event.frequency = next_harmonic_frequency;
                                    harmonic_event.frequency_transition = next_pitch_transition;
                                    harmonic_event.velocity = PVelocity {
                                        value: next_harmonic_velocity,
                                        transition: if next_harmonic.velocity.transition
                                            == PTransition::None
                                        {
                                            next_velocity.transition
                                        } else {
                                            next_harmonic.velocity.transition
                                        },
                                        fadein: next_harmonic.velocity.fadein
                                            || next_velocity.fadein,
                                        fadeout: next_harmonic.velocity.fadeout
                                            || next_velocity.fadeout,
                                    }
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
        fragment: &PFragment,
        harmonics_frequency_events: &mut VecDeque<Vec<AudioEventParameter>>,
        harmonics_velocity_events: &mut VecDeque<Vec<AudioEventParameter>>,
    ) -> Result<(), failure::Error> {
        match fragment {
            Part(part) => {
                self.create_part_events(
                    binder,
                    ticks_per_beat,
                    part,
                    harmonics_frequency_events,
                    harmonics_velocity_events,
                )?;
            }
            SeqRef(seqref) => {
                let seq = binder.fetch_sequence(&seqref.id)?;
                let mul = seqref.mul.unwrap_or(1);

                for _ in 0..mul {
                    self.create_events(
                        binder,
                        ticks_per_beat,
                        seq,
                        harmonics_frequency_events,
                        harmonics_velocity_events,
                    )?;
                }
            }
            Fragments((fragments, mul)) => {
                for _ in 0..*mul {
                    for fragment in fragments {
                        self.create_fragment_events(
                            binder,
                            ticks_per_beat,
                            fragment,
                            harmonics_frequency_events,
                            harmonics_velocity_events,
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
        sequence: &PSequence,
        harmonics_frequency_events: &mut VecDeque<Vec<AudioEventParameter>>,
        harmonics_velocity_events: &mut VecDeque<Vec<AudioEventParameter>>,
    ) -> Result<(), failure::Error> {
        let ticks_per_beat = match sequence.beat {
            Some(id) => binder.ticks_per_minute / (binder.fetch_beat(id)?).bpm as f32,
            None => ticks_per_beat,
        };

        for fragment in &sequence.fragments {
            self.create_fragment_events(
                binder,
                ticks_per_beat,
                fragment,
                harmonics_frequency_events,
                harmonics_velocity_events,
            )?;
        }
        Ok(())
    }

    pub fn create_last_events(
        &self,
        harmonics_frequency_events_parameters: &mut VecDeque<Vec<AudioEventParameter>>,
        harmonics_velocity_events_parameters: &mut VecDeque<Vec<AudioEventParameter>>,
    ) {
        let hit_end_tick = if self.hit_end_tick == binder::UNDEFINED_TICKS {
            self.tick
        } else {
            self.hit_end_tick
        };
        let hit_ticks_count = (hit_end_tick - self.hit_start_tick) as f32;

        for harmonic_idx in 0..self.harmonic_count {
            let harmonic_event = &self.chord_events[harmonic_idx];

            let start_tick =
                self.hit_start_tick + binder::to_ticks(&harmonic_event.delay, hit_ticks_count);

            if harmonic_event.frequency > 0. {
                harmonics_frequency_events_parameters[harmonic_idx].push(
                    audio_event::AudioEventParameter::new(
                        start_tick,
                        hit_end_tick,
                        harmonic_event.frequency,
                        harmonic_event.frequency,
                        PTransition::None,
                        false,
                        false,
                    ),
                );
            }

            if harmonic_event.velocity.value > 0. {
                harmonics_velocity_events_parameters[harmonic_idx].push(
                    audio_event::AudioEventParameter::new(
                        start_tick,
                        hit_end_tick,
                        harmonic_event.velocity.value,
                        harmonic_event.velocity.value,
                        PTransition::None,
                        harmonic_event.velocity.fadein,
                        true,
                    ),
                );
            }
        }
    }
}

fn limit_overflowing_durations(
    harmonics_frequency_events_parameters: &mut VecDeque<Vec<AudioEventParameter>>,
    harmonics_velocity_events_parameters: &mut VecDeque<Vec<AudioEventParameter>>,
) {
    for frequency_events_parameters in harmonics_frequency_events_parameters {
        for idx in 0..frequency_events_parameters.len() - 1 {
            if frequency_events_parameters[idx].end_tick
                >= frequency_events_parameters[idx + 1].start_tick
            {
                frequency_events_parameters[idx].end_tick =
                    frequency_events_parameters[idx + 1].start_tick;
            }
        }
    }

    for velocity_events_parameters in harmonics_velocity_events_parameters {
        for idx in 0..velocity_events_parameters.len() - 1 {
            if velocity_events_parameters[idx].end_tick
                > velocity_events_parameters[idx + 1].start_tick
            {
                velocity_events_parameters[idx].end_tick =
                    velocity_events_parameters[idx + 1].start_tick;
            }
        }
    }
}

pub fn create_events(
    binder: &Binder,
    sequence: &PSequence,
    bpm: usize,
) -> Result<(VecDeque<AudioEvents>, VecDeque<AudioEvents>), failure::Error> {
    let mut builder = EventsBuilder::new();
    let mut harmonics_frequency_events_parameters = VecDeque::with_capacity(6);
    let mut harmonics_velocity_events_parameters = VecDeque::with_capacity(6);

    let ticks_per_beat = binder.ticks_per_minute / bpm as f32;

    builder.create_events(
        binder,
        ticks_per_beat,
        sequence,
        &mut harmonics_frequency_events_parameters,
        &mut harmonics_velocity_events_parameters,
    )?;

    builder.create_last_events(
        &mut harmonics_frequency_events_parameters,
        &mut harmonics_velocity_events_parameters,
    );

    limit_overflowing_durations(
        &mut harmonics_frequency_events_parameters,
        &mut harmonics_velocity_events_parameters,
    );

    let harmonics_frequency_events: VecDeque<AudioEvents> = harmonics_frequency_events_parameters
        .iter()
        .map(|v| v.iter().map(audio_event::create_from_parameter).collect())
        .collect();

    let harmonics_velocity_events: VecDeque<AudioEvents> = harmonics_velocity_events_parameters
        .iter()
        .map(|v| v.iter().map(audio_event::create_from_parameter).collect())
        .collect();

    Ok((harmonics_frequency_events, harmonics_velocity_events))
}
