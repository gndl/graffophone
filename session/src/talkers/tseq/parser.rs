use nom::{
    branch::alt,
    bytes::complete::{tag, take_until},
    character::complete::{alphanumeric1, char, digit1, newline, one_of, space0, space1},
    combinator::{opt, recognize},
    multi::{many0, many1_count},
    number::complete::float,
    sequence::{delimited, preceded, terminated, tuple},
    IResult,
};

use std::str::FromStr;

use ASSIGNMENT_KW;
use BEAT_KW;
use COUPLING_KW;
use DEF_KW;
use DURATIONLINE_KW;
use EARLY_TRANSITION_KW;
use HITLINE_KW;
use JOIN_KW;
use LATE_TRANSITION_KW;
use LINEAR_TRANSITION_KW;
use LINE_COMMENT_KW;
use MIDI_OUTPUT_KW;
use MULTILINE_COMMENT_KW;
use MUL_KW;
use ON_KW;
use PER_KW;
use PITCHLINE_KW;
use REF_KW;
use ROUND_TRANSITION_KW;
use SECOND_SYM_KW;
use SEQUENCE_KW;
use SEQUENCE_OUTPUT_KW;
use SIN_TRANSITION_KW;
use VELOCITYLINE_KW;
use {ATTACK_KW, CHORDLINE_KW, CHORD_KW, INTERVAL_KW};
use {CLOSE_PARENT_KW, OPEN_PARENT_KW};
use {FADEIN_KW, FADEOUT_KW};

#[derive(Debug, PartialEq)]
pub struct PRatio {
    pub num: f32,
    pub den: f32,
}

#[derive(Debug, PartialEq)]
pub enum PTime {
    Rate(PRatio),
    Second(PRatio),
}
#[derive(Debug, PartialEq)]
pub struct PProperty<'a> {
    pub label: &'a str,
    pub value: &'a str,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum PTransition {
    None,
    Linear,
    Sin,
    Early,
    Late,
    Round,
}

#[derive(Debug, PartialEq)]
pub struct PBeat<'a> {
    pub id: &'a str,
    pub bpm: usize,
}

#[derive(Debug, PartialEq)]
pub enum PPitchGap {
    FreqRatio(PRatio),
    Interval(i32),
}

#[derive(Debug, PartialEq)]
pub struct PHarmonic {
    pub pitch_gap: PPitchGap,
    pub delay: Option<PTime>,
    pub velocity: Option<PVelocity>,
}

#[derive(Debug, PartialEq)]
pub struct PChord<'a> {
    pub id: &'a str,
    pub harmonics: Vec<PHarmonic>,
}

#[derive(Debug, PartialEq)]
pub struct PAccent {
    pub delay: PTime,
    pub velocity: Option<PVelocity>,
}

#[derive(Debug, PartialEq)]
pub struct PAttack<'a> {
    pub id: &'a str,
    pub accents: Vec<PAccent>,
}

#[derive(Debug, PartialEq)]
pub struct PChordAndAttack<'a> {
    pub chord_id: &'a str,
    pub attack_id: Option<&'a str>,
}

#[derive(Debug, PartialEq)]
pub struct PChordLine<'a> {
    pub id: &'a str,
    pub chords: Vec<PChordAndAttack<'a>>,
}

#[derive(Debug, PartialEq)]
pub struct PPitch<'a> {
    pub id: &'a str,
    pub transition: PTransition,
}

#[derive(Debug, PartialEq)]
pub struct PPitchLine<'a> {
    pub id: &'a str,
    pub pitchs: Vec<PPitch<'a>>,
}

#[derive(Debug, PartialEq)]
pub struct PHit {
    pub position: PTime,
    pub duration: Option<PTime>,
}

#[derive(Debug, PartialEq)]
pub struct PHitLine<'a> {
    pub id: &'a str,
    pub hits: Vec<PHit>,
    pub duration: PTime,
}

#[derive(Debug, PartialEq)]
pub struct PDurationLine<'a> {
    pub id: &'a str,
    pub durations: Vec<PTime>,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct PVelocity {
    pub value: f32,
    pub fadein: bool,
    pub fadeout: bool,
    pub transition: PTransition,
}

