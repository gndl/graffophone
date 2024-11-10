use talkers::tseq::audio_event;
use talkers::tseq::parser::PEnvelope;

pub const UNDEFINED: usize = usize::MAX;

pub fn create(penvelope: &PEnvelope, ticks_per_second: f32) -> Vec<f32> {
    let mut duration = 0.;
    let mut sections = Vec::with_capacity(penvelope.points.len());
    let mut start_level: f32 = 0.;

    for point in &penvelope.points {
        let end_tick = (point.duration * ticks_per_second) as i64;

        sections.push(audio_event::create(
            0,
            end_tick,
            start_level,
            point.level,
            point.shape,
            false,
            false,
            UNDEFINED,
        ));

        start_level = point.level;

        duration += point.duration;
    }

    let env_len = (duration * ticks_per_second) as usize;
    let mut envelop = Vec::with_capacity(env_len);
    envelop.resize(env_len, 0.);

    let buf = envelop.as_mut_slice();
    let mut ofset: usize = 0;
    let no_envelops = Vec::new();

    for section in sections {
        ofset += section.assign_buffer(&no_envelops, 0, buf, ofset, env_len - ofset) as usize;
    }
    envelop
}
