#[macro_export]
macro_rules! LINE_COMMENT_KW {
    () => {
        ";;"
    };
}
#[macro_export]
macro_rules! MULTILINE_COMMENT_KW {
    () => {
        ";;;"
    };
}

#[macro_export]
macro_rules! DEF_KW {
    () => {
        ':'
    };
}
#[macro_export]
macro_rules! ATTRIBUTE_KW {
    () => {
        '?'
    };
}
#[macro_export]
macro_rules! ASSIGNMENT_KW {
    () => {
        '='
    };
}
#[macro_export]
macro_rules! COUPLING_KW {
    () => {
        '&'
    };
}
#[macro_export]
macro_rules! JOIN_KW {
    () => {
        '-'
    };
}
#[macro_export]
macro_rules! REF_KW {
    () => {
        '@'
    };
}
#[macro_export]
macro_rules! MUL_KW {
    () => {
        '*'
    };
}
#[macro_export]
macro_rules! ON_KW {
    () => {
        '/'
    };
}
#[macro_export]
macro_rules! PER_KW {
    () => {
        '%'
    };
}
#[macro_export]
macro_rules! OPEN_PARENT_KW {
    () => {
        '('
    };
}
#[macro_export]
macro_rules! CLOSE_PARENT_KW {
    () => {
        ')'
    };
}
#[macro_export]
macro_rules! BEAT_KW {
    () => {
        "beat"
    };
}
#[macro_export]
macro_rules! SCALE_KW {
    () => {
        "scale"
    };
}
#[macro_export]
macro_rules! ENVELOP_KW {
    () => {
        "envelope"
    };
}
#[macro_export]
macro_rules! ATTACK_KW {
    () => {
        "attack"
    };
}
#[macro_export]
macro_rules! INTERVAL_KW {
    () => {
        '!'
    };
}
#[macro_export]
macro_rules! CHORD_KW {
    () => {
        "chord"
    };
}
#[macro_export]
macro_rules! CHORDLINE_KW {
    () => {
        "chords"
    };
}
#[macro_export]
macro_rules! HITLINE_KW {
    () => {
        "hits"
    };
}
#[macro_export]
macro_rules! DURATIONLINE_KW {
    () => {
        "durations"
    };
}
#[macro_export]
macro_rules! SECOND_SYM_KW {
    () => {
        's'
    };
}

#[macro_export]
macro_rules! PITCHLINE_KW {
    () => {
        "pitchs"
    };
}
#[macro_export]
macro_rules! LINEAR_SHAPE_KW {
    () => {
        '='
    };
}
#[macro_export]
macro_rules! SIN_SHAPE_KW {
    () => {
        '~'
    };
}
#[macro_export]
macro_rules! EARLY_SHAPE_KW {
    () => {
        '<'
    };
}
#[macro_export]
macro_rules! LATE_SHAPE_KW {
    () => {
        '>'
    };
}
#[macro_export]
macro_rules! ROUND_SHAPE_KW {
    () => {
        '°'
    };
}
#[macro_export]
macro_rules! OPEN_BRACKET_KW {
    () => {
        '{'
    };
}
#[macro_export]
macro_rules! CLOSE_BRACKET_KW {
    () => {
        '}'
    };
}
#[macro_export]
macro_rules! PARAM_SEP_KW {
    () => {
        ','
    };
}

#[macro_export]
macro_rules! NOTE_SHIFT_KW {
    () => {
        ">>"
    };
}
#[macro_export]
macro_rules! BACK_NOTE_SHIFT_KW {
    () => {
        "<<"
    };
}
#[macro_export]
macro_rules! PITCH_TRANSPO_KW {
    () => {
        "=>"
    };
}
#[macro_export]
macro_rules! PITCH_INV_KW {
    () => {
        "x"
    };
}

#[macro_export]
macro_rules! VELOCITYLINE_KW {
    () => {
        "velocities"
    };
}
#[macro_export]
macro_rules! FADEIN_KW {
    () => {
        '/'
    };
}
#[macro_export]
macro_rules! FADEOUT_KW {
    () => {
        '\\'
    };
}
#[macro_export]
macro_rules! SEQUENCE_KW {
    () => {
        "seq"
    };
}
#[macro_export]
macro_rules! SEQUENCE_OUTPUT_KW {
    () => {
        "seqout"
    };
}

#[macro_export]
macro_rules! MIDI_OUTPUT_KW {
    () => {
        "midiout"
    };
}

#[macro_export]
macro_rules! RATIO_DESC {
    ($r:expr) => {
        concat!(" <", $r, ">[", ON_KW!(), "<den>]",)
    };
}

#[macro_export]
macro_rules! TIME_DESC {
    ($t:expr, $u:expr) => {
        concat!(RATIO_DESC!($t), "[", SECOND_SYM_KW!(), "](", $u, "|second)")
    };
}

