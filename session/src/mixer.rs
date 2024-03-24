use std::cell::RefCell;
use std::rc::Rc;

use talker::audio_format::AudioFormat;
use talker::ear;
use talker::ear::Ear;
use talker::ear::Init;
use talker::ear::Set;
use talker::horn::PortType;
// use talker::talker::RTalker;
use talker::identifier::{Id, Identifiable, Index, RIdentifier};
use talker::talker::{MuteTalker, RTalker, TalkerBase};

use crate::audio_data::Vector;
use crate::output::ROutput;
use crate::track::Track;

pub const KIND: &str = "Mixer";

const VOLUME_EAR_INDEX: Index = 0;
const TRACKS_EAR_INDEX: Index = 1;

pub struct Mixer {
    talker: RTalker,
    outputs: Vec<ROutput>,
    tick: i64,
    productive: bool,
}

pub type RMixer = Rc<RefCell<Mixer>>;

impl Mixer {
    pub fn new(
        ooutputs: Option<Vec<ROutput>>,
    ) -> Result<Mixer, failure::Error> {
        let mut base = TalkerBase::new("", KIND);

        base.add_ear(ear::cv(Some("volume"), 0., 1., 0.1, &Init::DefValue)?);

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
        sets.push(stem_set.clone());

        base.add_ear(Ear::new(Some("Tracks"), true, Some(stem_set), Some(sets)));

        Ok(Self {
            talker: MuteTalker::new(base),
            outputs: ooutputs.unwrap_or(Vec::new()),
            tick: 0,
            productive: false,
        })
    }
    pub fn new_ref(
        outputs: Option<Vec<ROutput>>,
    ) -> Result<RMixer, failure::Error> {
        Ok(Rc::new(RefCell::new(Mixer::new(outputs)?)))
    }

    pub fn kind() -> &'static str {
        KIND
    }
    pub fn identifier(&self) -> &RIdentifier {
        self.talker.identifier()
    }
    pub fn talker(&self) -> &RTalker {
        &self.talker
    }
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
        extra_outputs_gain_buf:Option<&[f32]>,
    ) -> Result<usize, failure::Error> {
        let mut ln = self.talker.listen(tick, len);

        let tracks_ear = &self.talker.ear(TRACKS_EAR_INDEX);

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
        let master_volume_buf = self.talker.ear_cv_buffer(VOLUME_EAR_INDEX);

        for cn in 0..channels.len() {
            let ch = &mut channels[cn];

            for i in 0..ln {
                let mut sample = ch[i] * master_volume_buf[i];

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

        if let Some(gain_buf) = extra_outputs_gain_buf {
            for cn in 0..channels.len() {
                let ch = &mut channels[cn];

                for i in 0..ln {
                    ch[i] = ch[i] * gain_buf[i];
                }
            }
        }
        for o in extra_outputs {
            o.borrow_mut().write(channels, ln)?;
        }
        Ok(ln)
    }
}
impl Identifiable for Mixer {
    fn id(&self) -> Id {
        self.talker.id()
    }
    fn set_id(&self, id: Id) {
        self.talker.set_id(id);
    }
    fn name(&self) -> String {
        self.talker.name()
    }
    fn set_name(&self, name: &str) {
        self.talker.set_name(name);
    }
}
