use talker::ear::Set;
use talker::identifier::Index;
use crate::audio_data::Vector;

pub const KIND: &str = "track";

const INPUT_INDEX: Index = 0;
const GAIN_INDEX: Index = 1;
const CHANNEL_GAIN_INDEX: Index = 2;

fn compute_input_gain(set: &Set, _tick: i64, buf: &mut Vector, len: usize) -> usize {

    let in_buf = set.get_hum_audio_buffer(INPUT_INDEX);
    let gain_buf = set.get_hum_audio_buffer(GAIN_INDEX);

    for i in 0..len {
        buf[i] = in_buf[i] * gain_buf[i];
    }
    len
}

pub fn set(
    set: &Set,
    tick: i64,
    buf: &mut Vector,
    len: usize,
    channels: &mut Vec<Vector>,
) -> usize {
    let ln = compute_input_gain(set, tick, buf, len);

    for i in 0..channels.len() {
        let ch = &mut channels[i];
        let cg = set.get_hum_cv_buffer(CHANNEL_GAIN_INDEX + i);

        for j in 0..ln {
            ch[j] = cg[j] * buf[j];
        }
    }
    ln
}

pub fn add(
    set: &Set,
    tick: i64,
    buf: &mut Vector,
    len: usize,
    channels: &mut Vec<Vector>,
) -> usize {
    let ln = compute_input_gain(set, tick, buf, len);

    for i in 0..channels.len() {
        let ch = &mut channels[i];
        let cg = set.get_hum_cv_buffer(CHANNEL_GAIN_INDEX + i);

        for j in 0..ln {
            ch[j] = ch[j] + cg[j] * buf[j];
        }
    }

    ln
}
