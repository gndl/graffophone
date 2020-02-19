use crate::audio_data::Vector;
use crate::output::ROutput;
use crate::playback_output::Playback;
use crate::track::Track;
use gpplugin::audio_format::AudioFormat;
use gpplugin::ear;
use gpplugin::talker::{Talker, TalkerBase};
use std::boxed::Box;
use std::cell::RefCell;
use std::rc::Rc;

pub const KIND: &str = "mixer";

pub struct Mixer {
    base: TalkerBase,
    tracks: Vec<Track>,
    outputs: Vec<ROutput>,
    channels: Vec<Vector>,
    tick: i64,
    productive: bool,
}

impl Mixer {
    pub fn new(tracks: Vec<Track>, nb_channels: usize) -> Mixer {
        let mut base = TalkerBase::new();

        base.add_ear(ear::cv(Some("volume".to_string()), Some(1.), None));
        let mut channels = Vec::with_capacity(nb_channels);
        let chunk_size = AudioFormat::chunk_size();

        for _ in 0..nb_channels {
            channels.push(vec![0.; chunk_size]);
        }

        Self {
            base,
            tracks,
            outputs: vec![Box::new(Playback::new(nb_channels, chunk_size).unwrap())],
            channels,
            tick: 0,
            productive: false,
        }
    }
    /*
        pub fn id() -> &'static str {
            "Mixer"
        }
    */
    pub fn open_output(&mut self) -> Result<(), failure::Error> {
        for o in &self.outputs {
            o.open()?;
        }
        self.tick = 0;
        self.productive = true;
        Ok(())
    }

    pub fn close_output(&mut self) -> Result<(), failure::Error> {
        for o in &self.outputs {
            o.close()?;
        }
        self.productive = false;
        Ok(())
    }

    fn come_out(
        &mut self,
        tick: i64,
        buf: &mut Vector,
        channels: &mut Vec<Vector>,
        len: usize,
    ) -> Result<usize, failure::Error> {
        let mut ln = self.listen_ears(tick, len);

        match self.tracks.get(0) {
            Some(track) => {
                ln = track.set(tick, buf, ln, channels);
            }
            _ => (),
        };

        for i in 1..self.tracks.len() {
            match self.tracks.get(i) {
                Some(track) => {
                    ln = track.add(tick, buf, ln, channels);
                }
                _ => (),
            };
        }

        let master_volume_buf = self.ear_cv_buffer(0).unwrap();

        for cn in 0..channels.len() {
            let mut ch = &mut channels[cn];

            for i in 0..ln {
                let mut sample = ch[i] * master_volume_buf[i].get();

                if sample < AudioFormat::MIN_AUDIO {
                    sample = AudioFormat::MIN_AUDIO;
                } else if sample > AudioFormat::MAX_AUDIO {
                    sample = AudioFormat::MAX_AUDIO;
                }
                ch[i] = sample;
            }
        }

        for o in &self.outputs {
            o.write(channels, ln)?;
        }
        Ok(ln)
    }
}

impl Talker for Mixer {
    fn base<'a>(&'a self) -> &'a TalkerBase {
        &self.base
    }

    fn model(&self) -> &str {
        KIND //        "mixer"
    }
}

pub type RMixer = Rc<RefCell<Mixer>>;
