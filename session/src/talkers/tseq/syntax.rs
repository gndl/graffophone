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
        "_/"
    };
}
#[macro_export]
macro_rules! FADEOUT_KW {
    () => {
        "\\_"
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
        concat!("[", FADEIN_KW!(), "]<velocity_value>[", COUPLING_KW!(), "<envelop_id>][", FADEOUT_KW!(), "][", LINEAR_SHAPE_KW!(), "|", SIN_SHAPE_KW!(), "|", EARLY_SHAPE_KW!(), "|", LATE_SHAPE_KW!(), "|", ROUND_SHAPE_KW!(), "]",)
    };
}

pub const SYNTAX_DESCRIPTION: &str = concat!(
    MULTILINE_COMMENT_KW!(), " Description\n",
    BEAT_KW!(), " <beat_id> ", DEF_KW!(), " <bpm>\n",
    SCALE_KW!(), " <scale_alias> ", DEF_KW!(), " <scale_name (SCL_12ET|SCL_17ET|SCL_19ET|SCL_53ET|SCL_natural|SCL_pythagorean)>\n",
    CHORD_KW!(), " <chord_id> ", DEF_KW!(), RATIO_DESC!("ratio"), "[", JOIN_KW!(), TIME_DESC!("delay", "hit"), "[", JOIN_KW!(), VELOCITY_DESC!(), "]][...]\n",
    ATTACK_KW!(), " <attack_id> ", DEF_KW!(), TIME_DESC!("delay", "hit"), "[", JOIN_KW!(), VELOCITY_DESC!(), "][...]\n",
    CHORDLINE_KW!(), " <chords_id> ", DEF_KW!(),
    "[", OPEN_PARENT_KW!(), "]",
    " <chord_id>[", JOIN_KW!(), "<attack_id>]",
    "[", MUL_KW!(), "<num>][...][",
    CLOSE_PARENT_KW!(),
    MUL_KW!(), "<num>][...]\n",
    HITLINE_KW!(), " <hits_id> ", DEF_KW!(), TIME_DESC!("position", "beat"), "[", JOIN_KW!(), TIME_DESC!("duration", "beat"), "][...] ", PER_KW!(), TIME_DESC!("duration", "beat"), "\n",
    DURATIONLINE_KW!(), " <durations_id> ", DEF_KW!(), TIME_DESC!("duration", "beat"), "[...]\n",
    PITCHLINE_KW!(), " <pitchs_id> ", DEF_KW!(),
    " [", ATTRIBUTE_KW!(), SCALE_KW!(), ASSIGNMENT_KW!(), "<scale_alias>|<scale_name>] [",
    OPEN_PARENT_KW!(), "]",
    "<pitch>|", REF_KW!(), "<pitchs_id>[", OPEN_BRACKET_KW!(),
    NOTE_SHIFT_KW !(), "[<num>]|",
    BACK_NOTE_SHIFT_KW!(), "[<num>]|<pitch> ",
    PITCH_TRANSPO_KW!(), " <pitch>|",
    PITCH_INV_KW!(),
    "[", PARAM_SEP_KW!(), "...]",    
    CLOSE_BRACKET_KW!(),
    "][", LINEAR_SHAPE_KW!(), "|", SIN_SHAPE_KW!(), "|", EARLY_SHAPE_KW!(), "|", LATE_SHAPE_KW!(), "|", ROUND_SHAPE_KW!(),
    "][", MUL_KW!(), "<num>][...][",
    CLOSE_PARENT_KW!(),
    MUL_KW!(), "<num>][...]\n",
    VELOCITYLINE_KW!(), " <velocities_id> ", DEF_KW!(),
    "[", OPEN_PARENT_KW!(), "]",
    VELOCITY_DESC!(),
    "[", MUL_KW!(), "<num>][...][",
    CLOSE_PARENT_KW!(),
    MUL_KW!(), "<num>][...]\n",
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
    MUL_KW!(), "<num>][...]\n",
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


pub const TSEQ_LANGUAGE_ID: &str = "tseq";

pub const TSEQ_LANGUAGE_DEFINITION: &str = concat!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>
<language id=\"tseq\" name=\"tseq\" version=\"2.0\" _section=\"Source\">
  <metadata>
    <property name=\"mimetypes\">text/x-c</property>
    <property name=\"globs\">*.tsq</property>
  </metadata>
  <styles>
    <style id=\"comment\" name=\"Comment\" map-to=\"def:comment\"/>
    <style id=\"floating-point\" name=\"Floating point number\" map-to=\"def:floating-point\"/>
    <style id=\"identifier\" name=\"Identifier\" map-to=\"def:identifier\"/>
    <style id=\"keyword\" name=\"Keyword\" map-to=\"def:keyword\"/>
    <style id=\"unit\" name=\"Unit\" map-to=\"def:keyword\"/>
  </styles>
  <definitions>
    <context id=\"comment-multiline\" style-ref=\"comment\">
      <start>", MULTILINE_COMMENT_KW!(), "</start>
      <end>", MULTILINE_COMMENT_KW!(), "</end>
    </context>
    <context id=\"comment\" style-ref=\"comment\">
      <start>", LINE_COMMENT_KW!(), "</start>
      <end>$</end>
    </context>
    <context id=\"float\">
      <match extended=\"true\" case-sensitive=\"false\">[^\\#\\w](\\d*\\.?\\d+)</match>
      <include>
        <context sub-pattern=\"1\" style-ref=\"floating-point\"/>
      </include>
    </context>
    <context id=\"unit\">
      <match extended=\"true\" case-sensitive=\"false\">[0-9\\s](ms|s|m)\\W</match>
      <include>
        <context sub-pattern=\"1\" style-ref=\"unit\"/>
      </include>
    </context>
    <context id=\"identifier\">
      <match extended=\"true\" case-sensitive=\"false\">\\W([a-zA-Z_\\^][\\^\\w\\#]*)</match>
      <include>
        <context sub-pattern=\"1\" style-ref=\"identifier\"/>
      </include>
    </context>
    <context id=\"keywords\" style-ref=\"keyword\">
      <keyword>", BEAT_KW!(), "</keyword>
      <keyword>", SCALE_KW!(), "</keyword>
      <keyword>", ENVELOP_KW!(), "</keyword>
      <keyword>", ATTACK_KW!(), "</keyword>
      <keyword>", CHORD_KW!(), "</keyword>
      <keyword>", CHORDLINE_KW!(), "</keyword>
      <keyword>", HITLINE_KW!(), "</keyword>
      <keyword>", DURATIONLINE_KW!(), "</keyword>
      <keyword>", PITCHLINE_KW!(), "</keyword>
      <keyword>", VELOCITYLINE_KW!(), "</keyword>
      <keyword>", SEQUENCE_KW!(), "</keyword>
      <keyword>", SEQUENCE_OUTPUT_KW!(), "</keyword>
      <keyword>", MIDI_OUTPUT_KW!(), "</keyword>
    </context>
    <context id=\"tseq\" class=\"no-spell-check\">
      <include>
        <context ref=\"comment-multiline\"/>
        <context ref=\"comment\"/>
        <context ref=\"float\"/>
        <context ref=\"unit\"/>
        <context ref=\"keywords\"/>
        <context ref=\"identifier\"/>
      </include>
    </context>
  </definitions>
</language>
");
