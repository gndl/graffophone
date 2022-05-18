use nom::{
    branch::alt,
    bytes::complete::{tag, take_until},
    character::complete::{alphanumeric1, char, digit1, newline, space0, space1},
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

#[derive(Debug, PartialEq)]
pub struct PPitchLine<'a> {
    pub id: &'a str,
    pub pitchs: Vec<&'a str>,
}

#[derive(Debug, PartialEq)]
pub struct PHit {
    pub position: f32,
    pub duration: Option<f32>,
}

#[derive(Debug, PartialEq)]
pub struct PPattern<'a> {
    pub id: &'a str,
    pub hits: Vec<PHit>,
    pub duration: f32,
}

#[derive(Debug, PartialEq)]
pub struct PVelocityLine<'a> {
    pub id: &'a str,
    pub values: Vec<f32>,
}

#[derive(Debug, PartialEq)]
pub struct PPart<'a> {
    pub pattern: &'a str,
    pub pitchs: Option<&'a str>,
    pub velos: Option<&'a str>,
    pub mul: Option<f32>,
}

#[derive(Debug, PartialEq)]
pub struct PSeqRef<'a> {
    pub id: &'a str,
    pub mul: Option<f32>,
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
    PitchLine(PPitchLine<'a>),
    Pattern(PPattern<'a>),
    VelocityLine(PVelocityLine<'a>),
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
    let (input, _) = delimited(char('#'), take_until("\n"), end)(input)?;
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
        opt(delimited(terminated(char('d'), space0), float, space0)),
    ))(input)?;
    Ok((input, PHit { position, duration }))
}

fn pattern(input: &str) -> IResult<&str, Exp> {
    let (input, (id, hits, duration)) =
        tuple((head("pattern"), many0(hit), delimited(slash, float, end)))(input)?;
    Ok((input, Exp::Pattern(PPattern { id, hits, duration })))
}

fn velocities(input: &str) -> IResult<&str, Exp> {
    let (input, (id, values, _)) =
        tuple((head("velos"), many0(terminated(float, space0)), end))(input)?;
    Ok((input, Exp::VelocityLine(PVelocityLine { id, values })))
}

fn pitchs(input: &str) -> IResult<&str, Exp> {
    let (input, (id, pitchs, _)) = tuple((
        head("pitchs"),
        many0(terminated(alphanumeric1, space0)),
        end,
    ))(input)?;

    Ok((input, Exp::PitchLine(PPitchLine { id, pitchs })))
}

fn part(input: &str) -> IResult<&str, PFragment> {
    let (input, (pattern, pitchs, velos, mul, _)) = tuple((
        id,
        opt(preceded(char('.'), id)),
        opt(preceded(char('.'), id)),
        opt(preceded(delimited(space0, char('*'), space0), float)),
        space0,
    ))(input)?;
    Ok((
        input,
        PFragment::Part(PPart {
            pattern,
            pitchs,
            velos,
            mul,
        }),
    ))
}

