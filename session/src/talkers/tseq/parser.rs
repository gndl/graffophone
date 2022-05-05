use nom::{
    branch::alt,
    bytes::complete::{tag, take_till, take_until, take_while1},
    character::complete::{alpha1, alphanumeric1, char, digit1, newline, space0, space1},
    character::{is_alphanumeric, is_newline},
    combinator::{map, opt, recognize, value},
    error::ParseError,
    multi::{many0, many1_count},
    number::complete::float,
    sequence::{delimited, pair, preceded, terminated, tuple},
    IResult,
};

use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub struct Beat<'a> {
    pub id: &'a str,
    pub bpm: i32,
}

#[derive(Debug, PartialEq)]
pub struct Notes<'a> {
    pub id: &'a str,
    pub names: Vec<&'a str>,
}

#[derive(Debug, PartialEq)]
pub struct Pattern<'a> {
    pub id: &'a str,
    pub times: Vec<f32>,
    pub duration: f32,
}

#[derive(Debug, PartialEq)]
pub struct Velocities<'a> {
    pub id: &'a str,
    pub values: Vec<f32>,
}

#[derive(Debug, PartialEq)]
pub struct Part<'a> {
    pub pattern: &'a str,
    pub notes: Option<&'a str>,
    pub velos: Option<&'a str>,
    pub mul: Option<f32>,
}

#[derive(Debug, PartialEq)]
pub struct SeqRef<'a> {
    pub id: &'a str,
    pub mul: Option<f32>,
}

#[derive(Debug, PartialEq)]
pub enum Fragment<'a> {
    Part(Part<'a>),
    SeqRef(SeqRef<'a>),
}

#[derive(Debug, PartialEq)]
pub struct Sequence<'a> {
    pub id: &'a str,
    pub beat: Option<&'a str>,
    pub fragments: Vec<Fragment<'a>>,
}

