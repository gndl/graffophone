use std::cell::RefCell;
use std::rc::Rc;

use talker::audio_format::AudioFormat;
use talker::ear;
use talker::ear::Init;
use talker::talker::{Talker, TalkerBase};
use talker::voice::PortType;

use crate::audio_data::Vector;

pub const KIND: &str = "track";

pub struct Track {
    base: TalkerBase,
}
pub type RTrack = Rc<RefCell<Track>>;

impl Track {
    pub fn kind() -> &'static str {
        "track"
    }

    pub fn new() -> Result<Track, failure::Error> {
        let mut base = TalkerBase::new("", KIND);

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

        Ok(Self { base })
    }
    pub fn new_ref() -> Result<RTrack, failure::Error> {
        Ok(Rc::new(RefCell::new(Track::new()?)))
    }

    pub fn id() -> &'static str {
        "Track"
    }

    fn compute_input_gain(&self, tick: i64, buf: &mut Vector, len: usize) -> usize {
        let ln = self.listen_ears(tick, len);

        let in_buf = self.ear_audio_buffer(0).unwrap();
        let gain_buf = self.ear_audio_buffer(1).unwrap();

        for i in 0..ln {
            buf[i] = in_buf[i].get() * gain_buf[i].get();
        }
        ln
    }

    pub fn set(
        &self,
        tick: i64,
        buf: &mut Vector,
        len: usize,
        channels: &mut Vec<Vector>,
    ) -> usize {
        let ln = self.compute_input_gain(tick, buf, len);
        let channels_gains_ear = &self.ears()[2];
        let mut min_val = f32::MAX;
        let mut max_val = f32::MIN;

        let n = std::cmp::min(channels.len(), channels_gains_ear.hums_len());

        for i in 0..n {
            //            println!("Track::set channel {}/{}", i, n);
            let ch = &mut channels[i];
            let cg = channels_gains_ear.get_set_hum_cv_buffer(0, i).unwrap();

            for j in 0..ln {
                let v = cg[j].get() * buf[j];
                min_val = f32::min(min_val, v);
                max_val = f32::max(max_val, v);
                ch[j] = v;
            }
        }

        let amplitude = ((max_val - min_val) * 50.) as usize;
        // println!("{}", "-".repeat(amplitude));

        for i in n..channels.len() {
            let ch = &mut channels[i];
            for j in 0..ln {
                ch[j] = buf[j];
            }
        }
        ln
    }

    pub fn add(
        &self,
        tick: i64,
        buf: &mut Vector,
        len: usize,
        channels: &mut Vec<Vector>,
    ) -> usize {
        let ln = self.compute_input_gain(tick, buf, len);
        let channels_gains_ear = &self.ears()[2];

        let n = std::cmp::min(channels.len(), channels_gains_ear.hums_len());

        for i in 0..n {
            let ch = &mut channels[i];
            let cg = channels_gains_ear.get_set_hum_cv_buffer(0, i).unwrap();

            for j in 0..ln {
                ch[j] = ch[j] + cg[j].get() * buf[j];
            }
        }

        for i in n..channels.len() {
            let ch = &mut channels[i];
            for j in 0..ln {
                ch[j] = ch[j] + buf[j];
            }
        }
        ln
    }
}

impl Talker for Track {
    fn base<'a>(&'a self) -> &'a TalkerBase {
        &self.base
    }

    fn model(&self) -> &str {
        Track::kind()
    }
}
