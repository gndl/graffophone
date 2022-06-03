use std::f32;

use talker::audio_format::AudioFormat;
use talkers::tseq::audio_event;
use talkers::tseq::audio_event::RAudioEvent;
use talkers::tseq::parser::PFragment::Part;
use talkers::tseq::parser::PFragment::SeqRef;
use talkers::tseq::parser::PSequence;
use talkers::tseq::parser::PTransition;
use talkers::tseq::parsing_result::ParsingResult;

struct EventsBuilder {
    tick: i64,
    start_tick: i64,
    end_tick: i64,
    value: f32,
    transition: PTransition,
}

impl EventsBuilder {
    pub fn new() -> EventsBuilder {
        Self {
            tick: 0,
            start_tick: 0,
            end_tick: -1,
            value: 0.,
            transition: PTransition::None,
        }
    }

    pub fn create_frequency_events(
        &mut self,
        pare: &ParsingResult,
        bpm: usize,
        sequence: &PSequence,
        events: &mut Vec<RAudioEvent>,
    ) -> Result<(), failure::Error> {
        let bpm = match sequence.beat {
            Some(id) => (pare.fetch_beat(id)?).bpm,
            None => bpm,
        };
        let beat_ticks_count = ((AudioFormat::sample_rate() * 60) / bpm) as f32;

        for fragment in &sequence.fragments {
            match fragment {
                Part(part) => {
                    let mut part_is_empty = true;
                    let hitline = pare.fetch_hitline(part.hitline_id)?;
                    let hitline_hits_count = hitline.hits.len();
                    let hitline_ticks_count = (hitline.duration * beat_ticks_count) as i64;
                    let mut hitline_start_tick = self.tick;
                    let mut mul = part.mul.unwrap_or(1.);

                    if hitline_hits_count > 0 && mul > 0. {
                        if let Some(pitchline_id) = part.pitchline_id {
                            let pitchline = pare.fetch_pitchline(pitchline_id)?;
                            let pitchs_count = pitchline.len();

                            if pitchs_count > 0 {
                                part_is_empty = false;

                                let mut hit_idx = 0;
                                let mut pitch_idx = 0;
                                let max_n = usize::max(hitline_hits_count, pitchs_count);

                                while mul > 0. {
                                    let n = if mul < 1. {
                                        ((max_n as f32) * mul) as usize
                                    } else {
                                        max_n
                                    };
                                    for _ in 0..n {
                                        let hit = &hitline.hits[hit_idx];
                                        let start_tick = hitline_start_tick
                                            + (hit.position * beat_ticks_count) as i64;
                                        let (next_value, next_transition) = pitchline[pitch_idx];

                                        let end_tick = if self.end_tick < 0 {
                                            // -1 provide a raising edge on new note
                                            start_tick - 1
                                        } else {
                                            self.end_tick
                                        };

                                        events.push(audio_event::create(
                                            self.start_tick,
                                            end_tick,
                                            self.value,
                                            next_value,
                                            self.transition,
                                        ));

                                        self.start_tick = start_tick;
                                        self.end_tick = match hit.duration {
                                            Some(dur) => {
                                                hitline_start_tick
                                                    + ((hit.position + dur) * beat_ticks_count)
                                                        as i64
                                            }
                                            None => -1,
                                        };
                                        self.value = next_value;
                                        self.transition = next_transition;

                                        hit_idx += 1;

                                        if hit_idx == hitline_hits_count {
                                            hit_idx = 0;
                                            hitline_start_tick += hitline_ticks_count;
                                        }

                                        pitch_idx += 1;
                                        if pitch_idx == pitchs_count {
                                            pitch_idx = 0;
                                        }
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
                    let seq = pare.fetch_sequence(&seqref.id)?;
                    let mul = seqref.mul.unwrap_or(1);

                    for _ in 0..mul {
                        self.create_frequency_events(pare, bpm, seq, events)?;
                    }
                }
            }
        }
        Ok(())
    }

    pub fn create_velocity_events(
        &mut self,
        pare: &ParsingResult,
        bpm: usize,
        sequence: &PSequence,
        events: &mut Vec<RAudioEvent>,
    ) -> Result<(), failure::Error> {
        let bpm = match sequence.beat {
            Some(id) => (pare.fetch_beat(id)?).bpm,
            None => bpm,
        };
        let beat_ticks_count = ((AudioFormat::sample_rate() * 60) / bpm) as f32;

        for fragment in &sequence.fragments {
            match fragment {
                Part(part) => {
                    let mut part_is_empty = true;
                    let hitline = pare.fetch_hitline(part.hitline_id)?;
                    let hitline_hits_count = hitline.hits.len();
                    let hitline_ticks_count = (hitline.duration * beat_ticks_count) as i64;
                    let mut hitline_start_tick = self.tick;
                    let mut mul = part.mul.unwrap_or(1.);

                    if hitline_hits_count > 0 && mul > 0. {
                        if let Some(velocityline_id) = part.velocityline_id {
                            let velocityline = pare.fetch_velocityline(velocityline_id)?;
                            let velos_count = velocityline.velocities.len();

                            if velos_count > 0 {
                                part_is_empty = false;

                                let mut hit_idx = 0;
                                let mut vel_idx = 0;
                                let max_n = usize::max(hitline_hits_count, velos_count);

                                while mul > 0. {
                                    let n = if mul < 1. {
                                        ((max_n as f32) * mul) as usize
                                    } else {
                                        max_n
                                    };
                                    for _ in 0..n {
                                        let hit = &hitline.hits[hit_idx];
                                        let start_tick = hitline_start_tick
                                            + (hit.position * beat_ticks_count) as i64;

                                        let velocity = &velocityline.velocities[vel_idx];
                                        let next_value = velocity.value;
                                        let next_transition = velocity.transition;

                                        let end_tick = if self.end_tick < 0 {
                                            start_tick
                                        } else {
                                            self.end_tick
                                        };

                                        events.push(audio_event::create(
                                            self.start_tick,
                                            end_tick,
                                            self.value,
                                            next_value,
                                            self.transition,
                                        ));

                                        self.start_tick = start_tick;
                                        self.end_tick = match hit.duration {
                                            Some(dur) => {
                                                hitline_start_tick
                                                    + ((hit.position + dur) * beat_ticks_count)
                                                        as i64
                                            }
                                            None => -1,
                                        };
                                        self.value = next_value;
                                        self.transition = next_transition;

                                        hit_idx += 1;

                                        if hit_idx == hitline_hits_count {
                                            hit_idx = 0;
                                            hitline_start_tick += hitline_ticks_count;
                                        }

                                        vel_idx += 1;
                                        if vel_idx == velos_count {
                                            vel_idx = 0;
                                        }
                                    }
                                    mul -= 1.;
                                }
                                if velos_count > hitline_hits_count
                                    && velos_count % hitline_hits_count != 0
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
                    let seq = pare.fetch_sequence(&seqref.id)?;
                    let mul = seqref.mul.unwrap_or(1);

                    for _ in 0..mul {
                        self.create_velocity_events(pare, bpm, seq, events)?;
                    }
                }
            }
        }
        Ok(())
    }

    pub fn create_last_event(&self, events: &mut Vec<RAudioEvent>) {
        if self.value > 0. {
            let end_tick = if self.end_tick < 0 {
                self.tick
            } else {
                self.end_tick
            };
            events.push(audio_event::create(
                self.start_tick,
                end_tick,
                self.value,
                self.value,
                PTransition::None,
            ));
        }
    }
}

pub struct AudioSeq {
    pub events: Vec<RAudioEvent>,
}

impl AudioSeq {
    pub fn frequency(
        pare: &ParsingResult,
        sequence: &PSequence,
        bpm: usize,
    ) -> Result<AudioSeq, failure::Error> {
        let mut builder = EventsBuilder::new();
        let mut events: Vec<RAudioEvent> = Vec::new();

        builder.create_frequency_events(pare, bpm, sequence, &mut events)?;
        builder.create_last_event(&mut events);

        Ok(AudioSeq { events })
    }

    pub fn velocity(
        pare: &ParsingResult,
        sequence: &PSequence,
        bpm: usize,
    ) -> Result<AudioSeq, failure::Error> {
        let mut builder = EventsBuilder::new();
        let mut events: Vec<RAudioEvent> = Vec::new();

        builder.create_velocity_events(pare, bpm, sequence, &mut events)?;
        builder.create_last_event(&mut events);

        Ok(AudioSeq { events })
    }
}