#[derive(Debug, PartialEq)]
pub enum Exp<'a> {
    Beat(Beat<'a>),
    Notes(Notes<'a>),
    Pattern(Pattern<'a>),
    Velocities(Velocities<'a>),
    Seq(Sequence<'a>),
    FreqOut(Sequence<'a>),
    VelOut(Sequence<'a>),
    MidiOut(Sequence<'a>),
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
        Exp::Beat(Beat {
            id,
            bpm: i32::from_str(bpm).unwrap(),
        }),
    ))
}

fn pattern(input: &str) -> IResult<&str, Exp> {
    let (input, (id, times, duration)) = tuple((
        head("pattern"),
        many0(terminated(float, space0)),
        delimited(slash, float, end),
    ))(input)?;
    Ok((
        input,
        Exp::Pattern(Pattern {
            id,
            times,
            duration,
        }),
    ))
}

fn velocities(input: &str) -> IResult<&str, Exp> {
    let (input, (id, values, _)) =
        tuple((head("velos"), many0(terminated(float, space0)), end))(input)?;
    Ok((input, Exp::Velocities(Velocities { id, values })))
}

fn notes(input: &str) -> IResult<&str, Exp> {
    let (input, (id, names, _)) =
        tuple((head("notes"), many0(terminated(alphanumeric1, space0)), end))(input)?;

    Ok((input, Exp::Notes(Notes { id, names })))
}

fn part(input: &str) -> IResult<&str, Fragment> {
    let (input, (pattern, notes, velos, mul, _)) = tuple((
        id,
        opt(preceded(char('.'), id)),
        opt(preceded(char('.'), id)),
        opt(preceded(delimited(space0, char('*'), space0), float)),
        space0,
    ))(input)?;
    Ok((
        input,
        Fragment::Part(Part {
            pattern,
            notes,
            velos,
            mul,
        }),
    ))
}

fn seq_ref(input: &str) -> IResult<&str, Fragment> {
    let (input, (id, mul, _)) = tuple((
        delimited(char('$'), id, space0),
        opt(preceded(terminated(char('*'), space0), float)),
        space0,
    ))(input)?;
    Ok((input, Fragment::SeqRef(SeqRef { id, mul })))
}

fn sequence(input: &str) -> IResult<&str, Sequence> {
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
        Sequence {
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

fn parse(input: &str) -> Result<Vec<Exp>, failure::Error> {
    let (input, expressions) = many0(alt((
        beat, pattern, notes, velocities, seq, freqout, velout, midiout, comment, end,
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
        Ok(("", Exp::Beat(Beat { id: "Id06", bpm: 9 }),))
    );
    assert_eq!(
        beat("beat  9zZ:9  \n"),
        Ok(("", Exp::Beat(Beat { id: "9zZ", bpm: 9 }),))
    );
    assert_eq!(
        beat("beat titi   : 90\n"),
        Ok((
            "",
            Exp::Beat(Beat {
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
            Exp::Pattern(Pattern {
                id: "p1",
                times: vec![0.5, 0.75],
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
            Exp::Velocities(Velocities {
                id: "v1",
                values: vec![0.5, 1., 0.75, 0.9],
            }),
        ))
    );
}

#[test]
fn test_notes() {
    assert_eq!(
        notes("notes blank :\n"),
        Ok((
            "",
            Exp::Notes(Notes {
                id: "blank",
                names: vec![]
            }),
        ))
    );
    assert_eq!(
        notes("notes intro : a0 G9  B7 \n"),
        Ok((
            "",
            Exp::Notes(Notes {
                id: "intro",
                names: vec!["a0", "G9", "B7"]
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
            Fragment::Part(Part {
                pattern: "p",
                notes: Some("n"),
                velos: Some("v"),
                mul: Some(3.),
            }),
        ))
    );
    assert_eq!(
        part("p.n * 3 "),
        Ok((
            "",
            Fragment::Part(Part {
                pattern: "p",
                notes: Some("n"),
                velos: None,
                mul: Some(3.),
            }),
        ))
    );
    assert_eq!(
        part("p.n.v0"),
        Ok((
            "",
            Fragment::Part(Part {
                pattern: "p",
                notes: Some("n"),
                velos: Some("v0"),
                mul: None,
            }),
        ))
    );
    assert_eq!(
        part("p1*3 "),
        Ok((
            "",
            Fragment::Part(Part {
                pattern: "p1",
                notes: None,
                velos: None,
                mul: Some(3.),
            }),
        ))
    );
    assert_eq!(
        part("p.n.v*3"),
        Ok((
            "",
            Fragment::Part(Part {
                pattern: "p",
                notes: Some("n"),
                velos: Some("v"),
                mul: Some(3.),
            }),
        ))
    );
    assert_eq!(
        part("4_p0f"),
        Ok((
            "",
            Fragment::Part(Part {
                pattern: "4_p0f",
                notes: None,
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
            Fragment::SeqRef(SeqRef {
                id: "s_01",
                mul: None
            }),
        ))
    );
    assert_eq!(
        seq_ref("$1sr_ *2.5"),
        Ok((
            "",
            Fragment::SeqRef(SeqRef {
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
            Sequence {
                id: "seq_03",
                beat: Some("_b_"),
                fragments: vec![
                    Fragment::SeqRef(SeqRef {
                        id: "s_1",
                        mul: None
                    }),
                    Fragment::Part(Part {
                        pattern: "p1",
                        notes: None,
                        velos: None,
                        mul: None,
                    }),
                    Fragment::Part(Part {
                        pattern: "p1",
                        notes: Some("n2"),
                        velos: None,
                        mul: None,
                    }),
                    Fragment::SeqRef(SeqRef {
                        id: "s_2",
                        mul: Some(3.)
                    }),
                    Fragment::Part(Part {
                        pattern: "p2",
                        notes: Some("n1"),
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
            Exp::Seq(Sequence {
                id: "s",
                beat: None,
                fragments: vec![Fragment::SeqRef(SeqRef {
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
            Exp::FreqOut(Sequence {
                id: "s",
                beat: None,
                fragments: vec![Fragment::SeqRef(SeqRef {
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
            Exp::Beat(Beat { id: "b", bpm: 90 }),
            Exp::Velocities(Velocities {
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