fn seq_ref(input: &str) -> IResult<&str, PFragment> {
    let (input, (id, mul, _)) = tuple((
        delimited(char('$'), id, space0),
        opt(preceded(terminated(char('*'), space0), float)),
        space0,
    ))(input)?;
    Ok((input, PFragment::SeqRef(PSeqRef { id, mul })))
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
        beat, pattern, pitchs, velocities, seq, freqout, velout, midiout, comment, end,
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
fn test_pattern() {
    assert_eq!(
        pattern("pattern p1: 0.5 .75 / 1\n"),
        Ok((
            "",
            Exp::Pattern(PPattern {
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
        pattern("pattern p1: 0.5d.2 .75 d .3 / 1\n"),
        Ok((
            "",
            Exp::Pattern(PPattern {
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
fn test_velocities() {
    assert_eq!(
        velocities("velos v1: .5 1 .75 0.9\n"),
        Ok((
            "",
            Exp::VelocityLine(PVelocityLine {
                id: "v1",
                values: vec![0.5, 1., 0.75, 0.9],
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
        pitchs("pitchs intro : a0 G9  B7 \n"),
        Ok((
            "",
            Exp::PitchLine(PPitchLine {
                id: "intro",
                pitchs: vec!["a0", "G9", "B7"]
            }),
        ))
    );
}

#[test]
fn test_part() {
    assert_eq!(
        part("p.n.v*3"),
        Ok((
            "",
            PFragment::Part(PPart {
                pattern: "p",
                pitchs: Some("n"),
                velos: Some("v"),
                mul: Some(3.),
            }),
        ))
    );
    assert_eq!(
        part("p.n * 3 "),
        Ok((
            "",
            PFragment::Part(PPart {
                pattern: "p",
                pitchs: Some("n"),
                velos: None,
                mul: Some(3.),
            }),
        ))
    );
    assert_eq!(
        part("p.n.v0"),
        Ok((
            "",
            PFragment::Part(PPart {
                pattern: "p",
                pitchs: Some("n"),
                velos: Some("v0"),
                mul: None,
            }),
        ))
    );
    assert_eq!(
        part("p1*3 "),
        Ok((
            "",
            PFragment::Part(PPart {
                pattern: "p1",
                pitchs: None,
                velos: None,
                mul: Some(3.),
            }),
        ))
    );
    assert_eq!(
        part("p.n.v*3"),
        Ok((
            "",
            PFragment::Part(PPart {
                pattern: "p",
                pitchs: Some("n"),
                velos: Some("v"),
                mul: Some(3.),
            }),
        ))
    );
    assert_eq!(
        part("4_p0f"),
        Ok((
            "",
            PFragment::Part(PPart {
                pattern: "4_p0f",
                pitchs: None,
                velos: None,
                mul: None,
            }),
        ))
    );
}

#[test]
fn test_seq_ref() {
    assert_eq!(
        seq_ref("$s_01"),
        Ok((
            "",
            PFragment::SeqRef(PSeqRef {
                id: "s_01",
                mul: None
            }),
        ))
    );
    assert_eq!(
        seq_ref("$1sr_ *2.5"),
        Ok((
            "",
            PFragment::SeqRef(PSeqRef {
                id: "1sr_",
                mul: Some(2.5)
            }),
        ))
    );
}

#[test]
fn test_sequence() {
    assert_eq!(
        sequence(" seq_03:/ _b_ $s_1 p1 p1.n2 $s_2*3 p2.n1.v1 * 2 \n"),
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
                        pattern: "p1",
                        pitchs: None,
                        velos: None,
                        mul: None,
                    }),
                    PFragment::Part(PPart {
                        pattern: "p1",
                        pitchs: Some("n2"),
                        velos: None,
                        mul: None,
                    }),
                    PFragment::SeqRef(PSeqRef {
                        id: "s_2",
                        mul: Some(3.)
                    }),
                    PFragment::Part(PPart {
                        pattern: "p2",
                        pitchs: Some("n1"),
                        velos: Some("v1"),
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
        seq("seq s : $s_1\n"),
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
        freqout("freqout   s : $s_1 \n"),
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
    assert_eq!(comment("# this is a comment !\n"), Ok(("", Exp::None)));
    assert_eq!(comment("# this is a comment !\n\n\n"), Ok(("", Exp::None)));
}

#[test]
fn test_end() {
    assert_eq!(end("\n"), Ok(("", Exp::None)));
    assert_eq!(end("\n   \n"), Ok(("", Exp::None)));
    assert_eq!(end(" \n\n  \n"), Ok(("", Exp::None)));
}

#[test]
fn test_parse() {
    let res = parse("\n# 90 BPM\nbeat b : 90\n\nvelos v: 1\n\n").unwrap();
    assert_eq!(
        res,
        vec![
            Exp::None,
            Exp::None,
            Exp::Beat(PBeat { id: "b", bpm: 90 }),
            Exp::VelocityLine(PVelocityLine {
                id: "v",
                values: vec![1.],
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
