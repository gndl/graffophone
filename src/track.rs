use crate::audio_data::Vector;
use gpplugin::ear;
use gpplugin::ear::Ear;
use gpplugin::talker::{Talker, TalkerBase};

pub struct Track {
    base: TalkerBase,
}

impl Track {
    pub fn new() -> Track {
        let mut base = TalkerBase::new();

        base.add_ear(ear::audio(None, None, None));
        base.add_ear(ear::audio(Some("gain".to_string()), Some(1.), None));
        base.add_ear(ear::cvs(Some("channelGain".to_string())));

        Self { base }
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

        match self.ears().get(2).unwrap() {
            Ear::Talks(talks) => {
                let cgs = talks.borrow();
                let n = std::cmp::min(channels.len(), cgs.talks().len());

                for i in 0..n {
                    let mut ch = &mut channels[i];
                    let cg = cgs.talks()[i].borrow().cv_buffer().unwrap();

                    for j in 0..ln {
                        ch[j] = cg[j].get() * buf[j];
                    }
                }

                for i in n..channels.len() {
                    let mut ch = &mut channels[i];
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
                    let mut ch = &mut channels[i];
                    let cg = cgs.talks()[i].borrow().cv_buffer().unwrap();

                    for j in 0..ln {
                        ch[j] = ch[j] + cg[j].get() * buf[j];
                    }
                }

                for i in n..channels.len() {
                    let mut ch = &mut channels[i];
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
        "track"
    }

    fn talk(&mut self, _port: usize, _tick: i64, _len: usize) -> usize {
        0
    }
}
