use std::cell::RefCell;
use std::rc::Rc;

use talker::audio_format::AudioFormat;
use talker::ear;
use talker::ear::Ear;
use talker::ear::Init;
use talker::ear::Set;
use talker::voice::PortType;
// use talker::talker::RTalker;
use talker::identifier::Index;
use talker::talker::{Talker, TalkerBase};

use crate::audio_data::Vector;
use crate::output::ROutput;
use crate::track::RTrack;
use crate::track::Track;

pub const KIND: &str = "mixer";

const VOLUME_EAR_INDEX: Index = 0;
const TRACKS_EAR_INDEX: Index = 1;

pub struct Mixer {
    base: TalkerBase,
    //    tracks: Vec<RTrack>,
    outputs: Vec<ROutput>,
    tick: i64,
    productive: bool,
}

pub type RMixer = Rc<RefCell<Mixer>>;

impl Mixer {
    pub fn new(
        otracks: Option<Vec<RTrack>>,
        ooutputs: Option<Vec<ROutput>>,
    ) -> Result<Mixer, failure::Error> {
        let mut base = TalkerBase::new("", KIND);

        base.add_ear(ear::audio(Some("volume"), 0., 1., 1., &Init::DefValue)?);

        let stem_set = Set::from_attributs(&vec![
            (
                "",
                PortType::Audio,
                AudioFormat::MIN_AUDIO,
                AudioFormat::MAX_AUDIO,
                AudioFormat::DEF_AUDIO,
                Init::Empty,
            ),
            ("gain", PortType::Audio, 0., 1., 0.5, Init::DefValue),
            ("left", PortType::Cv, 0., 1., 1., Init::DefValue),
            ("right", PortType::Cv, 0., 1., 1., Init::DefValue),
        ])?;

        let mut sets = Vec::new();

        if let Some(tracks) = otracks {
            for track in tracks {
                sets.push(track.borrow().to_set()?);
            }
        } else {
            sets.push(stem_set.clone());
        }

        base.add_ear(Ear::new(Some("Tracks"), true, Some(stem_set), Some(sets)));

        Ok(Self {
            base,
            //            tracks: tracks.unwrap_or(Vec::new()),
            outputs: ooutputs.unwrap_or(Vec::new()),
            tick: 0,
            productive: false,
        })
    }
    pub fn new_ref(
        tracks: Option<Vec<RTrack>>,
        outputs: Option<Vec<ROutput>>,
    ) -> Result<RMixer, failure::Error> {
        Ok(Rc::new(RefCell::new(Mixer::new(tracks, outputs)?)))
    }

    pub fn kind() -> &'static str {
        KIND
    }
    /*
        pub fn tracks<'a>(&'a self) -> &'a Vec<RTrack> {
            &self.tracks
        }

        pub fn add_track(&mut self, track: RTrack) {
            let tracks_ear = &self.ears()[TRACKS_EAR_INDEX];
            let set_idx = tracks_ear.add_set();
    Set::new(vec![Hum::copy()]
            if let Some(hum) = track.borrow().ears()[TRACK_INPUT_HUM_INDEX].copy_hum(0, TRACK_INPUT_HUM_INDEX){
                tracks_ear.add_set(set);
            }

            //        self.tracks.push(track);
        }
    */
    pub fn outputs<'a>(&'a self) -> &'a Vec<ROutput> {
        &self.outputs
    }

    pub fn add_output(&mut self, output: ROutput) {
        self.outputs.push(output);
    }

    pub fn remove_output(&mut self, model: &str) {
        let mut outputs = Vec::new();

        for output in &self.outputs {
            if output.borrow().model() != model {
                outputs.push(output.clone());
            }
        }
        self.outputs = outputs;
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
        extra_outputs: &Vec<ROutput>,
    ) -> Result<usize, failure::Error> {
        let mut ln = self.listen_ears(tick, len);

        let tracks_ear = &self.ears()[TRACKS_EAR_INDEX];

        ln = tracks_ear.visit_set(
            0,
            |set, ln| Ok(Track::set(set, tick, buf, ln, channels)),
            ln,
        )?;

        for i in 1..tracks_ear.sets_len() {
            ln = tracks_ear.visit_set(
                i,
                |set, ln| Ok(Track::add(set, tick, buf, ln, channels)),
                ln,
            )?;
        }
        /*
                if let Some(first_set) = tracks_ear.get_set(0) {
                    ln = Track::set(first_set, tick, buf, ln, channels);

                    for i in 1..tracks_ear.sets_len() {
                        ln = Track::set(tracks_ear.sets()[i], tick, buf, ln, channels);
                    }
                }
        */
        let master_volume_buf = self.ear_audio_buffer(VOLUME_EAR_INDEX).unwrap();

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

        for o in extra_outputs {
            o.borrow_mut().write(channels, ln)?;
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
