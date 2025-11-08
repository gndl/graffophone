use std::str::FromStr;

use talkers::tseq::binder::Binder;
use talkers::tseq::sequence;
use talkers::tseq::sequence::EventReminder;
use talkers::tseq::parser::PMidiSequence;
use talker::horn::MAtomBuf;

const FREQ_0: f64 = 8.175799;
const NOTE_OFF: u8 = 0x80;
const NOTE_ON: u8 = 0x90;
const CONTROLLER: u8 = 0xB0;
const CTRL_BANK_SELECT_MSB: u8 = 0x00;
const CTRL_BANK_SELECT_LSB: u8 = 0x20;
const CTRL_VOLUME: u8 = 0x07;
const CTRL_BALANCE: u8 = 0x08;
const CTRL_PAN: u8 = 0x0A;
const PROGRAM_CHANGE: u8 = 0xC0;

fn midi_to_freq(code: u8) -> f32 {
    (FREQ_0 * (code as f64 / 12.).exp2()) as f32
}
#[test]
fn test_midi_to_freq() {
    assert_eq!(midi_to_freq(0), 8.175799);
    assert_eq!(midi_to_freq(69), 440.);
}

fn freq_to_midi(freq: f32) -> u8 {
    let code = 12. * (freq as f64 / FREQ_0).log2();
    (code.round() as u8) & 0x7F
}
#[test]
fn test_freq_to_midi() {
    assert_eq!(freq_to_midi(440.), 69);
}

fn freq_to_midi_pitch_bend(freq: f32) -> (u8, Option<u16>) {
    let code = 12. * (freq as f64 / FREQ_0).log2();
    let icode = code.round();
    let pitch_bend = (((code - icode) * 4096.) as i64 + 8192) as u16;
    let opb = if pitch_bend == 8192 { None} else {Some(pitch_bend)};
    (icode as u8, opb)
}
#[test]
fn test_freq_to_midi_pitch_bend() {
    assert_eq!(freq_to_midi_pitch_bend(440.), (69, None));
}

#[derive(Debug)]
struct MidiEvent {
    tick: i64,
    data: Vec<u8>,
}

pub struct MidiSeq {
    controller_events: Vec<MidiEvent>,
    program_change_events: Vec<MidiEvent>,
    events: Vec<MidiEvent>,
    midi_urid: lv2_raw::LV2Urid,
}

