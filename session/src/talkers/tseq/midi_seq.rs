use std::str::FromStr;

use talker::horn::MAtomBuf;

use talkers::tseq::audio_event::AudioEvents;
use talkers::tseq::binder::Binder;
use talkers::tseq::sequence::{self, EventReminder};
use talkers::tseq::parser::PMidiSequence;
use midi;

pub struct MidiSeq {
    controller_events: Vec<midi::Event>,
    events: Vec<midi::Event>,
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

        let mut controller_events = Vec::with_capacity(64);
        let mut events = Vec::with_capacity(1024);
        let mut channel_number: u8 = 0;

        for channel in &sequence.channels {
            let seq = binder.fetch_sequence(&channel.seq_id)?;

            // Channel configuration events
            if let Some(bank_msb) = channel.bank_msb {
                let msb = u8::from_str(bank_msb)?;

                controller_events.push(midi::Event::select_msb(channel_number, 0, msb));
            }

            if let Some(bank_lsb) = channel.bank_lsb {
                if !bank_lsb.is_empty() {
                    let lsb = u8::from_str(bank_lsb)?;

                    controller_events.push(midi::Event::select_lsb(channel_number, 0, lsb));
                }
            }

            if let Some(program) = channel.program {
                let prog = u8::from_str(program)?;

                controller_events.push(midi::Event::program_change(channel_number, 0, prog));
            }

            for attribute in &channel.attributes {
                let ctrl_type = if attribute.label.starts_with("vol") {
                    midi::CTRL_VOLUME
                } else if attribute.label == "bal" {
                    midi::CTRL_BALANCE
                } else if attribute.label.starts_with("pan") {
                    midi::CTRL_PAN
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

                controller_events.push(midi::Event::controller(channel_number, 0, ctrl_type, ctrl_value));
            }

            // Notes events
            let harmonics_sequence_events = sequence::create_events(&binder, &seq)?;
            
            let mut microtonal_channel = false;

            for harmonic_sequence_events in harmonics_sequence_events {
                for seq_ev in harmonic_sequence_events {
                    let (note_on_ev, note_off_ev) = midi::Event::note(
                        channel_number,
                        seq_ev.start_frequency,
                        seq_ev.start_tick,
                        seq_ev.start_velocity,
                        seq_ev.end_tick,
                        seq_ev.end_velocity,
                        seq_ev.microtonal,
                    );
                    events.push(note_on_ev);
                    events.push(note_off_ev);

                    microtonal_channel |= seq_ev.microtonal;
                }
            }

            if microtonal_channel {
                // controller_events.push(midi::Event::tuning_program(channel_number, 0));
                // controller_events.push(midi::Event::tuning_bank(channel_number, 0));
                controller_events.append(&mut midi::Event::tuning(channel_number, 0));
            }

            channel_number += 1;
        }

        events.sort_unstable_by(|a, b| a.tick.cmp(&b.tick));

        Ok(MidiSeq {
            controller_events,
            events,
            midi_urid,
        })
    }

    pub fn from_audio_events(
        audio_events: &AudioEvents,
        midi_urid: lv2_raw::LV2Urid,
    ) -> Result<MidiSeq, failure::Error> {
        let mut events = Vec::with_capacity(1024);

        // Notes events
        for audio_ev in audio_events {
            let (note_on_ev, note_off_ev) = midi::Event::note(
                0,
                audio_ev.frequency(),
                audio_ev.start_tick(),
                1.,
                audio_ev.end_tick(),
                1.,
                false
            );
            events.push(note_on_ev);
            events.push(note_off_ev);
        }

        events.sort_unstable_by(|a, b| a.tick.cmp(&b.tick));

        Ok(MidiSeq {
            controller_events: Vec::new(),
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
                    voice_buf.push_midi_event::<{ midi::CONTROLLER_DATA_SIZE }>(0, self.midi_urid, &ev.data)?;
                }
                event_reminder.initialized = true;
            }
            while ev_idx < ev_count {
                let ev = &self.events[ev_idx];

                if ev.tick < end_t {
                    let time_in_frames = ev.tick - tick;

                    voice_buf.push_midi_event::<{ midi::NOTE_DATA_SIZE }>(time_in_frames, self.midi_urid, &ev.data)?;
                    
                    if let Some(sysex_data) = &ev.sysex {
                        voice_buf.push_midi_event::<{ midi::SYSEX_DATA_SIZE }>(time_in_frames, self.midi_urid, sysex_data)?;
                    }

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

