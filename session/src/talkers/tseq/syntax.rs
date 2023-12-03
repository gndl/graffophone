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
macro_rules! ATTACK_KW {
    () => {
        "attack"
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
macro_rules! LINEAR_TRANSITION_KW {
    () => {
        '='
    };
}
#[macro_export]
macro_rules! SIN_TRANSITION_KW {
    () => {
        '~'
    };
}
#[macro_export]
macro_rules! EARLY_TRANSITION_KW {
    () => {
        '<'
    };
}
#[macro_export]
macro_rules! LATE_TRANSITION_KW {
    () => {
        '>'
    };
}
#[macro_export]
macro_rules! ROUND_TRANSITION_KW {
    () => {
        '°'
    };
}
#[macro_export]
macro_rules! VELOCITYLINE_KW {
    () => {
        "velocities"
    };
}
#[macro_export]
macro_rules! INTERVAL_KW {
    () => {
        '!'
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

pub const SYNTAX_DESCRIPTION: &str = concat!(
    MULTILINE_COMMENT_KW!(),
    " Description\n",
    BEAT_KW!(),
    " <beat_id> ",
    DEF_KW!(),
    " <bpm>\n",
    CHORD_KW!(),
    " <chord_id> ",
    DEF_KW!(),
    RATIO_DESC!("ratio"),
    "[",
    JOIN_KW!(),
    TIME_DESC!("delay", "hit"),
    "[",
    JOIN_KW!(),
    "[",
    FADEIN_KW!(),
    "]<velocity>[",
    FADEOUT_KW!(),
    "]]] [...]\n",
    ATTACK_KW!(),
    " <attack_id> ",
    DEF_KW!(),
    TIME_DESC!("delay", "hit"),
    "[",
    JOIN_KW!(),
    "[",
    FADEIN_KW!(),
    "]<velocity>[",
    FADEOUT_KW!(),
    "]] [...]\n",
    CHORDLINE_KW!(),
    " <chords_id> ",
    DEF_KW!(),
    " <chord_id>[",
    JOIN_KW!(),
    "<attack_id>] [...]\n",
    HITLINE_KW!(),
    " <hits_id> ",
    DEF_KW!(),
    TIME_DESC!("position", "beat"),
    "[",
    JOIN_KW!(),
    TIME_DESC!("duration", "beat"),
    "] [...] ",
    PER_KW!(),
    TIME_DESC!("duration", "beat"),
    "\n",
    DURATIONLINE_KW!(),
    " <durations_id> ",
    DEF_KW!(),
    TIME_DESC!("duration", "beat"),
    " [...]\n",
    PITCHLINE_KW!(),
    " <pitchs_id> ",
    DEF_KW!(),
    " <pitch> [...]\n",
    VELOCITYLINE_KW!(),
    " <velocities_id> ",
    DEF_KW!(),
    " [",
    FADEIN_KW!(),
    "]<velocity_value>[",
    FADEOUT_KW!(),
    "|",
    LINEAR_TRANSITION_KW!(),
    "|",
    SIN_TRANSITION_KW!(),
    "|",
    EARLY_TRANSITION_KW!(),
    "|",
    LATE_TRANSITION_KW!(),
    "|",
    ROUND_TRANSITION_KW!(),
    "] [...]\n",
    SEQUENCE_KW!(),
    " <seq_id> ",
    DEF_KW!(),
    " [",
    ON_KW!(),
    " <beat_id>] [",
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
    MUL_KW!(),
    " <num>] [...][",
    CLOSE_PARENT_KW!(),
    MUL_KW!(),
    " <num>]\n",
    SEQUENCE_OUTPUT_KW!(),
    " <sequence_output_id> ",
    DEF_KW!(),
    " ''\n",
    MULTILINE_COMMENT_KW!()
);
