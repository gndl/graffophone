use std::f32;

use talker::ctalker;
use talker::data::Data;
use talker::talker::{CTalker, Talker, TalkerBase};
use talker::talker_handler::TalkerHandlerBase;
use talker::voice;

use talkers::tseq::audio_seq::AudioSeq;
use talkers::tseq::binder::Binder;
use talkers::tseq::midi_seq::MidiSeq;
use talkers::tseq::parser::Expression;
use talkers::tseq::syntax::SYNTAX_DESCRIPTION;
use talkers::tseq::{audio_seq, parser};

pub const MODEL: &str = "Tseq";

const DEFAULT_BPM: usize = 90;

enum Seq {
    Freq(AudioSeq),
    Vel(AudioSeq),
    Midi(MidiSeq),
}

pub struct Tseq {
    sequences: Vec<Seq>,
    current_events_indexies: Vec<usize>,
}

impl Tseq {
    pub fn new() -> Result<CTalker, failure::Error> {
        let base = TalkerBase::new_data("", MODEL, Data::Text(SYNTAX_DESCRIPTION.to_string()));

        Ok(ctalker!(
            base,
            Self {
                sequences: Vec::new(),
                current_events_indexies: Vec::new()
            }
        ))
    }

    pub fn descriptor() -> TalkerHandlerBase {
        TalkerHandlerBase::new("Oscillator", MODEL, MODEL)
    }
}

impl Talker for Tseq {
    fn set_data_update(
        &mut self,
        base: &TalkerBase,
        data: Data,
    ) -> Result<Option<TalkerBase>, failure::Error> {
        match data {
            Data::Text(ref txt) => {
                let mut new_base = base.with(None, None, None);
                let mut sequences: Vec<Seq> = Vec::new();
                let input = format!("{}\n", txt);

                let exps = parser::parse(&input)?;
                {
                    let mut binder = Binder::new();
                    let mut outs = Vec::new();

                    for exp in &exps {
                        match exp {
                            Expression::Beat(ref beat) => {
                                binder.beats.insert(beat.id, &beat);
                            }
                            Expression::Chord(ref chord) => {
                                binder.chords.insert(chord.id, &chord);
                            }
                            Expression::Attack(ref attack) => {
                                binder.attacks.insert(attack.id, &attack);
                            }
                            Expression::ChordLine(ref line) => {
                                binder.chordlines.push(&line);
                            }
                            Expression::PitchLine(ref line) => {
                                binder.pitchlines.push(&line);
                            }
                            Expression::HitLine(ref line) => {
                                binder.hitlines.insert(line.id, &line);
                            }
                            Expression::DurationLine(ref line) => {
                                binder.durationlines.insert(line.id, &line);
                            }
                            Expression::VelocityLine(ref line) => {
                                binder.velocitylines.insert(line.id, &line);
                            }
                            Expression::Seq(ref sequence) => {
                                binder.sequences.insert(sequence.id, &sequence);
                            }
                            Expression::SeqOut(_) => outs.push(exp),
                            Expression::MidiOut(_) => outs.push(exp),
                            Expression::None => (),
                        }
                    }

                    binder.deserialize()?;

                    for out in outs {
                        match out {
                            Expression::SeqOut(seq) => {
                                let (mut harmonics_frequency_events, mut harmonics_velocity_events) =
                                    audio_seq::create_events(&binder, &seq, DEFAULT_BPM)?;

                                let harmonics_count = harmonics_frequency_events.len();
                                let display_harmonic_num = harmonics_count > 1;

                                for idx in 0..harmonics_count {
                                    let tag_base = if display_harmonic_num {
                                        format!("{}.{}", seq.id, harmonics_count - idx)
                                    } else {
                                        seq.id.to_string()
                                    };

                                    if let Some(harmonic_frequency_events) =
                                        harmonics_frequency_events.pop()
                                    {
                                        sequences.push(Seq::Freq(harmonic_frequency_events));

                                        let tag = format!("{}.freq", tag_base);
                                        new_base.add_voice(voice::cv(Some(&tag), 0.));
                                    }
                                    if let Some(harmonic_velocity_events) =
                                        harmonics_velocity_events.pop()
                                    {
                                        if !harmonic_velocity_events.is_empty() {
                                            sequences.push(Seq::Vel(harmonic_velocity_events));

                                            let tag = format!("{}.gain", tag_base);
                                            new_base.add_voice(voice::audio(Some(&tag), 0.));
                                        }
                                    }
                                }
                            }
                            Expression::MidiOut(seq) => {
                                sequences.push(Seq::Midi(MidiSeq::new(
                                    &binder,
                                    &seq,
                                    DEFAULT_BPM,
                                )?));
                                new_base.add_voice(voice::atom(Some(seq.id), None));
                            }
                            _ => (),
                        }
                    }
                }
                self.current_events_indexies = vec![0; sequences.len()];
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
        let ev_idx = self.current_events_indexies[port];

        self.current_events_indexies[port] = match &self.sequences[port] {
            Seq::Freq(seq) => {
                let voice_buf = base.voice(port).cv_buffer();
                audio_sequence_talk(base, port, tick, ln, &seq, ev_idx, false, voice_buf)
            }
            Seq::Vel(seq) => {
                let voice_buf = base.voice(port).audio_buffer();
                audio_sequence_talk(base, port, tick, ln, &seq, ev_idx, true, voice_buf)
            }
            Seq::Midi(seq) => midi_sequence_talk(base, port, tick, ln, &seq, ev_idx),
        };
        ln
    }
}

fn audio_sequence_talk(
    _base: &TalkerBase,
    _port: usize,
    tick: i64,
    len: usize,
    seq: &AudioSeq,
    current_event_index: usize,
    conservative_off: bool,
    voice_buf: &mut [f32],
) -> usize {
    let mut t = tick;
    let end_t = tick + len as i64;
    let mut ev_idx = if current_event_index < seq.len() {
        current_event_index
    } else {
        current_event_index - 1
    };

    while ev_idx > 0 && seq[ev_idx].start_tick() > end_t {
        ev_idx -= 1;
    }

    while t < end_t {
        while ev_idx < seq.len() && seq[ev_idx].end_tick() <= t {
            ev_idx += 1;
        }
        let mut ofset = (t - tick) as usize;
        let out_len = len - ofset;

        if ev_idx < seq.len() {
            let ev = &seq[ev_idx];

            if ev.start_tick() <= t {
                t = ev.assign_buffer(t, voice_buf, ofset, out_len);
            } else {
                let cur_len = usize::min((ev.start_tick() - t) as usize, out_len);

                let end = if ofset == 0 {
                    voice_buf[0] = voice_buf[len - 1];
                    ofset = 1;
                    cur_len
                } else {
                    ofset + cur_len
                };

                let off_value = if conservative_off {
                    voice_buf[ofset - 1]
                } else {
                    0.
                };

                for i in ofset..end {
                    voice_buf[i] = off_value;
                }
                t += cur_len as i64;
            }
        } else {
            if conservative_off {
                let end_coef = 0.9999;

                if ofset == 0 {
                    voice_buf[0] = voice_buf[len - 1] * end_coef;
                    ofset = 1;
                }
                for i in ofset..len {
                    voice_buf[i] = voice_buf[i - 1] * end_coef;
                }
            } else {
                for i in ofset..len {
                    voice_buf[i] = 0.;
                }
            }
            break;
        }
    }
    ev_idx
}

fn midi_sequence_talk(
    _base: &TalkerBase,
    _port: usize,
    _tick: i64,
    _len: usize,
    _midi_seq: &MidiSeq,
    current_event_index: usize,
) -> usize {
    current_event_index
}
