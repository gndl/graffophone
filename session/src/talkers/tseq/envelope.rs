use talkers::tseq::audio_event::{self, Shapes};
use talkers::tseq::parser::PShape;

pub const UNDEFINED: usize = usize::MAX;

pub struct Point {
    pub ticks: i64,
    pub shape: PShape,
    pub level: f32,
}
impl Point {
    pub fn new(ticks: i64, shape: PShape, level: f32) -> Point {
        Self { ticks, shape, level }
    }
}

pub fn create(shapes: &Shapes, points: &Vec<Point>) -> Vec<f32> {
    let mut duration = 0;
    let mut sections = Vec::with_capacity(points.len());
    let mut start_level: f32 = 0.;

    for point in points {
        sections.push(audio_event::create(
            shapes,
            0,
            point.ticks,
            start_level,
            point.level,
            point.shape,
            false,
            false,
            UNDEFINED,
        ));

        start_level = point.level;

        duration += point.ticks;
    }

    let env_len = duration as usize;
    let mut envelop = Vec::with_capacity(env_len);
    envelop.resize(env_len, 0.);

    let buf = envelop.as_mut_slice();
    let mut ofset: usize = 0;

    for section in sections {
        ofset += section.assign_buffer(shapes, 0, buf, ofset, env_len - ofset) as usize;
    }
    envelop
}