impl MidiSeq {
    pub fn new(
        binder: &Binder,
        sequence: &PMidiSequence,
        midi_urid: lv2_raw::LV2Urid,
    ) -> Result<MidiSeq, failure::Error> {
        if sequence.channels.len() > 16 {
            return Err(failure::err_msg(format!("Midi output {} have {} channels instead of 16 maximum!", sequence.id, sequence.channels.len())))
        }

        let mut controller_events = Vec::with_capacity(16);
        let mut program_change_events = Vec::with_capacity(16);
        let mut events = Vec::with_capacity(1024);
        let mut channel_number: u8 = 0;

        for channel in &sequence.channels {
            let seq = binder.fetch_sequence(&channel.seq_id)?;
            let mut bank_select_msb = 0;
            let mut bank_select_lsb = 0;

            // Channel configuration events
            for attribute in &channel.attributes {
                let ctrl_type = if attribute.label == "MSB" {
                    CTRL_BANK_SELECT_MSB
                } else if attribute.label == "LSB" {
                    CTRL_BANK_SELECT_LSB
                } else if attribute.label.starts_with("vol") {
                    CTRL_VOLUME
                } else if attribute.label == "bal" {
                    CTRL_BALANCE
                } else if attribute.label.starts_with("pan") {
                    CTRL_PAN
                } else {
                    match u8::from_str(attribute.label) {
                        Ok(ct) => ct,
                        Err(_) => {
                            return Err(failure::err_msg(format!("Midi controller type {} unknown!", attribute.label)))
                        }
                    }
                };

                let ctrl_value = match u8::from_str(attribute.value) {
                    Ok(cv) => cv,
                    Err(_) => return Err(failure::err_msg(format!("Midi controller value {} invalid!", attribute.value))),
                };

                if ctrl_type == CTRL_BANK_SELECT_MSB {
                    bank_select_msb = ctrl_value;
                } else if ctrl_type == CTRL_BANK_SELECT_LSB {
                    bank_select_lsb = ctrl_value;
                } else {
                    controller_events.push(MidiEvent {
                        tick: 0,
                        data: vec![CONTROLLER | channel_number, ctrl_type, ctrl_value],
                    });
                }
            }

            controller_events.push(MidiEvent {
                tick: 0,
                data: vec![CONTROLLER | channel_number, CTRL_BANK_SELECT_MSB, bank_select_msb],
            });

            controller_events.push(MidiEvent {
                tick: 0,
                data: vec![CONTROLLER | channel_number, CTRL_BANK_SELECT_LSB, bank_select_lsb],
            });

            program_change_events.push(MidiEvent {
                tick: 0,
                data: vec![PROGRAM_CHANGE | channel_number, channel.program],
            });

            // Notes events
            let harmonics_sequence_events = sequence::create_events(&binder, &seq)?;

            for harmonic_sequence_events in harmonics_sequence_events {
                for seq_ev in harmonic_sequence_events {
                    let note_number = freq_to_midi(seq_ev.start_frequency);
                    let start_velocity = (seq_ev.start_velocity * 127.) as u8;

                    let note_on_ev = MidiEvent {
                        tick: seq_ev.start_tick,
                        data: vec![NOTE_ON | channel_number, note_number, start_velocity],
                    };
                    events.push(note_on_ev);

                    let end_velocity = (seq_ev.end_velocity * 127.) as u8;

                    let note_off_ev = MidiEvent {
                        tick: seq_ev.end_tick,
                        data: vec![NOTE_OFF | channel_number, note_number, end_velocity],
                    };
                    events.push(note_off_ev);
                }
            }
            channel_number += 1;
        }

        events.sort_unstable_by(|a, b| a.tick.cmp(&b.tick));

        Ok(MidiSeq {
            controller_events,
            program_change_events,
            events,
            midi_urid,
        })
    }

    fn make_midi_event(&self, tick: i64, len: usize, event_reminder: &mut EventReminder, voice_buf: MAtomBuf) -> Result<(), failure::Error> {
        let end_t = tick + len as i64;
        let ev_count = self.events.len();
        let mut ev_idx = event_reminder.index;

        voice_buf.clear();

        while ev_idx > 0 && self.events[ev_idx - 1].tick >= tick {
            ev_idx -= 1;
        }
        while ev_idx < ev_count && self.events[ev_idx].tick < tick {
            ev_idx += 1;
        }

        if ev_idx < ev_count {
            if !event_reminder.initialized {
                for ev in &self.controller_events {
                    voice_buf.push_midi_event::<3>(0, self.midi_urid, &ev.data)?;
                }
                for ev in &self.program_change_events {
                    voice_buf.push_midi_event::<2>(0, self.midi_urid, &ev.data)?;
                }
                event_reminder.initialized = true;
            }
            while ev_idx < ev_count {
                let ev = &self.events[ev_idx];

                if ev.tick < end_t {
                    let time_in_frames = ev.tick - tick;
                    voice_buf.push_midi_event::<3>(time_in_frames, self.midi_urid, &ev.data)?;
                    ev_idx += 1;
                } else {
                    break;
                }
            }

            event_reminder.index = ev_idx;
        }
        Ok(())
    }

    pub fn talk(&self, tick: i64, len: usize, event_reminder: &mut EventReminder, voice_buf: MAtomBuf) {
        match self.make_midi_event(tick, len, event_reminder, voice_buf) {
            Ok(()) => (),
            Err(e) => eprintln!("MidiSeq::talk failed : {:?}", e),
        }
    }
}

