use std::f32;
use std::f64;

use audio_format::AudioFormat;

pub fn audioize_buffer_by_clipping(buffer: &mut [f32], start: usize, len: usize) {
    for i in start..len {
        let v = buffer[i];

        if v < AudioFormat::MIN_AUDIO {
            buffer[i] = AudioFormat::MIN_AUDIO;
        } else if v > AudioFormat::MAX_AUDIO {
            buffer[i] = AudioFormat::MAX_AUDIO;
        }
    }
}


pub fn audioize_buffer_by_tanh(buffer: &mut [f32], start: usize, len: usize) {
    for i in start..len {
        buffer[i] = buffer[i].tanh();
    }
}

pub fn audioize_buffer_by_atan(buffer: &mut [f32], start: usize, len: usize) {
    for i in start..len {
        buffer[i] = buffer[i].atan();
    }
}

pub fn fade_len(sample_rate: usize) -> usize {
    sample_rate / 100
}

fn fade_from_value_buffer(buffer: &mut [f32], start: usize, len: usize, value: f32) {
    let step = 1. / len as f32;
    let mut c = 0.;

    for i in start..(start + len) {
        buffer[i] = value * (1. - c) + buffer[i] * c;
        c += step;
    }
}

pub fn recoveryless_fade_buffer(sample_rate: usize, buffer: &mut [f32], start: usize, vm2: f32, vm1: f32) {
    let dt = 1. / sample_rate as f64;

    let v1 = vm2 as f64;
    let mut v2 = vm1 as f64;
    let mut g = (v2 - v1) / dt;

    let mut idx = start;

    while g > 0.25 || g < -0.25 {
        v2 += g * dt;

        if v2 > 1. {
            break;
        }
        buffer[idx] = v2 as f32;
        g *= 0.9;
        idx += 1;
    }
    let remaining_len = fade_len(sample_rate) - (idx - start);

    fade_from_value_buffer(buffer, idx, remaining_len, v2 as f32);
}
