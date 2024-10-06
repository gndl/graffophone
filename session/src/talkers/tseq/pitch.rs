use std::{collections::{HashMap, HashSet}, usize};

use scale::scale::{self, Scale};

use talkers::tseq::parser::{PPitchLineFragment, PPitchLine, PPitch, PPitchLineTransformation, PShape};

#[derive(Debug, PartialEq, Clone)]
pub struct Pitch {
    pub id: String,
    pub transition: PShape,
}
impl Pitch {
    pub fn new(pitch: &Pitch) -> Pitch {
        Self {
            id: pitch.id.clone(),
            transition: pitch.transition,
        }
    }

    pub fn from_parser_pitch(ppitch: &PPitch) -> Pitch {
        Self {
            id: String::from(ppitch.id),
            transition: ppitch.transition,
        }
    }
}

pub fn notes_shift(pitchs: &mut Vec<Pitch>, shift_count: usize) -> Result<(), failure::Error> {

    if shift_count >= pitchs.len() {
        return Err(failure::err_msg(format!("Tseq pitchline notes shift count {} invalide!", shift_count)));
    }
    pitchs.rotate_left(shift_count);
    Ok(())
}

pub fn backward_notes_shift(pitchs: &mut Vec<Pitch>, shift_count: usize) -> Result<(), failure::Error> {

    if shift_count >= pitchs.len() {
        return Err(failure::err_msg(format!("Tseq pitchline notes shift count {} invalide!", shift_count)));
    }
    pitchs.rotate_left(shift_count);
    pitchs.reverse();
    Ok(())
}

pub fn pitchs_transposition(pitchs: &mut Vec<Pitch>, scale: &Scale, initial_pitch: &str, final_pitch: &str) -> Result<(), failure::Error> {

    let init = scale.pitch_name_to_number(initial_pitch)? as i64;
    let fin = scale.pitch_name_to_number(final_pitch)? as i64;
    let dn = fin - init;

    for pitch in pitchs {
        let new_number = (scale.pitch_name_to_number(&pitch.id)? as i64 + dn) as usize;
        pitch.id = scale.pitch_number_to_name(new_number);
    }
    Ok(())
}

pub fn pitchs_inversion(pitchs: &mut Vec<Pitch>, scale: &Scale) -> Result<(), failure::Error> {

    let pitchs_count = pitchs.len();
    let mut n_min = usize::MAX;
    let mut n_max = usize::MIN;
    let mut nums = Vec::with_capacity(pitchs_count);

    for pitch in &mut *pitchs {
        let num = scale.pitch_name_to_number(&pitch.id)?;
        n_min = n_min.min(num);
        n_max = n_max.max(num);
        nums.push(num);
    }
    let min_plus_max = n_min + n_max;

    for i in 0..pitchs_count {
        let new_number = min_plus_max - nums[i];
        pitchs[i].id = scale.pitch_number_to_name(new_number);
    }
    Ok(())
}
