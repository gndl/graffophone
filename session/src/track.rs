use std::cell::RefCell;
use std::rc::Rc;

use talker::audio_format::AudioFormat;
use talker::ear;
use talker::talker::{Talker, TalkerBase};

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

    pub fn new() -> Track {
        let mut base = TalkerBase::new("", KIND);

        base.add_ear(ear::audio(
            None,
            AudioFormat::MIN_AUDIO,
            AudioFormat::MAX_AUDIO,
            AudioFormat::DEF_AUDIO,
            None,
        ));
        base.add_ear(ear::audio(Some("gain"), 0., 1., 1., None));
        base.add_ear(ear::cvs(Some("channelGain")));

        Self { base }
    }
    pub fn new_ref() -> RTrack {
        Rc::new(RefCell::new(Track::new()))
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

        let n = std::cmp::min(channels.len(), channels_gains_ear.sets_len());

        for i in 0..n {
            let ch = &mut channels[i];
            let cg = channels_gains_ear.get_set_cv_buffer(i).unwrap();

            for j in 0..ln {
                ch[j] = cg[j].get() * buf[j];
            }
        }

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

        let n = std::cmp::min(channels.len(), channels_gains_ear.sets_len());

        for i in 0..n {
            let ch = &mut channels[i];
            let cg = channels_gains_ear.get_set_cv_buffer(i).unwrap();

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
