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

#[derive(Debug, PartialEq)]
pub struct PBeat<'a> {
    pub id: &'a str,
    pub bpm: usize,
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
pub struct PRatio {
    pub num: f32,
    pub den: f32,
}

#[derive(Debug, PartialEq)]
pub struct PHarmonic {
    pub freq_ratio: PRatio,
    pub delay: Option<f32>,
    pub velocity: Option<PVelocity>,
}

#[derive(Debug, PartialEq)]
pub struct PChord<'a> {
    pub id: &'a str,
    pub harmonics: Vec<PHarmonic>,
}

#[derive(Debug, PartialEq)]
pub struct PChordLine<'a> {
    pub id: &'a str,
    pub chords: Vec<&'a str>,
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
    pub position: f32,
    pub duration: Option<f32>,
}

#[derive(Debug, PartialEq)]
pub struct PHitLine<'a> {
    pub id: &'a str,
    pub hits: Vec<PHit>,
    pub duration: f32,
}

#[derive(Debug, PartialEq)]
pub struct PDurationLine<'a> {
    pub id: &'a str,
    pub durations: Vec<f32>,
}

#[derive(Debug, PartialEq)]
pub struct PVelocity {
    pub value: f32,
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
}

#[derive(Debug, PartialEq)]
pub struct PSequence<'a> {
    pub id: &'a str,
    pub beat: Option<&'a str>,
    pub fragments: Vec<PFragment<'a>>,
}

