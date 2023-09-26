use std::collections::VecDeque;
use std::f32;

use talker::audio_format::AudioFormat;
use talkers::tseq::audio_event;
use talkers::tseq::audio_event::AudioEventParameter;
use talkers::tseq::audio_event::RAudioEvent;
use talkers::tseq::binder::Binder;
use talkers::tseq::parser::PFragment::Part;
use talkers::tseq::parser::PFragment::SeqRef;
use talkers::tseq::parser::PSequence;
use talkers::tseq::parser::PTransition;

pub type AudioSeq = Vec<RAudioEvent>;

#[derive(Debug)]
struct Event {
    start_tick: i64,
    end_tick: i64,
    frequency: f32,
    frequency_transition: PTransition,
    velocity: f32,
    velocity_transition: PTransition,
}
impl Event {
    pub fn new() -> Event {
        Self {
            start_tick: 0,
            end_tick: -1,
            frequency: audio_event::DEFAULT_FREQUENCY,
            frequency_transition: PTransition::None,
            velocity: audio_event::DEFAULT_VELOCITY,
            velocity_transition: PTransition::None,
        }
    }
}
struct EventsBuilder {
    tick: i64,
    harmonic_count: usize,
    chord_events: Vec<Event>,
}

impl EventsBuilder {
    pub fn new() -> EventsBuilder {
        Self {
            tick: 0,
            harmonic_count: 0,
            chord_events: Vec::new(),
        }
    }

