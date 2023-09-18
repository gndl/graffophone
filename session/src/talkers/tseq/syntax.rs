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
macro_rules! BEAT_KW {
    () => {
        "beat"
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
    " <num>[",
    ON_KW!(),
    "<den>][",
    JOIN_KW!(),
    "<delay>[",
    JOIN_KW!(),
    "<velocity>]] [...]\n",
    CHORDLINE_KW!(),
    " <chords_id> ",
    DEF_KW!(),
    " <chord_id> [...]\n",
    HITLINE_KW!(),
    " <hits_id> ",
    DEF_KW!(),
    " <position (beat)>[",
    JOIN_KW!(),
    "<duration (sec)>] [...] ",
    ON_KW!(),
    " <duration (beat)>\n",
    DURATIONLINE_KW!(),
    " <durations_id> ",
    DEF_KW!(),
    " <duration (sec)>] [...]\n",
    PITCHLINE_KW!(),
    " <pitchs_id> ",
    DEF_KW!(),
    " <pitch> [...]\n",
    VELOCITYLINE_KW!(),
    " <velocities_id> ",
    DEF_KW!(),
    " <velocity_value> [",
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
    " <num>] [...]\n",
    SEQUENCE_OUTPUT_KW!(),
    " <sequence_output_id> ",
    DEF_KW!(),
    " ''\n",
    MULTILINE_COMMENT_KW!()
);
