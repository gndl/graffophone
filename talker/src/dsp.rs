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
