use std::f32;

use talker::ctalker;
use talker::data::Data;
use talker::talker::{CTalker, Talker, TalkerBase};
use talker::talker_handler::TalkerHandlerBase;
use talker::voice;

use talkers::tseq::audio_seq::AudioSeq;
use talkers::tseq::midi_seq::MidiSeq;
use talkers::tseq::parser;
use talkers::tseq::parser::Exp;
use talkers::tseq::parsing_result::ParsingResult;
use talkers::tseq::scale::Scale;

pub const MODEL: &str = "Tseq";

const DEFAULT_BPM: usize = 90;

enum Seq {
    Freq(AudioSeq),
    Vel(AudioSeq),
    Midi(MidiSeq),
}

pub const DEFAULT_DATA: &str = ";; Text sequencer";

pub struct Tseq {
    sequences: Vec<Seq>,
    current_events_indexies: Vec<usize>,
}

impl Tseq {
    pub fn new() -> Result<CTalker, failure::Error> {
        let base = TalkerBase::new_data("", MODEL, Data::Text(DEFAULT_DATA.to_string()));

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
        let scale = Scale::tempered();

        match data {
            Data::Text(ref txt) => {
                let mut new_base = base.with(None, None, None);
                let mut sequences: Vec<Seq> = Vec::new();

                let exps = parser::parse(&txt)?;
                {
                    let mut pare = ParsingResult::new();
                    let mut outs = Vec::new();

                    for exp in &exps {
                        match exp {
                            Exp::Beat(ref beat) => {
                                pare.beats.insert(beat.id, &beat);
                            }
                            Exp::PitchLine(ref pitchline) => {
                                let mut pitchs = Vec::new();
                                for pitch in &pitchline.pitchs {
                                    pitchs.push((
                                        scale.fetch_frequency(pitch.id)?,
                                        pitch.progression,
                                    ));
                                }
                                pare.pitchlines.insert(pitchline.id, pitchs);
                            }
                            Exp::Pattern(ref pattern) => {
                                pare.patterns.insert(pattern.id, &pattern);
                            }
                            Exp::VelocityLine(ref velocityline) => {
                                pare.velocitylines.insert(velocityline.id, &velocityline);
                            }
                            Exp::Seq(ref sequence) => {
                                pare.sequences.insert(sequence.id, &sequence);
                            }
                            Exp::FreqOut(_) => outs.push(exp),
                            Exp::VelOut(_) => outs.push(exp),
                            Exp::MidiOut(_) => outs.push(exp),
                            Exp::None => (),
                        }
                    }

                    for out in outs {
                        match out {
                            Exp::FreqOut(seq) => {
                                sequences.push(Seq::Freq(AudioSeq::frequency(
                                    &pare,
                                    &seq,
                                    DEFAULT_BPM,
                                )?));
                                new_base.add_voice(voice::cv(Some(seq.id), 0.));
                            }
                            Exp::VelOut(seq) => {
                                sequences.push(Seq::Vel(AudioSeq::velocity(
                                    &pare,
                                    &seq,
                                    DEFAULT_BPM,
                                )?));
                                new_base.add_voice(voice::audio(Some(seq.id), 0.));
                            }
                            Exp::MidiOut(seq) => {
                                sequences.push(Seq::Midi(MidiSeq::new(&pare, &seq, DEFAULT_BPM)?));
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
                audio_sequence_talk(base, port, tick, ln, &seq, ev_idx, voice_buf)
            }
            Seq::Vel(seq) => {
                let voice_buf = base.voice(port).audio_buffer();
                audio_sequence_talk(base, port, tick, ln, &seq, ev_idx, voice_buf)
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
    voice_buf: &mut [f32],
) -> usize {
    let mut t = tick;
    let end_t = tick + len as i64;
    let mut ev_idx = if current_event_index < seq.events.len() {
        current_event_index
    } else {
        current_event_index - 1
    };

    while ev_idx > 0 && seq.events[ev_idx].start_tick() > end_t {
        ev_idx -= 1;
    }

    while t < end_t {
        while ev_idx < seq.events.len() && seq.events[ev_idx].end_tick() <= t {
            ev_idx += 1;
        }
        let ofset = (t - tick) as usize;
        let out_len = len - ofset;

        if ev_idx < seq.events.len() {
            let ev = &seq.events[ev_idx];

            if ev.start_tick() <= t {
                t = ev.assign_buffer(t, voice_buf, ofset, out_len);
            } else {
                let cur_len = usize::min((ev.start_tick() - t) as usize, out_len);

                for i in ofset..ofset + cur_len {
                    voice_buf[i] = 0.;
                }
                t += cur_len as i64;
            }
        } else {
            for i in ofset..out_len {
                voice_buf[i] = 0.;
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
