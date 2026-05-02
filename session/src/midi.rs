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

pub const NOTE_DATA_SIZE: usize = 3;
pub const NOTE_OFF_DATA_SIZE: usize = 3;
pub const CONTROLLER_DATA_SIZE: usize = 7;
pub const SYSEX_DATA_SIZE: usize = 13;

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

#[derive(Debug, PartialEq)]
pub struct Event {
    pub tick: i64,
    pub data: Vec<u8>,
    pub sysex: Option<Vec<u8>>,
}

impl Event {
    pub fn controller(channel_number: u8, tick: i64, ctrl_type: u8, ctrl_value: u8) -> Event {
        Self {
            tick,
            data: vec![CONTROLLER | channel_number, ctrl_type, ctrl_value],
            sysex: None,
        }
    }
    pub fn select_msb(channel_number: u8, tick: i64, msb: u8) -> Event {
        Self::controller(channel_number, tick, CTRL_BANK_SELECT_MSB, msb)
    }
    pub fn select_lsb(channel_number: u8, tick: i64, lsb: u8) -> Event {
        Self::controller(channel_number, tick, CTRL_BANK_SELECT_LSB, lsb)
    }
    pub fn tuning_program(channel_number: u8, tick: i64) -> Event {
        Self {
            tick,
            data: vec![CONTROLLER | channel_number, 0x64, 0x03, 0x65, 0x00, 0x06, channel_number],
            sysex: None,
        }
    }
    pub fn tuning_bank(channel_number: u8, tick: i64) -> Event {
        Self {
            tick,
            data: vec![CONTROLLER | channel_number, 0x64, 0x04, 0x65, 0x00, 0x06, 0x00],
            sysex: None,
        }
    }
    pub fn tuning(channel_number: u8, tick: i64) -> Vec<Event> {
        let mut events = Vec::new();

        events.push(
            Self {
                tick,
                data: vec![CONTROLLER | channel_number, 0x64, 0x03],
                sysex: None,
            }
        );

        events.push(
            Self {
                tick,
                data: vec![CONTROLLER | channel_number, 0x65, 0x00],
                sysex: None,
            }
        );

        events.push(
            Self {
                tick,
                data: vec![CONTROLLER | channel_number, 0x06, channel_number],
                sysex: None,
            }
        );

        events.push(
            Self {
                tick,
                data: vec![CONTROLLER | channel_number, 0x64, 0x04],
                sysex: None,
            }
        );

        events.push(
            Self {
                tick,
                data: vec![CONTROLLER | channel_number, 0x65, 0x00],
                sysex: None,
            }
        );

        events.push(
            Self {
                tick,
                data: vec![CONTROLLER | channel_number, 0x06, 0x00],
                sysex: None,
            }
        );

        events
    }

