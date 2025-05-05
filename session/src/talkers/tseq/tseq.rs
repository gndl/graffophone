use std::f32;

use talker::audio_format::AudioFormat;
use talker::ctalker;
use talker::data::Data;
use talker::talker::Language;
use talker::talker::{CTalker, Talker, TalkerBase};
use talker::talker_handler::TalkerHandlerBase;

use talkers::tseq::audio_event::{self, AudioEvents, Shapes};
use talkers::tseq::binder::Binder;
use talkers::tseq::midi_seq::MidiSeq;
use talkers::tseq::parser::Expression;
use talkers::tseq::syntax::{SYNTAX_DESCRIPTION, TSEQ_LANGUAGE_DEFINITION, TSEQ_LANGUAGE_ID};
use talkers::tseq::{sequence, parser};
use talkers::tseq::sequence::EventReminder;
use scale::scale;

pub const MODEL: &str = "Tseq";


enum Seq {
    Trig(Vec<i64>),
    Freq(AudioEvents),
    Vel(AudioEvents),
    Midi(MidiSeq),
}

pub struct Tseq {
    scales: scale::Collection,
    shapes: Shapes,
    sequences: Vec<Seq>,
    events_reminder: Vec<EventReminder>,
}

impl Tseq {
    pub fn new(base: TalkerBase) -> Result<CTalker, failure::Error> {
        base.set_data(Data::Text(SYNTAX_DESCRIPTION.to_string()));

        Ok(ctalker!(
            base,
            Self {
                scales: scale::Collection::new(),
                shapes: Shapes::empty(),
                sequences: Vec::new(),
                events_reminder: Vec::new()
            }
        ))
    }

    pub fn descriptor() -> TalkerHandlerBase {
        TalkerHandlerBase::builtin("Sequencer", MODEL, MODEL)
    }

    fn build_sequences(
        &self,
        expressions: &Vec<Expression>,
        base: &mut TalkerBase,
    ) -> Result<(Shapes, Vec<Seq>), failure::Error> {
        let mut envelopes = Vec::new();
        let mut sequences: Vec<Seq> = Vec::new();

        let sample_rate = AudioFormat::sample_rate();
        let mut shapes = Shapes::new(sample_rate);

        let mut binder = Binder::new(sample_rate);
        let mut outs = Vec::new();

        for exp in expressions {
            match exp {
                Expression::Beat(ref beat) => {
                    binder.add_beat(beat)?;
                }
                Expression::Scale(ref scale) => {
                    binder.add_scale(scale)?;
                }
                Expression::Chord(ref chord) => {
                    binder.add_chord(chord)?;
                }
                Expression::Attack(ref attack) => {
                    binder.add_attack(attack)?;
                }
                Expression::ChordLine(ref line) => {
                    binder.add_chordline(line)?;
                }
                Expression::PitchLine(ref line) => {
                    binder.add_pitchline(line)?;
                }
                Expression::HitLine(ref line) => {
                    binder.add_hitline(line)?;
                }
                Expression::DurationLine(ref line) => {
                    binder.add_duration(line)?;
                }
                Expression::VelocityLine(ref line) => {
                    binder.add_velocityline(line)?;
                }
                Expression::Envelope(ref envelope) => {
                    envelopes.push(binder.add_envelope(&shapes, envelope, envelopes.len())?);
                }
                Expression::Seq(ref sequence) => {
                    binder.add_sequence(sequence)?;
                }
                Expression::SeqOut(_) => outs.push(exp),
                Expression::MidiOut(_) => outs.push(exp),
                Expression::None => (),
            }
        }

        shapes.set_envelopes(envelopes);

        binder.check_sequences()?;
        binder.deserialize(&self.scales)?;

        for out in outs {
            match out {
                Expression::SeqOut(seq) => {
                    let harmonics_sequence_events = sequence::create_events(&binder, &seq)?;

                    let (mut harmonics_frequency_events, mut harmonics_velocity_events) =
                        audio_event::create_from_sequences(&shapes, &harmonics_sequence_events);

                    let harmonics_count = harmonics_frequency_events.len();
                    let display_harmonic_num = harmonics_count > 1;

                    for idx in 0..harmonics_count {
                        let tag_base = if display_harmonic_num {
                            format!("{}.{}", seq.id, idx + 1)
                        } else {
                            seq.id.to_string()
                        };

                        if let Some(harmonic_frequency_events) =
                            harmonics_frequency_events.pop_front()
                        {
                            // Creation of triggers corresponding to events
                            let mut events_start_ticks =
                                Vec::with_capacity(harmonic_frequency_events.len());

                            for ev in &harmonic_frequency_events {
                                events_start_ticks.push(ev.start_tick());
                            }

                            // Add trigger sequence and output
                            sequences.push(Seq::Trig(events_start_ticks));

                            let trig_tag = format!("{}.trig", tag_base);
                            base.add_cv_voice(Some(&trig_tag), 0.);

                            // Add frequency sequence and output
                            sequences.push(Seq::Freq(harmonic_frequency_events));

                            let freq_tag = format!("{}.freq", tag_base);
                            base.add_cv_voice(Some(&freq_tag), 0.);
                        }
                        if let Some(harmonic_velocity_events) =
                            harmonics_velocity_events.pop_front()
                        {
                            if !harmonic_velocity_events.is_empty() {
                                // Add velocity sequence and output
                                sequences.push(Seq::Vel(harmonic_velocity_events));

                                let tag = format!("{}.gain", tag_base);
                                base.add_audio_voice(Some(&tag), 0.);
                            }
                        }
                    }
                }
                Expression::MidiOut(seq) => {
                    sequences.push(Seq::Midi(MidiSeq::new(
                        &binder,
                        &seq,
                    )?));
                    base.add_atom_voice(Some(seq.id), None);
                }
                _ => (),
            }
        }
        Ok((shapes, sequences))
    }
}

