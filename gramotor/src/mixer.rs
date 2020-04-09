use std::cell::RefCell;

use granode::audio_format::AudioFormat;
use granode::ear;
use granode::talker::{Talker, TalkerBase};

use crate::audio_data::Vector;
use crate::output::ROutput;
use crate::track::Track;

pub const KIND: &str = "mixer";

pub struct Mixer {
    base: TalkerBase,
    tracks: Vec<Track>,
    outputs: Vec<ROutput>,
    tick: i64,
    productive: bool,
}

//pub type RMixer = Rc<RefCell<Mixer>>;
pub type RMixer = RefCell<Mixer>;

impl Mixer {
    pub fn new(tracks: Option<Vec<Track>>, outputs: Option<Vec<ROutput>>) -> Mixer {
        let mut base = TalkerBase::new("", KIND);

        base.add_ear(ear::cv(Some("volume"), Some(1.), None));

        Self {
            base,
            tracks: tracks.unwrap_or(Vec::new()),
            outputs: outputs.unwrap_or(Vec::new()),
            tick: 0,
            productive: false,
        }
    }
    pub fn new_ref(tracks: Option<Vec<Track>>, outputs: Option<Vec<ROutput>>) -> RMixer {
        RefCell::new(Mixer::new(tracks, outputs))
    }

    pub fn kind() -> &'static str {
        KIND
    }

    pub fn tracks<'a>(&'a self) -> &'a Vec<Track> {
        &self.tracks
    }

    pub fn add_track(&mut self, track: Track) {
        self.tracks.push(track);
    }

    pub fn outputs<'a>(&'a self) -> &'a Vec<ROutput> {
        &self.outputs
    }

    pub fn add_output(&mut self, output: ROutput) {
        self.outputs.push(output);
    }

    pub fn open(&mut self) -> Result<(), failure::Error> {
        for o in &self.outputs {
            o.borrow_mut().open()?;
        }
        self.tick = 0;
        self.productive = true;
        Ok(())
    }

    pub fn pause(&mut self) -> Result<(), failure::Error> {
        for o in &self.outputs {
            o.borrow_mut().pause()?;
        }
        Ok(())
    }

    pub fn run(&mut self) -> Result<(), failure::Error> {
        for o in &self.outputs {
            o.borrow_mut().run()?;
        }
        Ok(())
    }

    pub fn close(&mut self) -> Result<(), failure::Error> {
        for o in &self.outputs {
            o.borrow_mut().close()?;
        }
        self.productive = false;
        Ok(())
    }

    pub fn come_out(
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
            let ch = &mut channels[cn];

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
            o.borrow_mut().write(channels, ln)?;
        }
        Ok(ln)
    }
}

impl Talker for Mixer {
    fn base<'a>(&'a self) -> &'a TalkerBase {
        &self.base
    }

    fn model(&self) -> &str {
        Mixer::kind()
    }
}