#[macro_export]
macro_rules! VELOCITY_DESC {
    () => {
        concat!("[", FADEIN_KW!(), "][<envelop_id>", MUL_KW!(), "]<velocity_value>[", FADEOUT_KW!(), "][", LINEAR_SHAPE_KW!(), "|", SIN_SHAPE_KW!(), "|", EARLY_SHAPE_KW!(), "|", LATE_SHAPE_KW!(), "|", ROUND_SHAPE_KW!(), "]",)
    };
}

pub const SYNTAX_DESCRIPTION: &str = concat!(
    MULTILINE_COMMENT_KW!(), " Description\n",
    BEAT_KW!(), " <beat_id> ", DEF_KW!(), " <bpm>\n",
    CHORD_KW!(), " <chord_id> ", DEF_KW!(), RATIO_DESC!("ratio"), "[", JOIN_KW!(), TIME_DESC!("delay", "hit"), "[", JOIN_KW!(), VELOCITY_DESC!(), "]][...]\n",
    ATTACK_KW!(), " <attack_id> ", DEF_KW!(), TIME_DESC!("delay", "hit"), "[", JOIN_KW!(), VELOCITY_DESC!(), "][...]\n",
    CHORDLINE_KW!(), " <chords_id> ", DEF_KW!(), " <chord_id>[", JOIN_KW!(), "<attack_id>][...]\n",
    HITLINE_KW!(), " <hits_id> ", DEF_KW!(), TIME_DESC!("position", "beat"), "[", JOIN_KW!(), TIME_DESC!("duration", "beat"), "][...] ", PER_KW!(), TIME_DESC!("duration", "beat"), "\n",
    DURATIONLINE_KW!(), " <durations_id> ", DEF_KW!(), TIME_DESC!("duration", "beat"), "[...]\n",
    PITCHLINE_KW!(), " <pitchs_id> ", DEF_KW!(),
    " [", ATTRIBUTE_KW!(), SCALE_KW!(), ASSIGNMENT_KW!(), "<scale_id>]",
    " <pitch>|", REF_KW!(), "<pitchs_id>[", OPEN_BRACKET_KW!(),
    NOTE_SHIFT_KW !(), "[<num>]|",
    BACK_NOTE_SHIFT_KW!(), "[<num>]|<pitch> ",
    PITCH_TRANSPO_KW!(), " <pitch>|",
    PITCH_INV_KW!(),
    "[", PARAM_SEP_KW!(), "...]",    
    CLOSE_BRACKET_KW!(),
    "][", LINEAR_SHAPE_KW!(), "|", SIN_SHAPE_KW!(), "|", EARLY_SHAPE_KW!(), "|", LATE_SHAPE_KW!(), "|", ROUND_SHAPE_KW!(),
    "][", MUL_KW!(), "<num>][...]\n",
    VELOCITYLINE_KW!(), " <velocities_id> ", DEF_KW!(), " ", VELOCITY_DESC!(), "[...]\n",
    ENVELOP_KW!(),
    " <envelop_id> ",
    DEF_KW!(),
    " <duration>(s) ",
    LINEAR_SHAPE_KW!(), "|", SIN_SHAPE_KW!(), "|", EARLY_SHAPE_KW!(), "|", LATE_SHAPE_KW!(), "|", ROUND_SHAPE_KW!(), " <level>[...]\n",
    SEQUENCE_KW!(),
    " <seq_id> ",
    DEF_KW!(),
    " [", ATTRIBUTE_KW!(), BEAT_KW!(), ASSIGNMENT_KW!(), "<beat_id>|<bpm>] [", ATTRIBUTE_KW!(), ENVELOP_KW!(), ASSIGNMENT_KW!(), "<envelop_id>][",
    OPEN_PARENT_KW!(),
    "][",
    REF_KW!(),
    "<seq_id>|<hits_id>[",
    COUPLING_KW!(),
    "<durations_id>][",
    JOIN_KW!(),
    "<pitchs_id>[",
    COUPLING_KW!(),
    "<chords_id>][",
    JOIN_KW!(),
    "<velocities_id>]]][",
    MUL_KW!(), "<num>][...][",
    CLOSE_PARENT_KW!(),
    MUL_KW!(), "<num>]\n",
    SEQUENCE_OUTPUT_KW!(),
    " <sequence_output_id> ",
    DEF_KW!(),
    " ''\n",
    MIDI_OUTPUT_KW!(),
    " <midi_output_id> ",
    DEF_KW!(),
    REF_KW!(),
    "<seq_id>",
    JOIN_KW!(),
    "<program_number (instrument)>[",
    ATTRIBUTE_KW!(),
    "bank",
    ASSIGNMENT_KW!(),
    "<bank_number>[",
    ATTRIBUTE_KW!(),
    "vol|volume",
    ASSIGNMENT_KW!(),
    "<volume_value (0-127)>[",
    ATTRIBUTE_KW!(),
    "bal|balance",
    ASSIGNMENT_KW!(),
    "<balance_value (0-127)>[",
    ATTRIBUTE_KW!(),
    "pan",
    ASSIGNMENT_KW!(),
    "<pan_value (0-127)>]]]][...]\n",
    MULTILINE_COMMENT_KW!()
);