impl Talker for Tseq {
    fn activate(&mut self) {}

    fn deactivate(&mut self) {
        self.events_reminder.clear();

        for _ in 0..self.sequences.len() {
            self.events_reminder.push(EventReminder::new());
        }
    }

    fn data_language(&self) -> Option<Language> {
        Some(Language {id: TSEQ_LANGUAGE_ID.to_string(), definition: Some(TSEQ_LANGUAGE_DEFINITION.to_string())})
    }

    fn set_data_update(
        &mut self,
        base: &TalkerBase,
        data: Data,
    ) -> Result<Option<TalkerBase>, failure::Error> {
        match data {
            Data::Text(ref txt) => {
                let mut new_base = base.with(None, None, None);
                let input = format!("{}\n", txt);

                let expressions = parser::parse(&input)?;
                let (shapes, sequences) = self.build_sequences(&expressions, &mut new_base)?;

                self.events_reminder = Vec::with_capacity(sequences.len());

                for _ in 0..sequences.len() {
                    self.events_reminder.push(EventReminder::new());
                }

                self.shapes = shapes;
                self.sequences = sequences;

                new_base.set_data(data);
                Ok(Some(new_base))
            }
            _ => Err(failure::err_msg(format!(
                "tseq data type {} is not Text",
                data.type_str()
            ))),
        }
    }

    fn talk(&mut self, base: &TalkerBase, port: usize, tick: i64, len: usize) -> usize {
        let ln = base.listen(tick, len);
        let ev_rmd = &mut self.events_reminder[port];

        match &self.sequences[port] {
            Seq::Trig(events_start_ticks) => {
                let voice_buf = base.voice(port).cv_buffer();
                trigger_sequence_talk(tick, ln, events_start_ticks, ev_rmd, voice_buf);
            }
            Seq::Freq(audio_events) => {
                let voice_buf = base.voice(port).cv_buffer();
                audio_sequence_talk(
                    &self.shapes,
                    tick,
                    ln,
                    &audio_events,
                    ev_rmd,
                    false,
                    voice_buf,
                );
            }
            Seq::Vel(audio_events) => {
                let voice_buf = base.voice(port).audio_buffer();
                audio_sequence_talk(
                    &self.shapes,
                    tick,
                    ln,
                    &audio_events,
                    ev_rmd,
                    true,
                    voice_buf,
                );
            }
            Seq::Midi(seq) => {
                let voice_buf = base.voice(port).atom_buffer();
                seq.talk(tick, ln, ev_rmd, voice_buf);
            }
        };
        ln
    }
}

fn audio_sequence_talk(
    shapes: &Shapes,
    tick: i64,
    len: usize,
    audio_events: &AudioEvents,
    event_reminder: &mut EventReminder,
    conservative_off: bool,
    voice_buf: &mut [f32],
) {
    let mut t = tick;
    let end_t = tick + len as i64;
    let ev_count = audio_events.len();
    let mut ev_idx = event_reminder.index;
    let mut last_value = event_reminder.last_value;

    if ev_idx < ev_count {
        while ev_idx > 0 && audio_events[ev_idx].start_tick() > end_t {
            ev_idx -= 1;
        }
    }

    let mut ofset = 0;

    while t < end_t {
        while ev_idx < ev_count && audio_events[ev_idx].end_tick() <= t {
            ev_idx += 1;
        }
        let out_len = len - ofset;

        if ev_idx < ev_count {
            let ev = &audio_events[ev_idx];
            let ev_start_tick = ev.start_tick();

            if ev_start_tick <= t {
                t = ev.assign_buffer(shapes, t, voice_buf, ofset, out_len);
            } else {
                let cur_len = out_len.min((ev_start_tick - t) as usize);

                last_value = if conservative_off { last_value } else { 0. };

                for i in ofset..(ofset + cur_len) {
                    voice_buf[i] = last_value;
                }
                t += cur_len as i64;
            }
        } else {
            for i in ofset..len {
                voice_buf[i] = 0.;
            }

            t += out_len as i64;
        }
        ofset = (t - tick) as usize;
        last_value = voice_buf[ofset.max(1) - 1];
    }
    event_reminder.index = ev_idx;
    event_reminder.last_value = last_value;
}

fn trigger_sequence_talk(
    tick: i64,
    len: usize,
    events_start_ticks: &Vec<i64>,
    event_reminder: &mut EventReminder,
    voice_buf: &mut [f32],
) {
    let end_t = tick + len as i64;
    let mut ev_idx = event_reminder.index;
    let ev_count = events_start_ticks.len();

    voice_buf.fill(0.);

    while ev_idx > 0 && events_start_ticks[ev_idx - 1] >= tick {
        ev_idx -= 1;
    }
    while ev_idx < ev_count && events_start_ticks[ev_idx] < tick {
        ev_idx += 1;
    }

    while ev_idx < ev_count {
        let start_tick = events_start_ticks[ev_idx];

        if start_tick < end_t {
            let i = (events_start_ticks[ev_idx] - tick) as usize;
            voice_buf[i] = 1.;
            ev_idx += 1;
        } else {
            break;
        }
    }
    event_reminder.index = ev_idx;
}