    pub fn program_change(channel_number: u8, tick: i64, program: u8) -> Event {
        Self {
            tick,
            data: vec![PROGRAM_CHANGE | channel_number, program],
            sysex: None,
        }
    }
    pub fn note(
        channel_number: u8,
        frequency: f32,
        start_tick: i64,
        start_velocity: f32,
        end_tick: i64,
        end_velocity: f32,
        microtonal: bool,
    ) -> (Event, Event) {
        let code = 12. * (frequency as f64 / FREQ_0).log2();
        let note_number = (code.round() as u64).min(127) as u8;
            
        let sysex = if microtonal {
            let code_floor = code.trunc();
            let code_fract = code - code_floor;
            let mut note_number_below = (code_floor as u64).min(127) as u8;
            let fract = (code_fract * 16384.) as u64;
            let mut lsb = (fract & 0x7F) as u8;
            let mut msb = ((fract >> 7) & 0x7f) as u8;

            if msb == 127 && lsb == 127 && note_number_below < 127 {
                note_number_below += 1;
                msb = 0;
                lsb = 0;
            }

            Some(vec![0xF0, 0x7E, 0x7F, 0x08, 0x07, 0x00, 0x00, channel_number, 0x01, note_number, note_number_below, msb, lsb, 0xF7])
        }
        else {
            None
        };

        let start_vel = (start_velocity.abs() * 127.) as u8;
        let end_vel = (end_velocity.abs() * 127.) as u8;

        (Self {
            tick: start_tick,
            data: vec![NOTE_ON | channel_number, note_number, start_vel],
            sysex,
        },
        Self {
            tick: end_tick,
            data: vec![NOTE_OFF | channel_number, note_number, end_vel],
            sysex: None,
        })
    }

}
#[test]
fn test_event_note() {
    use scale::scale::{self, Scale};

    let scl_12et = scale::create_12et_scale();

    assert_eq!(Event::note(0, 8.1758, 1, 0.5, 2, 0.4, true),
        (Event {
            tick: 1,
            data: vec![NOTE_ON, 0, 63],
            sysex: Some(vec![0xF0, 0x7E, 0x7F, 0x08, 0x07, 0x00, 0x01, 0, 0, 0, 0, 0xF7]),
        },
        Event {
            tick: 2,
            data: vec![NOTE_OFF, 0, 50],
            sysex: None,
        })
    );

    assert_eq!(Event::note(0, 8.17585, 1, 1., 2, 0., true),
        (Event {
            tick: 1,
            data: vec![NOTE_ON, 0, 127],
            sysex: Some(vec![0xF0, 0x7F, 0x7F, 0x08, 0x02, 0x00, 0x01, 0, 0, 0, 01, 0xF7]),
        },
        Event {
            tick: 2,
            data: vec![NOTE_OFF, 0, 0],
            sysex: None,
        })
    );

    assert_eq!(Event::note(0, 8.66196, 1, 1., 2, 0., true),
        (Event {
            tick: 1,
            data: vec![NOTE_ON, 1, 127],
            sysex: Some(vec![0xF0, 0x7F, 0x7F, 0x08, 0x02, 0x00, 0x01, 1, 1, 0, 0, 0xF7]),
        },
        Event {
            tick: 2,
            data: vec![NOTE_OFF, 1, 0],
            sysex: None,
        })
    );

    assert_eq!(Event::note(0, scl_12et.pitch_name_to_frequency("C0").unwrap(), 1, 1., 2, 0., true),
        (Event {
            tick: 1,
            data: vec![NOTE_ON, 0x0C, 127],
            sysex: Some(vec![0xF0, 0x7F, 0x7F, 0x08, 0x02, 0x00, 0x01, 0x0C, 0x0C, 0, 0, 0xF7]),
        },
        Event {
            tick: 2,
            data: vec![NOTE_OFF, 0x0C, 0],
            sysex: None,
        })
    );

    assert_eq!(Event::note(0, scl_12et.pitch_name_to_frequency("C4").unwrap(), 1, 1., 2, 0., true),
        (Event {
            tick: 1,
            data: vec![NOTE_ON, 0x3C, 127],
            sysex: Some(vec![0xF0, 0x7F, 0x7F, 0x08, 0x02, 0x00, 0x01, 0x3C, 0x3C, 0, 0, 0xF7]),
        },
        Event {
            tick: 2,
            data: vec![NOTE_OFF, 0x3C, 0],
            sysex: None,
        })
    );

    assert_eq!(Event::note(0, scl_12et.pitch_name_to_frequency("C#4").unwrap(), 1, 1., 2, 0., true),
        (Event {
            tick: 1,
            data: vec![NOTE_ON, 0x3D, 127],
            sysex: Some(vec![0xF0, 0x7F, 0x7F, 0x08, 0x02, 0x00, 0x01, 0x3D, 0x3D, 0, 0, 0xF7]),
        },
        Event {
            tick: 2,
            data: vec![NOTE_OFF, 0x3D, 0],
            sysex: None,
        })
    );

    assert_eq!(Event::note(0, 439.9984, 1, 0.5, 2, 0.4, true),
        (Event {
            tick: 1,
            data: vec![NOTE_ON, 0x45, 63],
            sysex: Some(vec![0xF0, 0x7F, 0x7F, 0x08, 0x02, 0x00, 0x01, 0x45, 0x44, 0x7F, 0x7E, 0xF7]),
        },
        Event {
            tick: 2,
            data: vec![NOTE_OFF, 0x45, 50],
            sysex: None,
        })
    );

    assert_eq!(Event::note(0, scl_12et.pitch_name_to_frequency("A4").unwrap(), 1, 1., 2, 0., true),
        (Event {
            tick: 1,
            data: vec![NOTE_ON, 0x45, 127],
            sysex: Some(vec![0xF0, 0x7F, 0x7F, 0x08, 0x02, 0x00, 0x01, 0x45, 0x45, 0, 0, 0xF7]),
        },
        Event {
            tick: 2,
            data: vec![NOTE_OFF, 0x45, 0],
            sysex: None,
        })
    );

    assert_eq!(Event::note(0, 440.0016, 1, 0.5, 2, 0.4, true),
        (Event {
            tick: 1,
            data: vec![NOTE_ON, 0x45, 63],
            sysex: Some(vec![0xF0, 0x7F, 0x7F, 0x08, 0x02, 0x00, 0x01, 0x45, 0x45, 0x00, 0x01, 0xF7]),
        },
        Event {
            tick: 2,
            data: vec![NOTE_OFF, 0x45, 50],
            sysex: None,
        })
    );

    assert_eq!(Event::note(0, scl_12et.pitch_name_to_frequency("C9").unwrap(), 1, 1., 2, 0., true),
        (Event {
            tick: 1,
            data: vec![NOTE_ON, 0x78, 127],
            sysex: Some(vec![0xF0, 0x7F, 0x7F, 0x08, 0x02, 0x00, 0x01, 0x78, 0x78, 0, 0, 0xF7]),
        },
        Event {
            tick: 2,
            data: vec![NOTE_OFF, 0x78, 0],
            sysex: None,
        })
    );

    assert_eq!(Event::note(0, 8372.0630, 1, 1., 2, 0., true),
        (Event {
            tick: 1,
            data: vec![NOTE_ON, 0x78, 127],
            sysex: Some(vec![0xF0, 0x7F, 0x7F, 0x08, 0x02, 0x00, 0x01, 0x78, 0x78, 0, 1, 0xF7]),
        },
        Event {
            tick: 2,
            data: vec![NOTE_OFF, 0x78, 0],
            sysex: None,
        })
    );

    assert_eq!(Event::note(0, scl_12et.pitch_name_to_frequency("G9").unwrap(), 1, 1., 2, 0., true),
        (Event {
            tick: 1,
            data: vec![NOTE_ON, 0x7F, 127],
            sysex: Some(vec![0xF0, 0x7F, 0x7F, 0x08, 0x02, 0x00, 0x01, 0x7F, 0x7F, 0, 0, 0xF7]),
        },
        Event {
            tick: 2,
            data: vec![NOTE_OFF, 0x7F, 0],
            sysex: None,
        })
    );

    assert_eq!(Event::note(0, 13289.7, 1, 0.5, 2, 0.4, true),
        (Event {
            tick: 1,
            data: vec![NOTE_ON, 0x7F, 63],
            sysex: Some(vec![0xF0, 0x7F, 0x7F, 0x08, 0x02, 0x00, 0x01, 0x7F, 0x7F, 0x7F, 0x7E, 0xF7]),
        },
        Event {
            tick: 2,
            data: vec![NOTE_OFF, 0x7F, 50],
            sysex: None,
        })
    );
}
