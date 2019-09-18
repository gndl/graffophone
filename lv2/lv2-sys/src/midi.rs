use super::*;

#[inline]
pub unsafe fn lv2_midi_is_voice_message(msg: *const u8) -> bool {
    *msg >= 0x80 && *msg < 0xF0
}

#[inline]
pub unsafe fn lv2_midi_is_system_message(msg: *const u8) -> bool {
    match *msg {
        0xF4 | 0xF5 | 0xF7 | 0xF9 | 0xFD => false,
        _ => (*msg & 0xF0u8) == 0xF0,
    }
}

#[inline]
pub unsafe fn lv2_midi_message_type(msg: *const u8) -> LV2_Midi_Message_Type {
    if lv2_midi_is_voice_message(msg) {
        (*msg & 0xF0u8) as u32
    } else if lv2_midi_is_system_message(msg) {
        *msg as u32
    } else {
        LV2_Midi_Message_Type_LV2_MIDI_MSG_INVALID
    }
}

