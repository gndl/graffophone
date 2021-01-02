use std::cell::RefCell;
use std::rc::Rc;

use talker::ear;
use talker::ear::Ear;
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

        base.add_ear(ear::audio(None, None, None));
        base.add_ear(ear::audio(Some("gain"), Some(1.), None));
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

        let in_horn = self.ear_audio_buffer(0).unwrap();
        let in_buf = in_horn.borrow();
        let gain_horn = self.ear_audio_buffer(1).unwrap();
        let gain_buf = gain_horn.borrow();

        for i in 0..ln {
            buf[i] = in_buf[i] * gain_buf[i];
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

        match self.ears().get(2).unwrap() {
            Ear::Talks(talks) => {
                let cgs = talks.borrow();
                let n = std::cmp::min(channels.len(), cgs.talks().len());

                for i in 0..n {
                    let ch = &mut channels[i];
                    let cg_horn = cgs.talks()[i].borrow().cv_buffer().unwrap();
                    let cg = cg_horn.borrow();

                    for j in 0..ln {
                        ch[j] = cg[j] * buf[j];
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
            _ => 0,
        }
    }

    pub fn add(
        &self,
        tick: i64,
        buf: &mut Vector,
        len: usize,
        channels: &mut Vec<Vector>,
    ) -> usize {
        let ln = self.compute_input_gain(tick, buf, len);

        match self.ears().get(2).unwrap() {
            Ear::Talks(talks) => {
                let cgs = talks.borrow();
                let n = std::cmp::min(channels.len(), cgs.talks().len());

                for i in 0..n {
                    let ch = &mut channels[i];
                    let cg_horn = cgs.talks()[i].borrow().cv_buffer().unwrap();
                    let cg = cg_horn.borrow();

                    for j in 0..ln {
                        ch[j] = ch[j] + cg[j] * buf[j];
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
            _ => 0,
        }
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
