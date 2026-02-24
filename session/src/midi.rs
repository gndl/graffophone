pub const FREQ_0: f64 = 8.175799;
pub const NOTE_OFF: u8 = 0x80;
pub const NOTE_ON: u8 = 0x90;
pub const CONTROLLER: u8 = 0xB0;
pub const CTRL_BANK_SELECT_MSB: u8 = 0x00;
pub const CTRL_BANK_SELECT_LSB: u8 = 0x20;
pub const CTRL_VOLUME: u8 = 0x07;
pub const CTRL_BALANCE: u8 = 0x08;
pub const CTRL_PAN: u8 = 0x0A;
pub const PROGRAM_CHANGE: u8 = 0xC0;

pub fn to_freq(code: u8) -> f32 {
    (FREQ_0 * (code as f64 / 12.).exp2()) as f32
}
#[test]
fn test_to_freq() {
    assert_eq!(to_freq(0), 8.175799);
    assert_eq!(to_freq(69), 440.);
}

pub fn from_freq(freq: f32) -> u8 {
    let code = 12. * (freq as f64 / FREQ_0).log2();
    (code.round() as u8) & 0x7F
}
#[test]
fn test_from_freq() {
    assert_eq!(from_freq(440.), 69);
}

pub fn from_freq_pitch_bend(freq: f32) -> (u8, Option<u16>) {
    let code = 12. * (freq as f64 / FREQ_0).log2();
    let icode = code.round();
    let pitch_bend = (((code - icode) * 4096.) as i64 + 8192) as u16;
    let opb = if pitch_bend == 8192 { None} else {Some(pitch_bend)};
    (icode as u8, opb)
}
#[test]
fn test_from_freq_pitch_bend() {
    assert_eq!(from_freq_pitch_bend(440.), (69, None));
}

pub fn event_is_note_on(data: &[u8]) -> bool {
    (data[0] & 0xF0) == NOTE_ON
}
