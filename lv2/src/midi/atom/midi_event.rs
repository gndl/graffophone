use atom::{Atom, AtomType, RawAtomType};
use core::uri::UriBound;
use midi::MidiMessage;

#[repr(C)]
pub struct AtomMidiEvent {
    inner: Atom,
    raw_message: [u8; 3]
}

impl AtomMidiEvent {
    pub fn new(message: &MidiMessage) -> Self {
        Self {
            inner: unsafe { Atom::new(3) },
            raw_message: message.to_raw()
        }
    }

    pub fn message(&self) -> MidiMessage {
        MidiMessage::from_raw(&self.raw_message)
    }
}

unsafe impl UriBound for AtomMidiEvent {
    const URI: &'static [u8] = ::lv2_sys::LV2_MIDI__MidiEvent;
}

impl AtomType for AtomMidiEvent {}
unsafe impl RawAtomType for AtomMidiEvent {}
