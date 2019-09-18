use std::mem;
use atom::ToAtom;
use midi::atom::AtomMidiEvent;

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
#[repr(u8)]
pub enum Channel {
    Channel1 = 0,
    Channel2,
    Channel3,
    Channel4,
    Channel5,
    Channel6,
    Channel7,
    Channel8,
    Channel9,
    Channel10,
    Channel11,
    Channel12,
    Channel13,
    Channel14,
    Channel15,
    Channel16,
}

impl Channel {
    #[inline]
    pub(crate) fn from_raw(raw: u8) -> Channel {
        unsafe { mem::transmute::<u8, Channel>(raw & 0x0Fu8) }
    }

    #[inline]
    pub fn into_raw(self) -> u8 {
        (unsafe { mem::transmute::<Channel, u8>(self ) }) & 0x0Fu8
    }
}

// TODO: Use proper Key & Velocity Types

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum MidiMessage {
    NoteOff { channel: Channel, note: u8, velocity: u8 },
    NoteOn  { channel: Channel, note: u8, velocity: u8 },
    ProgramChange { channel: Channel, program: u8 },

    #[doc(hidden)]
    __NonExhaustive
}

impl MidiMessage {
    pub fn channel(&self) -> Option<Channel> {
        match self {
            MidiMessage::NoteOff {channel, ..} |
            MidiMessage::NoteOn {channel, ..} |
            MidiMessage::ProgramChange {channel, ..} => Some(*channel),
            _ => None
        }
    }

    pub fn raw_type(&self) -> u8 {
        (match self {
            MidiMessage::NoteOff {..} => ::lv2_sys::LV2_Midi_Message_Type_LV2_MIDI_MSG_NOTE_OFF,
            MidiMessage::NoteOn {..} => ::lv2_sys::LV2_Midi_Message_Type_LV2_MIDI_MSG_NOTE_ON,
            MidiMessage::ProgramChange {..} => ::lv2_sys::LV2_Midi_Message_Type_LV2_MIDI_MSG_PGM_CHANGE,
            _ => ::lv2_sys::LV2_Midi_Message_Type_LV2_MIDI_MSG_INVALID
        }) as u8
    }

    pub fn from_raw(raw: &[u8; 3]) -> MidiMessage {
        match unsafe { ::lv2_sys::lv2_midi_message_type(&raw[0]) } {
            ::lv2_sys::LV2_Midi_Message_Type_LV2_MIDI_MSG_NOTE_OFF => MidiMessage::NoteOff {
                channel: Channel::from_raw(raw[0]),
                note: raw[1],
                velocity: raw[2]
            },
            ::lv2_sys::LV2_Midi_Message_Type_LV2_MIDI_MSG_NOTE_ON => MidiMessage::NoteOn {
                channel: Channel::from_raw(raw[0]),
                note: raw[1],
                velocity: raw[2]
            },
            ::lv2_sys::LV2_Midi_Message_Type_LV2_MIDI_MSG_PGM_CHANGE => MidiMessage::ProgramChange {
                channel: Channel::from_raw(raw[0]),
                program: raw[1]
            },
            _ => MidiMessage::__NonExhaustive
        }
    }

    pub fn to_raw(&self) -> [u8; 3] {
        match self {
            MidiMessage::NoteOff { channel, note, velocity } => [
                (::lv2_sys::LV2_Midi_Message_Type_LV2_MIDI_MSG_NOTE_OFF as u8) | channel.into_raw(),
                *note,
                *velocity
            ],
            MidiMessage::NoteOn { channel, note, velocity } => [
                (::lv2_sys::LV2_Midi_Message_Type_LV2_MIDI_MSG_NOTE_ON as u8) | channel.into_raw(),
                *note,
                *velocity
            ],
            MidiMessage::ProgramChange { channel, program } => [
                (::lv2_sys::LV2_Midi_Message_Type_LV2_MIDI_MSG_PGM_CHANGE as u8) | channel.into_raw(),
                *program,
                0
            ],
            _ => [0;3]
        }
    }
}

impl ToAtom for MidiMessage {
    type AtomType = AtomMidiEvent;

    #[inline]
    fn to_atom(&self) -> <Self as ToAtom>::AtomType {
        AtomMidiEvent::new(self)
    }
}