    pub fn create_events(
        &mut self,
        binder: &Binder,
        bpm: usize,
        sequence: &PSequence,
        harmonics_frequency_events: &mut VecDeque<AudioSeq>,
        harmonics_velocity_events: &mut VecDeque<Vec<AudioEventParameter>>,
    ) -> Result<(), failure::Error> {
        let bpm = match sequence.beat {
            Some(id) => (binder.fetch_beat(id)?).bpm,
            None => bpm,
        };
        let beat_ticks_count = ((AudioFormat::sample_rate() * 60) / bpm) as f32;

        for fragment in &sequence.fragments {
            match fragment {
                Part(part) => {
                    let mut part_is_empty = true;
                    let hitline = binder.fetch_hitline(part.hitline_id)?;
                    let hitline_hits_count = hitline.hits.len();
                    let hitline_ticks_count = (hitline.duration * beat_ticks_count) as i64;
                    let mut hitline_start_tick = self.tick;
                    let mut mul = part.mul.unwrap_or(1.);

                    if hitline_hits_count > 0 && mul > 0. {
                        if let Some(pitchline_id) = part.pitchline_id {
                            let pitchline = binder.fetch_deserialized_pitchline(pitchline_id)?;
                            let pitchs_count = pitchline.len();

                            if pitchs_count > 0 {
                                part_is_empty = false;

                                let chordline =
                                    binder.fetch_deserialized_chordline(&part.chordline_id)?;
                                let chords_count = chordline.len();

                                let velocityline =
                                    binder.fetch_velocityline(&part.velocityline_id)?;
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
                                            + (next_hit.position * beat_ticks_count) as i64;
                                        let next_hit_end_tick =
                                            next_hit.duration.map_or(-1, |dur| {
                                                next_hit_start_tick
                                                    + (dur * beat_ticks_count) as i64
                                            });

                                        let (next_pitch_frequency, next_pitch_transition) =
                                            pitchline[next_pitch_idx];

                                        let next_chord = &chordline[next_chord_idx];

                                        if next_chord.len() > harmonics_frequency_events.len() {
                                            for _ in
                                                harmonics_frequency_events.len()..next_chord.len()
                                            {
                                                harmonics_frequency_events.push_back(Vec::new());
                                                harmonics_velocity_events.push_back(Vec::new());
                                                self.chord_events.push(Event::new());
                                            }
                                        }

                                        let next_velocity =
                                            &velocityline.velocities[next_velocity_idx];

                                        let max_harmonic_count =
                                            usize::max(self.harmonic_count, next_chord.len());

                                        for harmonic_idx in 0..max_harmonic_count {
                                            let harmonic_event =
                                                &mut self.chord_events[harmonic_idx];

                                            let next_harmonic_idx =
                                                usize::min(harmonic_idx, next_chord.len() - 1);
                                            let next_harmonic = &next_chord[next_harmonic_idx];

                                            let next_harmonic_start_tick =
                                                next_hit_start_tick + next_harmonic.delay_ticks;

                                            let next_harmonic_frequency =
                                                next_pitch_frequency * next_harmonic.freq_ratio;

                                            let next_harmonic_velocity =
                                                next_harmonic.velocity * next_velocity.value;

                                            if harmonic_idx < self.harmonic_count {
                                                let harmonic_end_tick =
                                                    if harmonic_event.end_tick < 0 {
                                                        next_harmonic_start_tick
                                                    } else {
                                                        harmonic_event.end_tick
                                                    };

                                                harmonics_frequency_events[harmonic_idx].push(
                                                    audio_event::create(
                                                        harmonic_event.start_tick,
                                                        harmonic_end_tick - 1, // -1 provide a raising edge on new note
                                                        harmonic_event.frequency,
                                                        next_harmonic_frequency,
                                                        harmonic_event.frequency_transition,
                                                    ),
                                                );

                                                harmonics_velocity_events[harmonic_idx].push(
                                                    audio_event::AudioEventParameter::new(
                                                        harmonic_event.start_tick,
                                                        harmonic_end_tick,
                                                        harmonic_event.velocity,
                                                        next_harmonic_velocity,
                                                        harmonic_event.velocity_transition,
                                                    ),
                                                );
                                            }

                                            if harmonic_idx < next_chord.len() {
                                                harmonic_event.start_tick =
                                                    next_harmonic_start_tick;
                                                harmonic_event.end_tick = next_hit_end_tick;
                                                harmonic_event.frequency = next_harmonic_frequency;
                                                harmonic_event.frequency_transition =
                                                    next_pitch_transition;
                                                harmonic_event.velocity = next_harmonic_velocity;
                                                harmonic_event.velocity_transition =
                                                    if next_harmonic.velocity_transition
                                                        == PTransition::None
                                                    {
                                                        next_velocity.transition
                                                    } else {
                                                        next_harmonic.velocity_transition
                                                    };
                                            }
                                        }
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

                                        next_velocity_idx =
                                            if next_velocity_idx < velocities_count - 1 {
                                                next_velocity_idx + 1
                                            } else {
                                                0
                                            };
                                    }
                                    mul -= 1.;
                                }
                                if pitchs_count > hitline_hits_count
                                    && pitchs_count % hitline_hits_count != 0
                                {
                                    hitline_start_tick += hitline_ticks_count;
                                }
                                self.tick = hitline_start_tick;
                            }
                        }
                    }
                    if part_is_empty {
                        self.tick += (hitline_ticks_count as f32 * mul.ceil()) as i64;
                    }
                }
                SeqRef(seqref) => {
                    let seq = binder.fetch_sequence(&seqref.id)?;
                    let mul = seqref.mul.unwrap_or(1);

                    for _ in 0..mul {
                        self.create_events(
                            binder,
                            bpm,
                            seq,
                            harmonics_frequency_events,
                            harmonics_velocity_events,
                        )?;
                    }
                }
            }
        }
        Ok(())
    }

    pub fn create_last_frequency_event(&self, harmonic_idx: usize, events: &mut AudioSeq) {
        if self.chord_events[harmonic_idx].frequency > 0. {
            let end_tick = if self.chord_events[harmonic_idx].end_tick < 0 {
                self.tick
            } else {
                self.chord_events[harmonic_idx].end_tick
            };
            events.push(audio_event::create(
                self.chord_events[harmonic_idx].start_tick,
                end_tick,
                self.chord_events[harmonic_idx].frequency,
                self.chord_events[harmonic_idx].frequency,
                PTransition::None,
            ));
        }
    }

    pub fn create_last_velocity_event(&self, harmonic_idx: usize, events: &mut AudioSeq) {
        if self.chord_events[harmonic_idx].velocity > 0. {
            let end_tick = if self.chord_events[harmonic_idx].end_tick < 0 {
                self.tick
            } else {
                self.chord_events[harmonic_idx].end_tick
            };
            events.push(audio_event::create(
                self.chord_events[harmonic_idx].start_tick,
                end_tick,
                self.chord_events[harmonic_idx].velocity,
                self.chord_events[harmonic_idx].velocity,
                PTransition::None,
            ));
        }
    }

    pub fn create_last_events(
        &self,
        harmonic_count: usize,
        mut harmonics_frequency_events: VecDeque<AudioSeq>,
        mut harmonics_velocity_events_parameters: VecDeque<Vec<AudioEventParameter>>,
    ) -> Result<(VecDeque<AudioSeq>, VecDeque<AudioSeq>), failure::Error> {
        for harmonic_idx in 0..harmonic_count {
            //let mut harmonic_frequency_events = &mut harmonics_frequency_events[harmonic_idx];
            self.create_last_frequency_event(
                harmonic_idx,
                &mut harmonics_frequency_events[harmonic_idx],
            );
        }
        for harmonic_idx in 0..harmonics_velocity_events_parameters.len() {
            let harmonic_velocity_events_parameters =
                &mut harmonics_velocity_events_parameters[harmonic_idx];

            if self.chord_events[harmonic_idx].velocity == audio_event::DEFAULT_VELOCITY
                && harmonic_velocity_events_parameters
                    .iter()
                    .all(|p| p.start_value == audio_event::DEFAULT_VELOCITY)
            {
                harmonic_velocity_events_parameters.clear();
            }
        }
        let mut harmonics_velocity_events: VecDeque<AudioSeq> =
            harmonics_velocity_events_parameters
                .iter()
                .map(|v| v.iter().map(audio_event::create_from_parameter).collect())
                .collect();

        for harmonic_idx in 0..harmonic_count {
            if !harmonics_velocity_events[harmonic_idx].is_empty() {
                self.create_last_velocity_event(
                    harmonic_idx,
                    &mut harmonics_velocity_events[harmonic_idx],
                );
            }
        }
        Ok((harmonics_frequency_events, harmonics_velocity_events))
    }
}

pub fn create_events(
    binder: &Binder,
    sequence: &PSequence,
    bpm: usize,
) -> Result<(VecDeque<AudioSeq>, VecDeque<AudioSeq>), failure::Error> {
    let mut builder = EventsBuilder::new();
    let mut harmonics_frequency_events = VecDeque::new();
    let mut harmonics_velocity_events_parameters = VecDeque::new();

    builder.create_events(
        binder,
        bpm,
        sequence,
        &mut harmonics_frequency_events,
        &mut harmonics_velocity_events_parameters,
    )?;
    builder.create_last_events(
        builder.harmonic_count,
        harmonics_frequency_events,
        harmonics_velocity_events_parameters,
    )
}
