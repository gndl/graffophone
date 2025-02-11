use nom::{
    branch::alt,
    bytes::complete::{tag, take_until},
    character::complete::{alphanumeric1, char, digit1, newline, one_of, space0, space1},
    combinator::{map_res, opt, recognize},
    multi::{many0, many1_count},
    number::complete::float,
    sequence::{delimited, preceded, terminated, tuple},
    IResult,
};

use std::str::FromStr;

use ASSIGNMENT_KW;
use BEAT_KW;
use SCALE_KW;
use ATTRIBUTE_KW;
use COUPLING_KW;
use DEF_KW;
use DURATIONLINE_KW;
use EARLY_SHAPE_KW;
use ENVELOP_KW;
use HITLINE_KW;
use JOIN_KW;
use LATE_SHAPE_KW;
use LINEAR_SHAPE_KW;
use LINE_COMMENT_KW;
use MIDI_OUTPUT_KW;
use MULTILINE_COMMENT_KW;
use MUL_KW;
use ON_KW;
use PER_KW;
use PITCHLINE_KW;
use REF_KW;
use ROUND_SHAPE_KW;
use SECOND_SYM_KW;
use SEQUENCE_KW;
use SEQUENCE_OUTPUT_KW;
use SIN_SHAPE_KW;
use VELOCITYLINE_KW;
use {ATTACK_KW, CHORDLINE_KW, CHORD_KW, INTERVAL_KW};
use {CLOSE_PARENT_KW, OPEN_PARENT_KW};
use {OPEN_BRACKET_KW, CLOSE_BRACKET_KW, PARAM_SEP_KW, NOTE_SHIFT_KW, BACK_NOTE_SHIFT_KW, PITCH_TRANSPO_KW, PITCH_INV_KW};
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
pub struct PAttribute<'a> {
    pub label: &'a str,
    pub value: &'a str,
}

#[derive(Debug, PartialEq)]
pub struct PRef<'a> {
    pub id: &'a str,
    pub mul: usize,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum PShape {
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
    pub bpm: f32,
}

#[derive(Debug, PartialEq)]
pub struct PScale<'a> {
    pub id: &'a str,
    pub name: &'a str,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum PPitchGap {
    FreqRatio(f32),
    Interval(i32),
}

#[derive(Debug, PartialEq)]
pub struct PHarmonic<'a> {
    pub pitch_gap: PPitchGap,
    pub delay: Option<PTime>,
    pub velocity: Option<PVelocity<'a>>,
}

#[derive(Debug, PartialEq)]
pub struct PChord<'a> {
    pub id: &'a str,
    pub harmonics: Vec<PHarmonic<'a>>,
}

#[derive(Debug, PartialEq)]
pub struct PAccent<'a> {
    pub delay: PTime,
    pub velocity: Option<PVelocity<'a>>,
}

#[derive(Debug, PartialEq)]
pub struct PAttack<'a> {
    pub id: &'a str,
    pub accents: Vec<PAccent<'a>>,
}

#[derive(Debug, PartialEq)]
pub struct PChordLinePart<'a> {
    pub chord_id: &'a str,
    pub attack_id: Option<&'a str>,
}