#[derive(Debug, PartialEq)]
pub struct PVelocityLine<'a> {
    pub id: &'a str,
    pub velocities: Vec<PVelocity>,
}

#[derive(Debug, PartialEq)]
pub struct PPart<'a> {
    pub hitline_id: &'a str,
    pub durationline_id: Option<&'a str>,
    pub pitchline_id: Option<&'a str>,
    pub chordline_id: Option<&'a str>,
    pub velocityline_id: Option<&'a str>,
    pub mul: Option<f32>,
}

#[derive(Debug, PartialEq)]
pub struct PSeqRef<'a> {
    pub id: &'a str,
    pub mul: Option<usize>,
}

#[derive(Debug, PartialEq)]
pub enum PFragment<'a> {
    Part(PPart<'a>),
    SeqRef(PSeqRef<'a>),
    Fragments((Vec<PFragment<'a>>, usize)),
}

#[derive(Debug, PartialEq)]
pub struct PSequence<'a> {
    pub id: &'a str,
    pub beat: Option<&'a str>,
    pub fragments: Vec<PFragment<'a>>,
}

#[derive(Debug, PartialEq)]
pub enum Expression<'a> {
    Beat(PBeat<'a>),
    Chord(PChord<'a>),
    Attack(PAttack<'a>),
    ChordLine(PChordLine<'a>),
    DurationLine(PDurationLine<'a>),
    VelocityLine(PVelocityLine<'a>),
    HitLine(PHitLine<'a>),
    PitchLine(PPitchLine<'a>),
    Seq(PSequence<'a>),
    SeqOut(PSequence<'a>),
    MidiOut(PSequence<'a>),
    None,
}

pub fn id(input: &str) -> IResult<&str, &str> {
    recognize(many1_count(alt((alphanumeric1, tag("_")))))(input)
}

fn head<'a>(inst: &'a str) -> impl FnMut(&'a str) -> IResult<&'a str, &'a str> {
    delimited(
        preceded(tag(inst), space1),
        id,
        delimited(space0, char(DEF_KW!()), space0),
    )
}

fn on(input: &str) -> IResult<&str, char> {
    delimited(space0, char(ON_KW!()), space0)(input)
}

fn per(input: &str) -> IResult<&str, char> {
    delimited(space0, char(PER_KW!()), space0)(input)
}

fn end(input: &str) -> IResult<&str, Expression> {
    let (input, _) = many1_count(preceded(space0, newline))(input)?;
    Ok((input, Expression::None))
}

fn line_comment(input: &str) -> IResult<&str, Expression> {
    let (input, _) = delimited(tag(LINE_COMMENT_KW!()), take_until("\n"), end)(input)?;
    Ok((input, Expression::None))
}
fn multiline_comment(input: &str) -> IResult<&str, Expression> {
    let (input, _) = delimited(
        tag(MULTILINE_COMMENT_KW!()),
        take_until(MULTILINE_COMMENT_KW!()),
        tag(MULTILINE_COMMENT_KW!()),
    )(input)?;
    Ok((input, Expression::None))
}

fn beat(input: &str) -> IResult<&str, Expression> {
    let (input, (id, bpm, _)) = tuple((head(BEAT_KW!()), digit1, end))(input)?;
    Ok((
        input,
        Expression::Beat(PBeat {
            id,
            bpm: usize::from_str(bpm).unwrap(),
        }),
    ))
}

fn property(input: &str) -> IResult<&str, PProperty> {
    let (input, (label, _, _, _, value, _)) =
        tuple((id, space0, char(ASSIGNMENT_KW!()), space0, id, space0))(input)?;
    Ok((input, PProperty { label, value }))
}

fn ratio(input: &str) -> IResult<&str, PRatio> {
    let (input, (num, den, _)) = tuple((float, opt(preceded(on, float)), space0))(input)?;
    Ok((
        input,
        PRatio {
            num,
            den: den.unwrap_or(1.),
        },
    ))
}

fn freq_ratio(input: &str) -> IResult<&str, PPitchGap> {
    let (input, freq_ratio) = ratio(input)?;
    Ok((input, PPitchGap::FreqRatio(freq_ratio)))
}

fn interval(input: &str) -> IResult<&str, PPitchGap> {
    let (input, interval) = delimited(char(INTERVAL_KW!()), digit1, space0)(input)?;
    Ok((input, PPitchGap::Interval(i32::from_str(interval).unwrap())))
}

fn time(input: &str) -> IResult<&str, PTime> {
    let (input, (value, ounit, _)) = tuple((ratio, opt(char(SECOND_SYM_KW!())), space0))(input)?;
    let duration = if ounit.is_some() {
        PTime::Second(value)
    } else {
        PTime::Rate(value)
    };
    Ok((input, duration))
}

fn harmonic(input: &str) -> IResult<&str, PHarmonic> {
    let (input, (pitch_gap, delay, velocity, _)) = tuple((
        alt((freq_ratio, interval)),
        opt(preceded(terminated(char(JOIN_KW!()), space0), time)),
        opt(preceded(terminated(char(JOIN_KW!()), space0), velocity)),
        space0,
    ))(input)?;
    Ok((
        input,
        PHarmonic {
            pitch_gap,
            delay,
            velocity,
        },
    ))
}

fn chord(input: &str) -> IResult<&str, Expression> {
    let (input, (id, harmonics, _)) =
        tuple((head(CHORD_KW!()), many0(terminated(harmonic, space0)), end))(input)?;

    Ok((input, Expression::Chord(PChord { id, harmonics })))
}

fn accent(input: &str) -> IResult<&str, PAccent> {
    let (input, (delay, velocity)) = tuple((
        time,
        opt(delimited(
            terminated(char(JOIN_KW!()), space0),
            velocity,
            space0,
        )),
    ))(input)?;
    Ok((input, PAccent { delay, velocity }))
}

fn attack(input: &str) -> IResult<&str, Expression> {
    let (input, (id, accents, _)) =
        tuple((head(ATTACK_KW!()), many0(terminated(accent, space0)), end))(input)?;

    Ok((input, Expression::Attack(PAttack { id, accents })))
}

fn accentuated_chord(input: &str) -> IResult<&str, PChordAndAttack> {
    let (input, (chord_id, attack_id)) = tuple((id, opt(preceded(char(JOIN_KW!()), id))))(input)?;
    Ok((
        input,
        PChordAndAttack {
            chord_id,
            attack_id,
        },
    ))
}

fn chords(input: &str) -> IResult<&str, Expression> {
    let (input, (id, chords, _)) = tuple((
        head(CHORDLINE_KW!()),
        many0(terminated(accentuated_chord, space0)),
        end,
    ))(input)?;

    Ok((input, Expression::ChordLine(PChordLine { id, chords })))
}

fn hit(input: &str) -> IResult<&str, PHit> {
    let (input, (position, duration)) = tuple((
        terminated(time, space0),
        opt(delimited(
            terminated(char(JOIN_KW!()), space0),
            time,
            space0,
        )),
    ))(input)?;
    Ok((input, PHit { position, duration }))
}

fn hits(input: &str) -> IResult<&str, Expression> {
    let (input, (id, hits, duration)) =
        tuple((head(HITLINE_KW!()), many0(hit), delimited(per, time, end)))(input)?;
    Ok((input, Expression::HitLine(PHitLine { id, hits, duration })))
}

fn durations(input: &str) -> IResult<&str, Expression> {
    let (input, (id, durations)) =
        tuple((head(DURATIONLINE_KW!()), many0(terminated(time, space0))))(input)?;
    Ok((
        input,
        Expression::DurationLine(PDurationLine { id, durations }),
    ))
}

fn transition(input: &str) -> IResult<&str, PTransition> {
    let (input, oprog) = delimited(space0, opt(one_of("=~<>°")), space0)(input)?;

    let transition = match oprog {
        Some(c) => match c {
            LINEAR_TRANSITION_KW!() => PTransition::Linear,
            SIN_TRANSITION_KW!() => PTransition::Sin,
            EARLY_TRANSITION_KW!() => PTransition::Early,
            LATE_TRANSITION_KW!() => PTransition::Late,
            ROUND_TRANSITION_KW!() => PTransition::Round,
            _ => PTransition::None,
        },
        None => PTransition::None,
    };

    Ok((input, transition))
}

fn velocity(input: &str) -> IResult<&str, PVelocity> {
    let (input, (fadein, value, fadeout, transition, _)) = tuple((
        opt(char(FADEIN_KW!())),
        float,
        opt(char(FADEOUT_KW!())),
        transition,
        space0,
    ))(input)?;
    Ok((
        input,
        PVelocity {
            value,
            fadein: fadein.is_some(),
            fadeout: fadeout.is_some(),
            transition,
        },
    ))
}

fn velocities(input: &str) -> IResult<&str, Expression> {
    let (input, (id, velocities, _)) = tuple((
        head(VELOCITYLINE_KW!()),
        many0(terminated(velocity, space0)),
        end,
    ))(input)?;
    Ok((
        input,
        Expression::VelocityLine(PVelocityLine { id, velocities }),
    ))
}

fn pitch(input: &str) -> IResult<&str, PPitch> {
    let (input, (id, transition)) = tuple((alphanumeric1, transition))(input)?;

    Ok((input, PPitch { id, transition }))
}

fn pitchs(input: &str) -> IResult<&str, Expression> {
    let (input, (id, pitchs, _)) =
        tuple((head(PITCHLINE_KW!()), many0(terminated(pitch, space0)), end))(input)?;

    Ok((input, Expression::PitchLine(PPitchLine { id, pitchs })))
}

fn part(input: &str) -> IResult<&str, PFragment> {
    let (input, (hitline_id, durationline_id, pitchline_id, chordline_id, velocityline_id, mul, _)) =
        tuple((
            id,
            opt(preceded(char(COUPLING_KW!()), id)),
            opt(preceded(char(JOIN_KW!()), id)),
            opt(preceded(char(COUPLING_KW!()), id)),
            opt(preceded(char(JOIN_KW!()), id)),
            opt(preceded(delimited(space0, char(MUL_KW!()), space0), float)),
            space0,
        ))(input)?;
    Ok((
        input,
        PFragment::Part(PPart {
            hitline_id,
            durationline_id,
            pitchline_id,
            chordline_id,
            velocityline_id,
            mul,
        }),
    ))
}

fn seq_ref(input: &str) -> IResult<&str, PFragment> {
    let (input, (id, mul, _)) = tuple((
        delimited(char(REF_KW!()), id, space0),
        opt(preceded(terminated(char(MUL_KW!()), space0), digit1)),
        space0,
    ))(input)?;
    Ok((
        input,
        PFragment::SeqRef(PSeqRef {
            id,
            mul: mul.map(|s| usize::from_str(s).unwrap()),
        }),
    ))
}

fn fragments(input: &str) -> IResult<&str, PFragment> {
    let (input, (fragments, mul)) = tuple((
        delimited(
            terminated(char(OPEN_PARENT_KW!()), space0),
            many0(alt((seq_ref, part, fragments))),
            terminated(char(CLOSE_PARENT_KW!()), space0),
        ),
        delimited(terminated(char(MUL_KW!()), space0), digit1, space0),
    ))(input)?;
    Ok((
        input,
        PFragment::Fragments((fragments, usize::from_str(mul).unwrap())),
    ))
}

fn sequence(input: &str) -> IResult<&str, PSequence> {
    let (input, (_, id, _, properties, fragments, _)) = tuple((
        space1,
        id,
        delimited(space0, char(DEF_KW!()), space0),
        many0(property),
        many0(alt((seq_ref, part, fragments))),
        end,
    ))(input)?;
    let mut beat = None;

    for property in properties {
        if property.label == BEAT_KW!() {
            beat = Some(property.value);
        }
    }
    Ok((
        input,
        PSequence {
            id,
            beat,
            fragments,
        },
    ))
}

fn seq(input: &str) -> IResult<&str, Expression> {
    let (input, sequence) = preceded(tag(SEQUENCE_KW!()), sequence)(input)?;
    Ok((input, Expression::Seq(sequence)))
}

fn seqout(input: &str) -> IResult<&str, Expression> {
    let (input, sequence) = preceded(tag(SEQUENCE_OUTPUT_KW!()), sequence)(input)?;
    Ok((input, Expression::SeqOut(sequence)))
}

fn midiout(input: &str) -> IResult<&str, Expression> {
    let (input, sequence) = preceded(tag(MIDI_OUTPUT_KW!()), sequence)(input)?;
    Ok((input, Expression::MidiOut(sequence)))
}

pub fn parse(input: &str) -> Result<Vec<Expression>, failure::Error> {
    let (input, expressions) = many0(alt((
        beat,
        chord,
        attack,
        chords,
        hits,
        durations,
        pitchs,
        velocities,
        seq,
        seqout,
        midiout,
        multiline_comment, // multiline_comment must be evaluated before line_comment
        line_comment,
        end,
    )))(input)
    .map_err(|e| failure::err_msg(format!("tseq parser error : {:?}", e)))?;

    if input.is_empty() {
        Ok(expressions)
    } else {
        Err(failure::err_msg(format!("tseq parser error : {:?}", input)))
    }
}

#[test]
fn test_beat() {
    assert_eq!(
        beat(concat!(BEAT_KW!(), " Id06 ", DEF_KW!(), " 09\n")),
        Ok(("", Expression::Beat(PBeat { id: "Id06", bpm: 9 }),))
    );
    assert_eq!(
        beat(concat!(BEAT_KW!(), "  9zZ", DEF_KW!(), "9  \n")),
        Ok(("", Expression::Beat(PBeat { id: "9zZ", bpm: 9 }),))
    );
    assert_eq!(
        beat(concat!(BEAT_KW!(), " titi   ", DEF_KW!(), " 90\n")),
        Ok((
            "",
            Expression::Beat(PBeat {
                id: "titi",
                bpm: 90,
            }),
        ))
    );
}

#[test]
fn test_chord() {
    let s = concat!(
        CHORD_KW!(),
        " c ",
        DEF_KW!(),
        " 1 1.5",
        JOIN_KW!(),
        "2",
        SECOND_SYM_KW!(),
        " 3",
        ON_KW!(),
        "2",
        JOIN_KW!(),
        "1",
        ON_KW!(),
        "2",
        JOIN_KW!(),
        ".4 ",
        INTERVAL_KW!(),
        "6 ",
        JOIN_KW!(),
        "2",
        INTERVAL_KW!(),
        "15\n"
    );
    assert_eq!(
        chord(s),
        Ok((
            "",
            Expression::Chord(PChord {
                id: "c",
                harmonics: vec![
                    PHarmonic {
                        pitch_gap: PPitchGap::FreqRatio(PRatio { num: 1., den: 1. }),
                        delay: None,
                        velocity: None,
                    },
                    PHarmonic {
                        pitch_gap: PPitchGap::FreqRatio(PRatio { num: 1.5, den: 1. }),
                        delay: Some(PTime::Second(PRatio { num: 2., den: 1. })),
                        velocity: None,
                    },
                    PHarmonic {
                        pitch_gap: PPitchGap::FreqRatio(PRatio { num: 3., den: 2. }),
                        delay: Some(PTime::Rate(PRatio { num: 1., den: 2. })),
                        velocity: Some(PVelocity {
                            value: 0.4,
                            fadein: false,
                            fadeout: false,
                            transition: PTransition::None
                        }),
                    },
                    PHarmonic {
                        pitch_gap: PPitchGap::Interval(6),
                        delay: Some(PTime::Rate(PRatio { num: 2., den: 1. })),
                        velocity: None,
                    },
                    PHarmonic {
                        pitch_gap: PPitchGap::Interval(15),
                        delay: None,
                        velocity: None,
                    },
                ]
            })
        ))
    );
}

#[test]
fn test_chords() {
    assert_eq!(
        chords(concat!(
            CHORDLINE_KW!(),
            " cs ",
            DEF_KW!(),
            " c1 c2-a c3 \n"
        )),
        Ok((
            "",
            Expression::ChordLine(PChordLine {
                id: "cs",
                chords: vec![
                    PChordAndAttack {
                        chord_id: "c1",
                        attack_id: None
                    },
                    PChordAndAttack {
                        chord_id: "c2",
                        attack_id: Some("a")
                    },
                    PChordAndAttack {
                        chord_id: "c3",
                        attack_id: None
                    }
                ]
            }),
        ))
    );
}

#[test]
fn test_hits() {
    assert_eq!(
        hits(concat!(
            HITLINE_KW!(),
            " p1",
            DEF_KW!(),
            " 0.5 .75 ",
            PER_KW!(),
            " 1/3s\n"
        )),
        Ok((
            "",
            Expression::HitLine(PHitLine {
                id: "p1",
                hits: vec![
                    PHit {
                        position: PTime::Rate(PRatio { num: 0.5, den: 1. }),
                        duration: None
                    },
                    PHit {
                        position: PTime::Rate(PRatio { num: 0.75, den: 1. }),
                        duration: None
                    }
                ],
                duration: PTime::Second(PRatio { num: 1., den: 3. })
            }),
        ))
    );
    assert_eq!(
        hits(concat!(
            HITLINE_KW!(),
            " p1",
            DEF_KW!(),
            " 0.5",
            JOIN_KW!(),
            ".2 .75 ",
            JOIN_KW!(),
            " .3 ",
            PER_KW!(),
            " 1\n"
        )),
        Ok((
            "",
            Expression::HitLine(PHitLine {
                id: "p1",
                hits: vec![
                    PHit {
                        position: PTime::Rate(PRatio { num: 0.5, den: 1. }),
                        duration: Some(PTime::Rate(PRatio { num: 0.2, den: 1. })),
                    },
                    PHit {
                        position: PTime::Rate(PRatio { num: 0.75, den: 1. }),
                        duration: Some(PTime::Rate(PRatio { num: 0.3, den: 1. }))
                    }
                ],
                duration: PTime::Rate(PRatio { num: 1., den: 1. })
            }),
        ))
    );
}

#[test]
fn test_velos() {
    assert_eq!(
        velocities(concat!(
            VELOCITYLINE_KW!(),
            " v1",
            DEF_KW!(),
            " /.5\\ ~ /1 .75\\=0.9\n"
        )),
        Ok((
            "",
            Expression::VelocityLine(PVelocityLine {
                id: "v1",
                velocities: vec![
                    PVelocity {
                        value: 0.5,
                        fadein: true,
                        fadeout: true,
                        transition: PTransition::Sin
                    },
                    PVelocity {
                        value: 1.,
                        fadein: true,
                        fadeout: false,
                        transition: PTransition::None
                    },
                    PVelocity {
                        value: 0.75,
                        fadein: false,
                        fadeout: true,
                        transition: PTransition::Linear
                    },
                    PVelocity {
                        value: 0.9,
                        fadein: false,
                        fadeout: false,
                        transition: PTransition::None
                    },
                ],
            }),
        ))
    );
}

#[test]
fn test_pitchs() {
    assert_eq!(
        pitchs(concat!(PITCHLINE_KW!(), " blank ", DEF_KW!(), "\n")),
        Ok((
            "",
            Expression::PitchLine(PPitchLine {
                id: "blank",
                pitchs: vec![]
            }),
        ))
    );
    assert_eq!(
        pitchs(concat!(
            PITCHLINE_KW!(),
            " intro ",
            DEF_KW!(),
            " G9 = B7~e5 > f2 <a0  \n"
        )),
        Ok((
            "",
            Expression::PitchLine(PPitchLine {
                id: "intro",
                pitchs: vec![
                    PPitch {
                        id: "G9",
                        transition: PTransition::Linear
                    },
                    PPitch {
                        id: "B7",
                        transition: PTransition::Sin
                    },
                    PPitch {
                        id: "e5",
                        transition: PTransition::Late
                    },
                    PPitch {
                        id: "f2",
                        transition: PTransition::Early
                    },
                    PPitch {
                        id: "a0",
                        transition: PTransition::None
                    },
                ]
            }),
        ))
    );
}

#[test]
fn test_part() {
    assert_eq!(
        part(concat!(
            "p",
            JOIN_KW!(),
            "n",
            JOIN_KW!(),
            "v",
            MUL_KW!(),
            "3"
        )),
        Ok((
            "",
            PFragment::Part(PPart {
                hitline_id: "p",
                durationline_id: None,
                pitchline_id: Some("n"),
                chordline_id: None,
                velocityline_id: Some("v"),
                mul: Some(3.),
            }),
        ))
    );
    assert_eq!(
        part(concat!("p", JOIN_KW!(), "n ", MUL_KW!(), " 3 ")),
        Ok((
            "",
            PFragment::Part(PPart {
                hitline_id: "p",
                durationline_id: None,
                pitchline_id: Some("n"),
                chordline_id: None,
                velocityline_id: None,
                mul: Some(3.),
            }),
        ))
    );
    assert_eq!(
        part(concat!("p", JOIN_KW!(), "n", JOIN_KW!(), "v0")),
        Ok((
            "",
            PFragment::Part(PPart {
                hitline_id: "p",
                durationline_id: None,
                pitchline_id: Some("n"),
                chordline_id: None,
                velocityline_id: Some("v0"),
                mul: None,
            }),
        ))
    );
    assert_eq!(
        part(concat!("p1", MUL_KW!(), "3 ")),
        Ok((
            "",
            PFragment::Part(PPart {
                hitline_id: "p1",
                durationline_id: None,
                pitchline_id: None,
                chordline_id: None,
                velocityline_id: None,
                mul: Some(3.),
            }),
        ))
    );
    assert_eq!(
        part(concat!(
            "p",
            JOIN_KW!(),
            "n",
            JOIN_KW!(),
            "v",
            MUL_KW!(),
            "3"
        )),
        Ok((
            "",
            PFragment::Part(PPart {
                hitline_id: "p",
                durationline_id: None,
                pitchline_id: Some("n"),
                chordline_id: None,
                velocityline_id: Some("v"),
                mul: Some(3.),
            }),
        ))
    );
    assert_eq!(
        part("4_p0f"),
        Ok((
            "",
            PFragment::Part(PPart {
                hitline_id: "4_p0f",
                durationline_id: None,
                pitchline_id: None,
                chordline_id: None,
                velocityline_id: None,
                mul: None,
            }),
        ))
    );
}

#[test]
fn test_seq_ref() {
    assert_eq!(
        seq_ref(concat!(REF_KW!(), "s_01")),
        Ok((
            "",
            PFragment::SeqRef(PSeqRef {
                id: "s_01",
                mul: None
            }),
        ))
    );
    assert_eq!(
        seq_ref(concat!(REF_KW!(), "1sr_ ", MUL_KW!(), "2")),
        Ok((
            "",
            PFragment::SeqRef(PSeqRef {
                id: "1sr_",
                mul: Some(2)
            }),
        ))
    );
}

#[test]
fn test_fragments() {
    assert_eq!(
        fragments(concat!(
            OPEN_PARENT_KW!(),
            "p ",
            REF_KW!(),
            "s ",
            CLOSE_PARENT_KW!(),
            MUL_KW!(),
            "3"
        )),
        Ok((
            "",
            PFragment::Fragments((
                vec![
                    PFragment::Part(PPart {
                        hitline_id: "p",
                        durationline_id: None,
                        pitchline_id: None,
                        chordline_id: None,
                        velocityline_id: None,
                        mul: None,
                    }),
                    PFragment::SeqRef(PSeqRef { id: "s", mul: None })
                ],
                3
            ))
        ))
    );
}

#[test]
fn test_sequence() {
    assert_eq!(
        sequence(concat!(
            " seq_03",
            DEF_KW!(),
            BEAT_KW!(),
            ASSIGNMENT_KW!(),
            " _b_ ",
            REF_KW!(),
            "s_1 p1 p1",
            JOIN_KW!(),
            "n2 ",
            REF_KW!(),
            "s_2",
            MUL_KW!(),
            "3 p2",
            JOIN_KW!(),
            "n1",
            JOIN_KW!(),
            "v1 ",
            MUL_KW!(),
            " 2 \n"
        )),
        Ok((
            "",
            PSequence {
                id: "seq_03",
                beat: Some("_b_"),
                fragments: vec![
                    PFragment::SeqRef(PSeqRef {
                        id: "s_1",
                        mul: None
                    }),
                    PFragment::Part(PPart {
                        hitline_id: "p1",
                        durationline_id: None,
                        pitchline_id: None,
                        chordline_id: None,
                        velocityline_id: None,
                        mul: None,
                    }),
                    PFragment::Part(PPart {
                        hitline_id: "p1",
                        durationline_id: None,
                        pitchline_id: Some("n2"),
                        chordline_id: None,
                        velocityline_id: None,
                        mul: None,
                    }),
                    PFragment::SeqRef(PSeqRef {
                        id: "s_2",
                        mul: Some(3)
                    }),
                    PFragment::Part(PPart {
                        hitline_id: "p2",
                        durationline_id: None,
                        pitchline_id: Some("n1"),
                        chordline_id: None,
                        velocityline_id: Some("v1"),
                        mul: Some(2.),
                    })
                ],
            }
        ),)
    );
}

#[test]
fn test_seq() {
    assert_eq!(
        seq(concat!(
            SEQUENCE_KW!(),
            " s ",
            DEF_KW!(),
            " ",
            REF_KW!(),
            "s_1\n"
        )),
        Ok((
            "",
            Expression::Seq(PSequence {
                id: "s",
                beat: None,

                fragments: vec![PFragment::SeqRef(PSeqRef {
                    id: "s_1",
                    mul: None
                })]
            })
        ))
    );
}

#[test]
fn test_seqout() {
    assert_eq!(
        seqout(concat!(
            SEQUENCE_OUTPUT_KW!(),
            "   s ",
            DEF_KW!(),
            " ",
            REF_KW!(),
            "s_1 \n"
        )),
        Ok((
            "",
            Expression::SeqOut(PSequence {
                id: "s",
                beat: None,

                fragments: vec![PFragment::SeqRef(PSeqRef {
                    id: "s_1",
                    mul: None
                })]
            })
        ))
    );
}

#[test]
fn test_line_comment() {
    assert_eq!(
        line_comment(concat!(LINE_COMMENT_KW!(), " this is a comment !\n")),
        Ok(("", Expression::None))
    );
    assert_eq!(
        line_comment(concat!(LINE_COMMENT_KW!(), " this is a comment !\n\n\n")),
        Ok(("", Expression::None))
    );
}

#[test]
fn test_end() {
    assert_eq!(end("\n"), Ok(("", Expression::None)));
    assert_eq!(end("\n   \n"), Ok(("", Expression::None)));
    assert_eq!(end(" \n\n  \n"), Ok(("", Expression::None)));
}

#[test]
fn test_parse() {
    let res = parse(concat!(
        "\n",
        LINE_COMMENT_KW!(),
        " 90 BPM\n",
        BEAT_KW!(),
        " b ",
        DEF_KW!(),
        " 90\n\n",
        VELOCITYLINE_KW!(),
        " v",
        DEF_KW!(),
        " 1\n\n"
    ))
    .unwrap();
    assert_eq!(
        res,
        vec![
            Expression::None,
            Expression::None,
            Expression::Beat(PBeat { id: "b", bpm: 90 }),
            Expression::VelocityLine(PVelocityLine {
                id: "v",
                velocities: vec![PVelocity {
                    value: 1.,
                    transition: PTransition::None,
                    fadein: false,
                    fadeout: false
                }],
            })
        ]
    );
}

/*
#[cfg(test)]
mod tests {
    use super::*;
*/
//}