#[derive(Debug, PartialEq)]
pub enum Exp<'a> {
    Beat(PBeat<'a>),
    Chord(PChord<'a>),
    ChordLine(PChordLine<'a>),
    DurationLine(PDurationLine<'a>),
    VelocityLine(PVelocityLine<'a>),
    HitLine(PHitLine<'a>),
    PitchLine(PPitchLine<'a>),
    Seq(PSequence<'a>),
    FreqOut(PSequence<'a>),
    VelOut(PSequence<'a>),
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
        delimited(space0, char(':'), space0),
    )
}

fn slash(input: &str) -> IResult<&str, char> {
    delimited(space0, char('/'), space0)(input)
}

fn end(input: &str) -> IResult<&str, Exp> {
    let (input, _) = many1_count(preceded(space0, newline))(input)?;
    Ok((input, Exp::None))
}

fn comment(input: &str) -> IResult<&str, Exp> {
    let (input, _) = delimited(tag(";;"), take_until("\n"), end)(input)?;
    Ok((input, Exp::None))
}

fn beat(input: &str) -> IResult<&str, Exp> {
    let (input, (id, bpm, _)) = tuple((head("beat"), digit1, end))(input)?;
    Ok((
        input,
        Exp::Beat(PBeat {
            id,
            bpm: usize::from_str(bpm).unwrap(),
        }),
    ))
}

fn hit(input: &str) -> IResult<&str, PHit> {
    let (input, (position, duration)) = tuple((
        terminated(float, space0),
        opt(delimited(terminated(char(','), space0), float, space0)),
    ))(input)?;
    Ok((input, PHit { position, duration }))
}

fn hits(input: &str) -> IResult<&str, Exp> {
    let (input, (id, hits, duration)) =
        tuple((head("hits"), many0(hit), delimited(slash, float, end)))(input)?;
    Ok((input, Exp::HitLine(PHitLine { id, hits, duration })))
}

fn durations(input: &str) -> IResult<&str, Exp> {
    let (input, (id, durations)) =
        tuple((head("durations"), many0(terminated(float, space0))))(input)?;
    Ok((input, Exp::DurationLine(PDurationLine { id, durations })))
}

fn transition(input: &str) -> IResult<&str, PTransition> {
    let (input, oprog) = preceded(space0, opt(one_of("=~<>°!")))(input)?;

    let transition = match oprog {
        Some(c) => match c {
            '=' => PTransition::Linear,
            '~' => PTransition::Sin,
            '<' => PTransition::Early,
            '>' => PTransition::Late,
            '°' => PTransition::Round,
            _ => PTransition::None,
        },
        None => PTransition::None,
    };

    Ok((input, transition))
}

fn velocity(input: &str) -> IResult<&str, PVelocity> {
    let (input, (value, transition)) = tuple((float, transition))(input)?;
    Ok((input, PVelocity { value, transition }))
}

fn velos(input: &str) -> IResult<&str, Exp> {
    let (input, (id, velocities, _)) =
        tuple((head("velos"), many0(terminated(velocity, space0)), end))(input)?;
    Ok((input, Exp::VelocityLine(PVelocityLine { id, velocities })))
}

fn ratio(input: &str) -> IResult<&str, PRatio> {
    let (input, (num, den)) = tuple((float, opt(delimited(slash, float, space0))))(input)?;
    Ok((
        input,
        PRatio {
            num,
            den: den.unwrap_or(1.),
        },
    ))
}

fn harmonic(input: &str) -> IResult<&str, PHarmonic> {
    let (input, (freq_ratio, delay, velocity)) = tuple((
        ratio,
        opt(delimited(terminated(char(','), space0), float, space0)),
        opt(delimited(terminated(char(','), space0), velocity, space0)),
    ))(input)?;
    Ok((
        input,
        PHarmonic {
            freq_ratio,
            delay,
            velocity,
        },
    ))
}

fn chord(input: &str) -> IResult<&str, Exp> {
    let (input, (id, harmonics, _)) =
        tuple((head("chord"), many0(terminated(harmonic, space0)), end))(input)?;

    Ok((input, Exp::Chord(PChord { id, harmonics })))
}

fn chords(input: &str) -> IResult<&str, Exp> {
    let (input, (id, chords, _)) = tuple((
        head("chords"),
        many0(terminated(alphanumeric1, space0)),
        end,
    ))(input)?;

    Ok((input, Exp::ChordLine(PChordLine { id, chords })))
}

fn pitch(input: &str) -> IResult<&str, PPitch> {
    let (input, (id, transition)) = tuple((alphanumeric1, transition))(input)?;

    Ok((input, PPitch { id, transition }))
}

fn pitchs(input: &str) -> IResult<&str, Exp> {
    let (input, (id, pitchs, _)) =
        tuple((head("pitchs"), many0(terminated(pitch, space0)), end))(input)?;

    Ok((input, Exp::PitchLine(PPitchLine { id, pitchs })))
}

fn part(input: &str) -> IResult<&str, PFragment> {
    let (input, (hitline_id, durationline_id, pitchline_id, chordline_id, velocityline_id, mul, _)) =
        tuple((
            id,
            opt(preceded(char('&'), id)),
            opt(preceded(char(','), id)),
            opt(preceded(char('&'), id)),
            opt(preceded(char(','), id)),
            opt(preceded(delimited(space0, char('*'), space0), float)),
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
        delimited(char('@'), id, space0),
        opt(preceded(terminated(char('*'), space0), digit1)),
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

fn sequence(input: &str) -> IResult<&str, PSequence> {
    let (input, (_, id, _, beat, fragments, _)) = tuple((
        space1,
        id,
        delimited(space0, char(':'), space0),
        opt(delimited(slash, id, space0)),
        many0(alt((seq_ref, part))),
        end,
    ))(input)?;
    Ok((
        input,
        PSequence {
            id,
            beat,
            fragments,
        },
    ))
}

fn seq(input: &str) -> IResult<&str, Exp> {
    let (input, sequence) = preceded(tag("seq"), sequence)(input)?;
    Ok((input, Exp::Seq(sequence)))
}

fn freqout(input: &str) -> IResult<&str, Exp> {
    let (input, sequence) = preceded(tag("freqout"), sequence)(input)?;
    Ok((input, Exp::FreqOut(sequence)))
}

fn velout(input: &str) -> IResult<&str, Exp> {
    let (input, sequence) = preceded(tag("velout"), sequence)(input)?;
    Ok((input, Exp::VelOut(sequence)))
}

fn midiout(input: &str) -> IResult<&str, Exp> {
    let (input, sequence) = preceded(tag("midiout"), sequence)(input)?;
    Ok((input, Exp::MidiOut(sequence)))
}

pub fn parse(input: &str) -> Result<Vec<Exp>, failure::Error> {
    let (input, expressions) = many0(alt((
        beat, chord, chords, hits, durations, pitchs, velos, seq, freqout, velout, midiout,
        comment, end,
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
        beat("beat Id06 : 09\n"),
        Ok(("", Exp::Beat(PBeat { id: "Id06", bpm: 9 }),))
    );
    assert_eq!(
        beat("beat  9zZ:9  \n"),
        Ok(("", Exp::Beat(PBeat { id: "9zZ", bpm: 9 }),))
    );
    assert_eq!(
        beat("beat titi   : 90\n"),
        Ok((
            "",
            Exp::Beat(PBeat {
                id: "titi",
                bpm: 90,
            }),
        ))
    );
}

#[test]
fn test_chord() {
    assert_eq!(
        chord("chord c : 1 1.5,2 3/2,0.1,.4\n"),
        Ok((
            "",
            Exp::Chord(PChord {
                id: "c",
                harmonics: vec![
                    PHarmonic {
                        freq_ratio: PRatio { num: 1., den: 1. },
                        delay: None,
                        velocity: None,
                    },
                    PHarmonic {
                        freq_ratio: PRatio { num: 1.5, den: 1. },
                        delay: Some(2.),
                        velocity: None,
                    },
                    PHarmonic {
                        freq_ratio: PRatio { num: 3., den: 2. },
                        delay: Some(0.1),
                        velocity: Some(PVelocity {
                            value: 0.4,
                            transition: PTransition::None
                        }),
                    }
                ]
            })
        ))
    );
}

#[test]
fn test_chords() {
    assert_eq!(
        chords("chords cs : c1 c2 c3 \n"),
        Ok((
            "",
            Exp::ChordLine(PChordLine {
                id: "cs",
                chords: vec!["c1", "c2", "c3"]
            }),
        ))
    );
}

#[test]
fn test_hits() {
    assert_eq!(
        hits("hits p1: 0.5 .75 / 1\n"),
        Ok((
            "",
            Exp::HitLine(PHitLine {
                id: "p1",
                hits: vec![
                    PHit {
                        position: 0.5,
                        duration: None
                    },
                    PHit {
                        position: 0.75,
                        duration: None
                    }
                ],
                duration: 1.0
            }),
        ))
    );
    assert_eq!(
        hits("hits p1: 0.5,.2 .75 , .3 / 1\n"),
        Ok((
            "",
            Exp::HitLine(PHitLine {
                id: "p1",
                hits: vec![
                    PHit {
                        position: 0.5,
                        duration: Some(0.2),
                    },
                    PHit {
                        position: 0.75,
                        duration: Some(0.3)
                    }
                ],
                duration: 1.0
            }),
        ))
    );
}

#[test]
fn test_velos() {
    assert_eq!(
        velos("velos v1: .5 ~ 1 .75=0.9\n"),
        Ok((
            "",
            Exp::VelocityLine(PVelocityLine {
                id: "v1",
                velocities: vec![
                    PVelocity {
                        value: 0.5,
                        transition: PTransition::Sin
                    },
                    PVelocity {
                        value: 1.,
                        transition: PTransition::None
                    },
                    PVelocity {
                        value: 0.75,
                        transition: PTransition::Linear
                    },
                    PVelocity {
                        value: 0.9,
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
        pitchs("pitchs blank :\n"),
        Ok((
            "",
            Exp::PitchLine(PPitchLine {
                id: "blank",
                pitchs: vec![]
            }),
        ))
    );
    assert_eq!(
        pitchs("pitchs intro : G9 = B7~e5 > f2 <a0  \n"),
        Ok((
            "",
            Exp::PitchLine(PPitchLine {
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
        part("p,n,v*3"),
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
        part("p,n * 3 "),
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
        part("p,n,v0"),
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
        part("p1*3 "),
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
        part("p,n,v*3"),
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
        seq_ref("@s_01"),
        Ok((
            "",
            PFragment::SeqRef(PSeqRef {
                id: "s_01",
                mul: None
            }),
        ))
    );
    assert_eq!(
        seq_ref("@1sr_ *2"),
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
fn test_sequence() {
    assert_eq!(
        sequence(" seq_03:/ _b_ @s_1 p1 p1,n2 @s_2*3 p2,n1,v1 * 2 \n"),
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
        seq("seq s : @s_1\n"),
        Ok((
            "",
            Exp::Seq(PSequence {
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
fn test_freqout() {
    assert_eq!(
        freqout("freqout   s : @s_1 \n"),
        Ok((
            "",
            Exp::FreqOut(PSequence {
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
fn test_comment() {
    assert_eq!(comment(";; this is a comment !\n"), Ok(("", Exp::None)));
    assert_eq!(comment(";; this is a comment !\n\n\n"), Ok(("", Exp::None)));
}

#[test]
fn test_end() {
    assert_eq!(end("\n"), Ok(("", Exp::None)));
    assert_eq!(end("\n   \n"), Ok(("", Exp::None)));
    assert_eq!(end(" \n\n  \n"), Ok(("", Exp::None)));
}

#[test]
fn test_parse() {
    let res = parse("\n;; 90 BPM\nbeat b : 90\n\nvelos v: 1\n\n").unwrap();
    assert_eq!(
        res,
        vec![
            Exp::None,
            Exp::None,
            Exp::Beat(PBeat { id: "b", bpm: 90 }),
            Exp::VelocityLine(PVelocityLine {
                id: "v",
                velocities: vec![PVelocity {
                    value: 1.,
                    transition: PTransition::None
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