#[derive(Debug, PartialEq)]
pub enum PChordLineFragment<'a> {
    Part((PChordLinePart<'a>, usize)),
    Ref(PRef<'a>),
    Fragments((Vec<PChordLineFragment<'a>>, usize)),
}

#[derive(Debug, PartialEq)]
pub struct PChordLine<'a> {
    pub id: &'a str,
    pub fragments: Vec<PChordLineFragment<'a>>,
}


#[derive(Debug, PartialEq, Clone, Copy)]
pub struct PPitch<'a> {
    pub id: &'a str,
    pub transition: PShape,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum PPitchLineTransformation<'a> {
    NoteShift(usize),
    BackwardNoteShift(usize),
    PitchTranspo(&'a str, &'a str),
    PitchInv,
}

#[derive(Debug, PartialEq)]
pub enum PPitchLineFragment<'a> {
    Part((PPitch<'a>, usize)),
    Ref((PRef<'a>, Option<Vec<PPitchLineTransformation<'a>>>)),
    Fragments((Vec<PPitchLineFragment<'a>>, usize)),
}

#[derive(Debug, PartialEq)]
pub struct PPitchLine<'a> {
    pub id: &'a str,
    pub scale: Option<&'a str>,
    pub fragments: Vec<PPitchLineFragment<'a>>,
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
pub struct PEnvelopePoint {
    pub duration: f32,
    pub shape: PShape,
    pub level: f32,
}

#[derive(Debug, PartialEq)]
pub struct PEnvelope<'a> {
    pub id: &'a str,
    pub points: Vec<PEnvelopePoint>,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct PVelocity<'a> {
    pub level: f32,
    pub envelope_id: Option<&'a str>,
    pub fadein: bool,
    pub fadeout: bool,
    pub transition: PShape,
}

#[derive(Debug, PartialEq)]
pub enum PVelocityLineFragment<'a> {
    Part((PVelocity<'a>, usize)),
    Ref(PRef<'a>),
    Fragments((Vec<PVelocityLineFragment<'a>>, usize)),
}

#[derive(Debug, PartialEq)]
pub struct PVelocityLine<'a> {
    pub id: &'a str,
    pub fragments: Vec<PVelocityLineFragment<'a>>,
}

#[derive(Debug, PartialEq)]
pub struct PSeqPart<'a> {
    pub hitline_id: &'a str,
    pub durationline_id: Option<&'a str>,
    pub pitchline_id: Option<&'a str>,
    pub chordline_id: Option<&'a str>,
    pub velocityline_id: Option<&'a str>,
    pub mul: f32,
}

#[derive(Debug, PartialEq)]
pub enum PSeqFragment<'a> {
    Part(PSeqPart<'a>),
    Ref(PRef<'a>),
    Fragments((Vec<PSeqFragment<'a>>, usize)),
}

#[derive(Debug, PartialEq)]
pub struct PSequence<'a> {
    pub id: &'a str,
    pub beat: Option<&'a str>,
    pub envelope_id: Option<&'a str>,
    pub fragments: Vec<PSeqFragment<'a>>,
}

#[derive(Debug, PartialEq)]
pub struct PMidiChannel<'a> {
    pub seq_id: &'a str,
    pub program: u8,
    pub attributes: Vec<PAttribute<'a>>,
}

#[derive(Debug, PartialEq)]
pub struct PMidiSequence<'a> {
    pub id: &'a str,
    pub channels: Vec<PMidiChannel<'a>>,
}

#[derive(Debug, PartialEq)]
pub enum Expression<'a> {
    Beat(PBeat<'a>),
    Scale(PScale<'a>),
    Chord(PChord<'a>),
    Attack(PAttack<'a>),
    ChordLine(PChordLine<'a>),
    DurationLine(PDurationLine<'a>),
    VelocityLine(PVelocityLine<'a>),
    Envelope(PEnvelope<'a>),
    HitLine(PHitLine<'a>),
    PitchLine(PPitchLine<'a>),
    Seq(PSequence<'a>),
    SeqOut(PSequence<'a>),
    MidiOut(PMidiSequence<'a>),
    None,
}

fn uint(input: &str) -> IResult<&str, usize> {
    map_res(terminated(digit1, space0), str::parse)(input)
}

pub fn id(input: &str) -> IResult<&str, &str> {
    terminated( recognize(many1_count(alt((alphanumeric1, tag("_"))))), space0)(input)
}

fn head<'a>(inst: &'a str) -> impl FnMut(&'a str) -> IResult<&'a str, &'a str> {
    delimited(
        preceded(tag(inst), space1),
        id,
        terminated(char(DEF_KW!()), space0),
    )
}

fn on(input: &str) -> IResult<&str, char> {
    delimited(space0, char(ON_KW!()), space0)(input)
}

fn per(input: &str) -> IResult<&str, char> {
    terminated(char(PER_KW!()), space0)(input)
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
            bpm: f32::from_str(bpm).unwrap(),
        }),
    ))
}

fn scale(input: &str) -> IResult<&str, Expression> {
    let (input, (id, name, _)) = tuple((head(SCALE_KW!()), id, end))(input)?;
    Ok((
        input,
        Expression::Scale(PScale {
            id,
            name,
        }),
    ))
}

fn attribute(input: &str) -> IResult<&str, PAttribute> {
    let (input, (_, _, label, _, _, value)) =
        tuple((char(ATTRIBUTE_KW!()), space0, id, char(ASSIGNMENT_KW!()), space0, id))(input)?;
    Ok((input, PAttribute { label, value }))
}

fn ratio(input: &str) -> IResult<&str, PRatio> {
    let (input, (num, den, _)) = tuple((float, opt(preceded(on, float)), space0))(input)?;
    Ok((input, PRatio {num, den: den.unwrap_or(1.)}))
}

fn freq_ratio(input: &str) -> IResult<&str, PPitchGap> {
    let (input, freq_ratio) = ratio(input)?;
    Ok((input, PPitchGap::FreqRatio(freq_ratio.num / freq_ratio.den)))
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

fn chordline_part(input: &str) -> IResult<&str, PChordLineFragment> {
    let (input, (chord_id, attack_id, omul)) = tuple((
        id,
        opt(preceded(char(JOIN_KW!()), id)),
        opt(preceded(terminated(char(MUL_KW!()), space0), uint)),
     ))(input)?;
    Ok((
        input,
        PChordLineFragment::Part((PChordLinePart {
            chord_id,
            attack_id,
        },
        omul.unwrap_or(1))),
    ))
}

fn chordline_ref(input: &str) -> IResult<&str, PChordLineFragment> {
    let (input, (id, omul)) = tuple((
        preceded(char(REF_KW!()), id),
        opt(preceded(terminated(char(MUL_KW!()), space0), uint)),
    ))(input)?;

    Ok((input, PChordLineFragment::Ref(PRef{id, mul: omul.unwrap_or(1)})))
}

fn chordline_fragments(input: &str) -> IResult<&str, PChordLineFragment> {
    let (input, (fragments, mul)) = tuple((
        delimited(
            terminated(char(OPEN_PARENT_KW!()), space0),
            many0(alt((chordline_part, chordline_ref, chordline_fragments))),
            terminated(char(CLOSE_PARENT_KW!()), space0),
        ),
        preceded(terminated(char(MUL_KW!()), space0), uint),
    ))(input)?;
    Ok((
        input,
        PChordLineFragment::Fragments((fragments, mul)),
    ))
}

fn chordline(input: &str) -> IResult<&str, Expression> {
    let (input, (id, fragments, _)) = tuple((
        head(CHORDLINE_KW!()),
        many0(alt((chordline_part, chordline_ref, chordline_fragments))),
        end,
    ))(input)?;

    Ok((input, Expression::ChordLine(PChordLine {id, fragments})))
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

fn shape(input: &str) -> IResult<&str, PShape> {
    let (input, oprog) = delimited(space0, opt(one_of("=~<>°")), space0)(input)?;

    let shape = match oprog {
        Some(c) => match c {
            LINEAR_SHAPE_KW!() => PShape::Linear,
            SIN_SHAPE_KW!() => PShape::Sin,
            EARLY_SHAPE_KW!() => PShape::Early,
            LATE_SHAPE_KW!() => PShape::Late,
            ROUND_SHAPE_KW!() => PShape::Round,
            _ => PShape::None,
        },
        None => PShape::None,
    };

    Ok((input, shape))
}

fn velocity(input: &str) -> IResult<&str, PVelocity> {
    let (input, (ofadein, level, envelope_id, ofadeout, transition)) = tuple((
        opt(tag(FADEIN_KW!())),
        ratio,
        opt(preceded(char(COUPLING_KW!()), id)),
        opt(tag(FADEOUT_KW!())),
        shape,
    ))(input)?;
    Ok((
        input,
        PVelocity {
            level: level.num / level.den,
            envelope_id,
            fadein: ofadein.is_some(),
            fadeout: ofadeout.is_some(),
            transition,
        },
    ))
}

fn velocityline_part(input: &str) -> IResult<&str, PVelocityLineFragment> {
    let (input, (velocity, omul)) = tuple((
        velocity,
        opt(preceded(terminated(char(MUL_KW!()), space0), uint)),
    ))(input)?;
    Ok((input, PVelocityLineFragment::Part((velocity, omul.unwrap_or(1)))))
}

fn velocityline_ref(input: &str) -> IResult<&str, PVelocityLineFragment> {
    let (input, (id, omul)) = tuple((
        preceded(char(REF_KW!()), id),
        opt(preceded(terminated(char(MUL_KW!()), space0), uint)),
    ))(input)?;

    Ok((input, PVelocityLineFragment::Ref(PRef{id, mul: omul.unwrap_or(1)})))
}

fn velocityline_fragments(input: &str) -> IResult<&str, PVelocityLineFragment> {
    let (input, (fragments, mul)) = tuple((
        delimited(
            terminated(char(OPEN_PARENT_KW!()), space0),
            many0(alt((velocityline_part, velocityline_ref, velocityline_fragments))),
            terminated(char(CLOSE_PARENT_KW!()), space0),
        ),
        preceded(terminated(char(MUL_KW!()), space0), uint),
    ))(input)?;
    Ok((
        input,
        PVelocityLineFragment::Fragments((fragments, mul)),
    ))
}

fn velocityline(input: &str) -> IResult<&str, Expression> {
    let (input, (id, fragments, _)) = tuple((
        head(VELOCITYLINE_KW!()),
        many0(alt((velocityline_part, velocityline_ref, velocityline_fragments))),
        end,
    ))(input)?;
    Ok((
        input,
        Expression::VelocityLine(PVelocityLine { id, fragments }),
    ))
}

fn envelop_point(input: &str) -> IResult<&str, PEnvelopePoint> {
    let (input, (duration, shape, level, _)) = tuple((ratio, shape, ratio, space0))(input)?;
    Ok((
        input,
        PEnvelopePoint {
            duration: duration.num / duration.den,
            shape,
            level: level.num / level.den,
        },
    ))
}

fn envelope(input: &str) -> IResult<&str, Expression> {
    let (input, (id, points, _)) = tuple((head(ENVELOP_KW!()), many0(envelop_point), end))(input)?;
    Ok((input, Expression::Envelope(PEnvelope { id, points })))
}


pub fn pitch_id(input: &str) -> IResult<&str, &str> {
    recognize(many1_count(alt((alphanumeric1, tag("#"), tag("."), tag("^") ) ) ) ) (input)
}

fn pitch(input: &str) -> IResult<&str, PPitchLineFragment> {
    let (input, (id, transition, omul)) = tuple((
        recognize(pitch_id),
        shape,
        opt(preceded(terminated(char(MUL_KW!()), space0), uint)),
    ))(input)?;
    Ok((input, PPitchLineFragment::Part((PPitch {id, transition}, omul.unwrap_or(1)))))
}

fn note_shift_transformation(input: &str) -> IResult<&str, PPitchLineTransformation> {
    let (input, (_, _, oshift_count)) = tuple((tag(NOTE_SHIFT_KW!()), space0, opt(digit1)))(input)?;
    let shift_count = oshift_count.map_or(0, |s| usize::from_str(s).unwrap());
    Ok((input, PPitchLineTransformation::NoteShift(shift_count ) ) )
}

fn backward_note_shift_transformation(input: &str) -> IResult<&str, PPitchLineTransformation> {
    let (input, (_, _, oshift_count)) = tuple((tag(BACK_NOTE_SHIFT_KW!()), space0, opt(digit1)))(input)?;
    let shift_count = oshift_count.map_or(0, |s| usize::from_str(s).unwrap());
    Ok((input, PPitchLineTransformation::BackwardNoteShift(shift_count) ) )
}

fn pitch_transpo_transformation(input: &str) -> IResult<&str, PPitchLineTransformation> {
    let (input, (pa, _, _, _, pb)) = tuple((pitch_id, space1, tag(PITCH_TRANSPO_KW!()), space1, pitch_id))(input)?;
    Ok((input, PPitchLineTransformation::PitchTranspo(pa, pb) ) )
}

fn pitch_inv_transformation(input: &str) -> IResult<&str, PPitchLineTransformation> {
    let (input, _) = tag(PITCH_INV_KW!()) (input)?;
    Ok((input, PPitchLineTransformation::PitchInv))
}

fn pitchline_transformation(input: &str) -> IResult<&str, PPitchLineTransformation> {
    let (input, (_, transfo, _, _, _)) = tuple((space0,
        alt((note_shift_transformation, backward_note_shift_transformation, pitch_transpo_transformation, pitch_inv_transformation)),
        space0,
        opt(char(PARAM_SEP_KW!())),
        space0,
    )) (input)?;
    Ok((input, transfo))
}

fn pitchline_ref(input: &str) -> IResult<&str, PPitchLineFragment> {
    let (input, (id, transformations, _, omul)) = tuple((
        preceded(char(REF_KW!()), id),
        opt(delimited(char(OPEN_BRACKET_KW!()), many0(pitchline_transformation), char(CLOSE_BRACKET_KW!()))),
        space0,
        opt(preceded(terminated(char(MUL_KW!()), space0), uint)),
    ))(input)?;
    let mul = omul.unwrap_or(1);
    Ok((
        input,
        PPitchLineFragment::Ref((PRef {id, mul}, transformations)),
    ))
}

fn pitchline_fragments(input: &str) -> IResult<&str, PPitchLineFragment> {
    let (input, (fragments, mul)) = tuple((
        delimited(
            terminated(char(OPEN_PARENT_KW!()), space0),
            many0(alt((pitch, pitchline_ref, pitchline_fragments))),
            terminated(char(CLOSE_PARENT_KW!()), space0),
        ),
        preceded(terminated(char(MUL_KW!()), space0), uint),
    ))(input)?;
    Ok((
        input,
        PPitchLineFragment::Fragments((fragments, mul)),
    ))
}

fn pitchline(input: &str) -> IResult<&str, Expression> {
    let (input, (id, attributes, fragments, _, _)) =
        tuple((head(PITCHLINE_KW!()),
        many0(attribute),
        many0(alt((pitch, pitchline_ref, pitchline_fragments))),
        space0,
        end,
    ))(input)?;

    let mut scale = None;

    for attribute in attributes {
        if attribute.label == SCALE_KW!() {
            scale = Some(attribute.value);
        }
    }
    Ok((input, Expression::PitchLine(PPitchLine { id, scale, fragments })))
}


fn seq_part(input: &str) -> IResult<&str, PSeqFragment> {
    let (input, (hitline_id, durationline_id, pitchline_id, chordline_id, velocityline_id, omul, _)) =
        tuple((
            id,
            opt(preceded(char(COUPLING_KW!()), id)),
            opt(preceded(char(JOIN_KW!()), id)),
            opt(preceded(char(COUPLING_KW!()), id)),
            opt(preceded(char(JOIN_KW!()), id)),
            opt(preceded(terminated(char(MUL_KW!()), space0), float)),
            space0,
        ))(input)?;
    Ok((
        input,
        PSeqFragment::Part(PSeqPart {
            hitline_id,
            durationline_id,
            pitchline_id,
            chordline_id,
            velocityline_id,
            mul: omul.unwrap_or(1.),
        }),
    ))
}

fn seq_ref(input: &str) -> IResult<&str, PSeqFragment> {
    let (input, (id, omul)) = tuple((
        preceded(char(REF_KW!()), id),
        opt(preceded(terminated(char(MUL_KW!()), space0), uint)),
    ))(input)?;
    
    Ok((input, PSeqFragment::Ref(PRef{id, mul: omul.unwrap_or(1)})))
}

fn seq_fragments(input: &str) -> IResult<&str, PSeqFragment> {
    let (input, (fragments, mul)) = tuple((
        delimited(
            terminated(char(OPEN_PARENT_KW!()), space0),
            many0(alt((seq_part, seq_ref, seq_fragments))),
            terminated(char(CLOSE_PARENT_KW!()), space0),
        ),
        preceded(terminated(char(MUL_KW!()), space0), uint),
    ))(input)?;
    Ok((
        input,
        PSeqFragment::Fragments((fragments, mul)),
    ))
}

fn sequence(input: &str) -> IResult<&str, PSequence> {
    let (input, (_, id, _, attributes, fragments, _)) = tuple((
        space1,
        id,
        terminated(char(DEF_KW!()), space0),
        many0(attribute),
        many0(alt((seq_part, seq_ref, seq_fragments))),
        end,
    ))(input)?;
    let mut beat = None;
    let mut envelope_id = None;

    for attribute in attributes {
        if attribute.label == BEAT_KW!() {
            beat = Some(attribute.value);
        } else if attribute.label == ENVELOP_KW!() {
            envelope_id = Some(attribute.value);
        }
    }
    Ok((
        input,
        PSequence {
            id,
            beat,
            envelope_id,
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

fn midi_channel(input: &str) -> IResult<&str, PMidiChannel> {
    let (input, (seq_id, program, attributes, _)) = tuple((
        preceded(char(REF_KW!()), id),
        preceded(char(JOIN_KW!()), digit1),
        many0(attribute),
        space0,
    ))(input)?;
    Ok((
        input,
        PMidiChannel {
            seq_id,
            program: u8::from_str(program).unwrap(),
            attributes,
        },
    ))
}

fn midiout(input: &str) -> IResult<&str, Expression> {
    let (input, (id, _attributes, channels, _)) = tuple((
        head(MIDI_OUTPUT_KW!()),
        many0(attribute),
        many0(midi_channel),
        end,
    ))(input)?;
    Ok((input, Expression::MidiOut(PMidiSequence {id, channels})))
}

pub fn parse(input: &str) -> Result<Vec<Expression>, failure::Error> {
    let (input, expressions) = many0(alt((
        beat,
        scale,
        chord,
        attack,
        chordline,
        hits,
        durations,
        pitchline,
        velocityline,
        envelope,
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
        Ok(("", Expression::Beat(PBeat { id: "Id06", bpm: 9. }),))
    );
    assert_eq!(
        beat(concat!(BEAT_KW!(), "  9zZ", DEF_KW!(), "9  \n")),
        Ok(("", Expression::Beat(PBeat { id: "9zZ", bpm: 9. }),))
    );
    assert_eq!(
        beat(concat!(BEAT_KW!(), " titi   ", DEF_KW!(), " 90\n")),
        Ok((
            "",
            Expression::Beat(PBeat {
                id: "titi",
                bpm: 90.,
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
                        pitch_gap: PPitchGap::FreqRatio(1.),
                        delay: None,
                        velocity: None,
                    },
                    PHarmonic {
                        pitch_gap: PPitchGap::FreqRatio(1.5),
                        delay: Some(PTime::Second(PRatio { num: 2., den: 1. })),
                        velocity: None,
                    },
                    PHarmonic {
                        pitch_gap: PPitchGap::FreqRatio(1.5),
                        delay: Some(PTime::Rate(PRatio { num: 1., den: 2. })),
                        velocity: Some(PVelocity {
                            level: 0.4,
                            envelope_id: None,
                            fadein: false,
                            fadeout: false,
                            transition: PShape::None
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
fn test_chordline() {
    assert_eq!(
        chordline(concat!(
            CHORDLINE_KW!(),
            " cs ",
            DEF_KW!(),
            " c1 c2-a c3 c1 * 2 c2-a * 2 @id ( c1*2 c2-a ( @id c1 c2-a * 2)*3) *4 @id * 5 \n"
        )),
        Ok((
            "",
            Expression::ChordLine(PChordLine {
                id: "cs",
                fragments: vec![
                    PChordLineFragment::Part((PChordLinePart {
                        chord_id: "c1",
                        attack_id: None,
                    }, 1)),
                    PChordLineFragment::Part((PChordLinePart {
                        chord_id: "c2",
                        attack_id: Some("a"),
                    }, 1)),
                    PChordLineFragment::Part((PChordLinePart {
                        chord_id: "c3",
                        attack_id: None,
                    }, 1)),
                    PChordLineFragment::Part((PChordLinePart {
                        chord_id: "c1",
                        attack_id: None,
                    }, 2)),
                    PChordLineFragment::Part((PChordLinePart {
                        chord_id: "c2",
                        attack_id: Some("a"),
                    }, 2)),
                    PChordLineFragment::Ref(PRef {
                        id: "id",
                        mul: 1,
                    }),
                    PChordLineFragment::Fragments((vec![
                        PChordLineFragment::Part((PChordLinePart {
                            chord_id: "c1",
                            attack_id: None,
                        }, 2)),
                        PChordLineFragment::Part((PChordLinePart {
                            chord_id: "c2",
                            attack_id: Some("a"),
                        }, 1)),
                        PChordLineFragment::Fragments((vec![
                            PChordLineFragment::Ref(PRef {
                                id: "id",
                                mul: 1,
                            }),
                            PChordLineFragment::Part((PChordLinePart {
                                chord_id: "c1",
                                attack_id: None,
                            }, 1)),
                            PChordLineFragment::Part((PChordLinePart {
                                chord_id: "c2",
                                attack_id: Some("a"),
                            }, 2)),
                        ], 3)),
                    ], 4)),
                    PChordLineFragment::Ref(PRef {
                        id: "id",
                        mul: 5,
                    }),
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
        velocityline(concat!(
            VELOCITYLINE_KW!(),
            " v1",
            DEF_KW!(),
            " _/1/2\\_ ~ * 7 _/1 .75\\_=0.9 1&env * 9 .6&e \n"
        )),
        Ok((
            "",
            Expression::VelocityLine(PVelocityLine {
                id: "v1",
                fragments: vec![
                    PVelocityLineFragment::Part((PVelocity {
                        level: 0.5,
                        envelope_id: None,
                        fadein: true,
                        fadeout: true,
                        transition: PShape::Sin
                    }, 7)),
                    PVelocityLineFragment::Part((PVelocity {
                        level: 1.,
                        envelope_id: None,
                        fadein: true,
                        fadeout: false,
                        transition: PShape::None
                    }, 1)),
                    PVelocityLineFragment::Part((PVelocity {
                        level: 0.75,
                        envelope_id: None,
                        fadein: false,
                        fadeout: true,
                        transition: PShape::Linear
                    }, 1)),
                    PVelocityLineFragment::Part((PVelocity {
                        level: 0.9,
                        envelope_id: None,
                        fadein: false,
                        fadeout: false,
                        transition: PShape::None
                    }, 1)),
                    PVelocityLineFragment::Part((PVelocity {
                        level: 1.,
                        envelope_id: Some("env"),
                        fadein: false,
                        fadeout: false,
                        transition: PShape::None
                    }, 9)),
                    PVelocityLineFragment::Part((PVelocity {
                        level: 0.6,
                        envelope_id: Some("e"),
                        fadein: false,
                        fadeout: false,
                        transition: PShape::None
                    }, 1)),
                ],
            }),
        ))
    );
}

#[test]
fn test_envelop() {
    assert_eq!(
        envelope(concat!(
            ENVELOP_KW!(),
            " e",
            DEF_KW!(),
            " .02<1 .2~2/3 2>1/3 20°0\n"
        )),
        Ok((
            "",
            Expression::Envelope(PEnvelope {
                id: "e",
                points: vec![
                    PEnvelopePoint {
                        duration: 0.02,
                        shape: PShape::Early,
                        level: 1.
                    },
                    PEnvelopePoint {
                        duration: 0.2,
                        shape: PShape::Sin,
                        level: 0.6666667
                    },
                    PEnvelopePoint {
                        duration: 2.,
                        shape: PShape::Late,
                        level: 0.33333334
                    },
                    PEnvelopePoint {
                        duration: 20.,
                        shape: PShape::Round,
                        level: 0.
                    },
                ],
            }),
        ))
    );
}

#[test]
fn test_pitchs() {
    assert_eq!(
        pitchline(concat!(PITCHLINE_KW!(), " blank ", DEF_KW!(), ATTRIBUTE_KW!(), SCALE_KW!(), ASSIGNMENT_KW!(), "midi\n")),
        Ok((
            "",
            Expression::PitchLine(PPitchLine {
                id: "blank",
                scale: Some("midi"),
                fragments: vec![]
            }),
        ))
    );
    assert_eq!(
        pitchline(concat!(
            PITCHLINE_KW!(),
            " intro ",
            DEF_KW!(),
            " G9 = B7~e5 > f2 <a0  \n"
        )),
        Ok((
            "",
            Expression::PitchLine(PPitchLine {
                id: "intro",
                scale: None,
                fragments: vec![
                    PPitchLineFragment::Part((PPitch {
                        id: "G9",
                        transition: PShape::Linear,
                    }, 1)),
                    PPitchLineFragment::Part((PPitch {
                        id: "B7",
                        transition: PShape::Sin,
                    }, 1)),
                    PPitchLineFragment::Part((PPitch {
                        id: "e5",
                        transition: PShape::Late,
                    }, 1)),
                    PPitchLineFragment::Part((PPitch {
                        id: "f2",
                        transition: PShape::Early,
                    }, 1)),
                    PPitchLineFragment::Part((PPitch {
                        id: "a0",
                        transition: PShape::None,
                    }, 1)),
                ]
            }),
        ))
    );
}

#[test]
fn test_part() {
    assert_eq!(
        seq_part(concat!(
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
            PSeqFragment::Part(PSeqPart {
                hitline_id: "p",
                durationline_id: None,
                pitchline_id: Some("n"),
                chordline_id: None,
                velocityline_id: Some("v"),
                mul: 3.,
            }),
        ))
    );
    assert_eq!(
        seq_part(concat!("p", JOIN_KW!(), "n ", MUL_KW!(), " 3 ")),
        Ok((
            "",
            PSeqFragment::Part(PSeqPart {
                hitline_id: "p",
                durationline_id: None,
                pitchline_id: Some("n"),
                chordline_id: None,
                velocityline_id: None,
                mul: 3.,
            }),
        ))
    );
    assert_eq!(
        seq_part(concat!("p", JOIN_KW!(), "n", JOIN_KW!(), "v0")),
        Ok((
            "",
            PSeqFragment::Part(PSeqPart {
                hitline_id: "p",
                durationline_id: None,
                pitchline_id: Some("n"),
                chordline_id: None,
                velocityline_id: Some("v0"),
                mul: 1.,
            }),
        ))
    );
    assert_eq!(
        seq_part(concat!("p1", MUL_KW!(), "3 ")),
        Ok((
            "",
            PSeqFragment::Part(PSeqPart {
                hitline_id: "p1",
                durationline_id: None,
                pitchline_id: None,
                chordline_id: None,
                velocityline_id: None,
                mul: 3.,
            }),
        ))
    );
    assert_eq!(
        seq_part(concat!(
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
            PSeqFragment::Part(PSeqPart {
                hitline_id: "p",
                durationline_id: None,
                pitchline_id: Some("n"),
                chordline_id: None,
                velocityline_id: Some("v"),
                mul: 3.,
            }),
        ))
    );
    assert_eq!(
        seq_part("4_p0f"),
        Ok((
            "",
            PSeqFragment::Part(PSeqPart {
                hitline_id: "4_p0f",
                durationline_id: None,
                pitchline_id: None,
                chordline_id: None,
                velocityline_id: None,
                mul: 1.,
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
            PSeqFragment::Ref(PRef {
                id: "s_01",
                mul: 1
            }),
        ))
    );
    assert_eq!(
        seq_ref(concat!(REF_KW!(), "1sr_ ", MUL_KW!(), "2")),
        Ok((
            "",
            PSeqFragment::Ref(PRef {
                id: "1sr_",
                mul: 2
            }),
        ))
    );
}

#[test]
fn test_fragments() {
    assert_eq!(
        seq_fragments(concat!(
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
            PSeqFragment::Fragments((
                vec![
                    PSeqFragment::Part(PSeqPart {
                        hitline_id: "p",
                        durationline_id: None,
                        pitchline_id: None,
                        chordline_id: None,
                        velocityline_id: None,
                        mul: 1.,
                    }),
                    PSeqFragment::Ref(PRef { id: "s", mul: 1 })
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
            ATTRIBUTE_KW!(),
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
                envelope_id: None,
                fragments: vec![
                    PSeqFragment::Ref(PRef {
                        id: "s_1",
                        mul: 1
                    }),
                    PSeqFragment::Part(PSeqPart {
                        hitline_id: "p1",
                        durationline_id: None,
                        pitchline_id: None,
                        chordline_id: None,
                        velocityline_id: None,
                        mul: 1.,
                    }),
                    PSeqFragment::Part(PSeqPart {
                        hitline_id: "p1",
                        durationline_id: None,
                        pitchline_id: Some("n2"),
                        chordline_id: None,
                        velocityline_id: None,
                        mul: 1.,
                    }),
                    PSeqFragment::Ref(PRef {
                        id: "s_2",
                        mul: 3
                    }),
                    PSeqFragment::Part(PSeqPart {
                        hitline_id: "p2",
                        durationline_id: None,
                        pitchline_id: Some("n1"),
                        chordline_id: None,
                        velocityline_id: Some("v1"),
                        mul: 2.,
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
                envelope_id: None,
                fragments: vec![PSeqFragment::Ref(PRef {
                    id: "s_1",
                    mul: 1
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
                envelope_id: None,
                fragments: vec![PSeqFragment::Ref(PRef {
                    id: "s_1",
                    mul: 1
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
            Expression::Beat(PBeat { id: "b", bpm: 90. }),
            Expression::VelocityLine(PVelocityLine {
                id: "v",
                fragments: vec![PVelocityLineFragment::Part((PVelocity {
                    envelope_id: None,
                    level: 1.,
                    transition: PShape::None,
                    fadein: false,
                    fadeout: false
                }, 1))],
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
