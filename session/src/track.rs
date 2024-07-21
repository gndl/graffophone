use std::cell::RefCell;
use std::rc::Rc;

use talker::ear::Set;
use talker::identifier::Index;
use talker::talker::TalkerBase;
use crate::audio_data::Vector;

pub const KIND: &str = "track";

const INPUT_INDEX: Index = 0;
const GAIN_INDEX: Index = 1;
const CHANNEL_GAIN_INDEX: Index = 2;
// const LEFT_GAIN_INDEX: Index = 2;
// const RIGHT_GAIN_INDEX: Index = 3;

pub struct Track {
    base: TalkerBase,
}
pub type RTrack = Rc<RefCell<Track>>;

impl Track {
    pub fn kind() -> &'static str {
        "track"
    }

    pub fn new(base: TalkerBase) -> Result<Track, failure::Error> {
        /*
                base.add_ear(ear::audio(
                    None,
                    AudioFormat::MIN_AUDIO,
                    AudioFormat::MAX_AUDIO,
                    AudioFormat::DEF_AUDIO,
                    &Init::Empty,
                )?);
                base.add_ear(ear::audio(Some("gain"), 0., 1., 0.5, &Init::DefValue)?);
                base.add_ear(ear::set(
                    Some("channels gains"),
                    false,
                    &vec![
                        ("left", PortType::Cv, 0., 1., 1., Init::DefValue),
                        ("right", PortType::Cv, 0., 1., 1., Init::DefValue),
                    ],
                )?);
        */
        Ok(Self { base })
    }
    pub fn new_ref(base: TalkerBase) -> Result<RTrack, failure::Error> {
        Ok(Rc::new(RefCell::new(Track::new(base)?)))
    }

    pub fn id() -> &'static str {
        "Track"
    }

    pub fn base(&self) -> &TalkerBase {
        &self.base
    }

    pub fn to_set(&self) -> Result<Set, failure::Error> {
        /*
                let mut hums
                Ok(Set::new(vec![
                    self.ears()[INPUT_INDEX].clone_hum(0, 0)?,
                    self.ears()[GAIN_INDEX].clone_hum(0, 0)?,
                    self.ears()[LEFT_GAIN_INDEX].clone_hum(0, 0)?,
                    self.ears()[RIGHT_GAIN_INDEX].clone_hum(0, 0)?,
                ]))
        */
        Ok(Set::new(
            self.base
                .ears()
                .iter()
                .map(|ear| ear.clone_hum(0, 0).unwrap())
                .collect::<Vec<_>>(),
        ))
    }

    fn compute_input_gain(set: &Set, _tick: i64, buf: &mut Vector, len: usize) -> usize {
        //        let ln = self.listen_ears(tick, len);

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
        let ln = Track::compute_input_gain(set, tick, buf, len);

        // let mut min_val = f32::MAX;
        // let mut max_val = f32::MIN;

        for i in 0..channels.len() {
            //            println!("Track::set channel {}/{}", i, n);
            let ch = &mut channels[i];
            let cg = set.get_hum_cv_buffer(i + CHANNEL_GAIN_INDEX);

            for j in 0..ln {
                let v = cg[j] * buf[j];
                // min_val = f32::min(min_val, v);
                // max_val = f32::max(max_val, v);
                ch[j] = v;
            }
        }

        // println!("{}", "-".repeat(((max_val - min_val) * 50.) as usize));

        ln
    }

    pub fn add(
        set: &Set,
        tick: i64,
        buf: &mut Vector,
        len: usize,
        channels: &mut Vec<Vector>,
    ) -> usize {
        let ln = Track::compute_input_gain(set, tick, buf, len);

        for i in 0..channels.len() {
            let ch = &mut channels[i];
            let cg = set.get_hum_cv_buffer(i + CHANNEL_GAIN_INDEX);

            for j in 0..ln {
                ch[j] = ch[j] + cg[j] * buf[j];
            }
        }

        ln
    }
}
