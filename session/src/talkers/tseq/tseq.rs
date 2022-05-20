use std::f32;

use talker::ctalker;
use talker::data::Data;
//use talker::ear;
//use talker::ear::Init;
use talker::talker::{CTalker, Talker, TalkerBase};
use talker::talker_handler::TalkerHandlerBase;
use talker::voice;

use talkers::tseq::freq_seq::FreqSeq;
use talkers::tseq::parser;
use talkers::tseq::parser::{Exp, PSequence};
use talkers::tseq::parsing_result::ParsingResult;
use talkers::tseq::scale::Scale;

pub const MODEL: &str = "Tseq";

const DEFAULT_BPM: usize = 90;

#[derive(Clone, Copy)]
pub enum Progression {
    I,
    L,
    D,
    CS,
}

struct VelEvent {
    start_tick: i64,
    end_tick: i64,
    start_vel: f32,
    end_vel: f32,
    prog: Progression,
}
pub struct VelSeq {
    current_event: usize,
    events: Vec<VelEvent>,
}

struct MidiEvent {}
pub struct MidiSeq {
    current_event: usize,
    events: Vec<MidiEvent>,
}

enum Seq {
    Freq(FreqSeq),
    Vel(VelSeq),
    Midi(MidiSeq),
}

pub struct Tseq {
    sequences: Vec<Seq>,
    current_events_indexies: Vec<usize>,
}

impl Tseq {
    pub fn new() -> Result<CTalker, failure::Error> {
        let base = TalkerBase::new("", MODEL);

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
        _: &TalkerBase,
        data: Data,
    ) -> Result<Option<TalkerBase>, failure::Error> {
        let scale = Scale::tempered();

        match data {
            Data::Text(ref txt) => {
                let mut base = TalkerBase::new("", MODEL);
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
                                let mut freqs = Vec::new();
                                for pitch in &pitchline.pitchs {
                                    freqs.push(scale.fetch_frequency(pitch)?);
                                }
                                pare.pitchlines.insert(pitchline.id, freqs);
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
                                sequences.push(Seq::Freq(FreqSeq::new(&pare, &seq)?));
                                base.add_voice(voice::cv(Some(seq.id), 0.));
                            }
                            Exp::VelOut(seq) => {
                                let mut events: Vec<VelEvent> = Vec::new();
                                create_velocity_events(
                                    &mut pare,
                                    DEFAULT_BPM,
                                    &seq,
                                    0,
                                    &mut events,
                                )?;
                                sequences.push(Seq::Vel(VelSeq {
                                    current_event: 0,
                                    events,
                                }));
                                base.add_voice(voice::cv(Some(seq.id), 0.));
                            }
                            Exp::MidiOut(seq) => {
                                let mut events: Vec<MidiEvent> = Vec::new();
                                create_midi_events(&mut pare, DEFAULT_BPM, &seq, 0, &mut events)?;
                                sequences.push(Seq::Midi(MidiSeq {
                                    current_event: 0,
                                    events,
                                }));
                                base.add_voice(voice::atom(Some(seq.id), None));
                            }
                            _ => (),
                        }
                    }

                    /*        base.add_ear(ear::cv(Some("freq"), 0., 20000., 440., &Init::DefValue)?);
                           base.add_ear(ear::audio(Some("phase"), -1., 1., 0., &Init::DefValue)?);

                    */
                }
                self.current_events_indexies = vec![0; sequences.len()];
                self.sequences = sequences;
                base.set_data(data);
                Ok(Some(base))
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
            Seq::Freq(freq_seq) => frequency_sequence_talk(base, port, tick, ln, &freq_seq, ev_idx),
            Seq::Vel(vel_seq) => velocity_sequence_talk(base, port, tick, ln, &vel_seq, ev_idx),
            Seq::Midi(midi_seq) => midi_sequence_talk(base, port, tick, ln, &midi_seq, ev_idx),
        };
        ln
    }
}

fn create_velocity_events(
    _pare: &mut ParsingResult,
    _bpm: usize,
    _sequence: &PSequence,
    tick: i64,
    _events: &mut Vec<VelEvent>,
) -> Result<i64, failure::Error> {
    Ok(tick)
}

fn create_midi_events(
    _pare: &mut ParsingResult,
    _bpm: usize,
    _sequence: &PSequence,
    tick: i64,
    _events: &mut Vec<MidiEvent>,
) -> Result<i64, failure::Error> {
    Ok(tick)
}

fn frequency_sequence_talk(
    base: &TalkerBase,
    port: usize,
    tick: i64,
    len: usize,
    freq_seq: &FreqSeq,
    current_event_index: usize,
) -> usize {
    let voice_buf = base.voice(port).cv_buffer();
    let mut t = tick;
    let end_t = tick + len as i64;
    let mut ev_idx = current_event_index;

    while ev_idx > 0 && freq_seq.events[ev_idx].start_tick > end_t {
        ev_idx -= 1;
    }

    while t < end_t {
        while ev_idx < freq_seq.events.len() && freq_seq.events[ev_idx].end_tick <= t {
            ev_idx += 1;
        }
        let ofset = (t - tick) as usize;
        let out_len = len - ofset;

        if ev_idx < freq_seq.events.len() {
            let mut ev = &freq_seq.events[ev_idx];

            if ev.start_tick <= t {
                let cur_len = usize::min((ev.end_tick - t) as usize, out_len);

                for i in ofset..ofset + cur_len {
                    voice_buf[i] = ev.start_freq;
                }
                t += cur_len as i64;
            } else {
                let cur_len = usize::min((ev.start_tick - t) as usize, out_len);

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

fn velocity_sequence_talk(
    base: &TalkerBase,
    port: usize,
    tick: i64,
    len: usize,
    vel_seq: &VelSeq,
    current_event_index: usize,
) -> usize {
    current_event_index
}

fn midi_sequence_talk(
    base: &TalkerBase,
    port: usize,
    tick: i64,
    len: usize,
    midi_seq: &MidiSeq,
    current_event_index: usize,
) -> usize {
    current_event_index
}
