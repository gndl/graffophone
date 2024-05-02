use std::cell::RefCell;
use std::rc::Rc;

use talker::audio_format::AudioFormat;
use talker::ear;
use talker::ear::Ear;
use talker::ear::Init;
use talker::ear::Set;
use talker::horn::PortType;
use talker::identifier::{Id, Identifiable, Index, RIdentifier};
use talker::talker::{MuteTalker, RTalker, TalkerBase};

use crate::audio_data::Vector;
use crate::feedback::Feedback;
use crate::output::{Output, ROutput};
use tables::fadeout;
use crate::track::Track;

pub const KIND: &str = "Mixer";

const VOLUME_EAR_INDEX: Index = 0;
const TRACKS_EAR_INDEX: Index = 1;
const INPUT_HUM_INDEX: Index = 0;
const GAIN_HUM_INDEX: Index = 1;
const CHANNELS_HUM_INDEX: Index = 2;

pub struct Mixer {
    talker: RTalker,
    outputs: Vec<ROutput>,
    tick: i64,
    productive: bool,
    record: bool,
    feedback: Option<Feedback>,
    buf: Vector,
    channels_buffers: Vec<Vector>,
}

pub type RMixer = Rc<RefCell<Mixer>>;

impl Mixer {
    pub fn new(
        source: Option<&Mixer>,
        outputs: Vec<ROutput>,
    ) -> Result<Mixer, failure::Error> {
        let mut channels = 0;
        let mut output_idx = usize::MAX;

        for (idx, out) in outputs.iter().enumerate() {
            let ocs = out.borrow().channels();

            if ocs > channels {
                channels = ocs;
                output_idx = idx;
            }
        }

        let mut hums_attributs = vec![
            ("", PortType::Audio, AudioFormat::MIN_AUDIO, AudioFormat::MAX_AUDIO, AudioFormat::DEF_AUDIO, Init::Empty),
            ("gain", PortType::Audio, 0., 1., 0.5, Init::DefValue),
        ];

        if output_idx < outputs.len() {
            for chan_name in outputs[output_idx].borrow().channels_names() {
                hums_attributs.push((chan_name, PortType::Cv, 0., 1., 1., Init::DefValue));
            }
        }
        else {
            hums_attributs.push(("left", PortType::Cv, 0., 1., 1., Init::DefValue));
            hums_attributs.push(("right", PortType::Cv, 0., 1., 1., Init::DefValue));
            channels = 2;
        }
        let stem_track = Set::from_attributs(&hums_attributs)?;

        let mut tracks = Vec::new();

        let mut base = TalkerBase::new("", KIND);

        if let Some(src) = source {
            base.add_ear(src.talker.ear(VOLUME_EAR_INDEX).clone());

            let channels_hums_end = usize::min(channels, src.channels()) + CHANNELS_HUM_INDEX;

            for src_track in src.talker.ear(TRACKS_EAR_INDEX).sets() {
                let track = stem_track.clone();
                let track = track.with_hum(INPUT_HUM_INDEX, |_| Ok(src_track.hums()[INPUT_HUM_INDEX].clone()))?;
                let mut track = track.with_hum(GAIN_HUM_INDEX, |_| Ok(src_track.hums()[GAIN_HUM_INDEX].clone()))?;

                for hum_idx in CHANNELS_HUM_INDEX..channels_hums_end {
                    track = track.with_hum(hum_idx, |h| Ok(src_track.hums()[hum_idx].with_tag(h.tag())))?;
                }
                tracks.push(track);
            }
        }
        else {
            base.add_ear(ear::cv(Some("volume"), 0., 1., 0.1, &Init::DefValue)?);
            tracks.push(stem_track.clone());
        }
        
        base.add_ear(Ear::new(Some("Tracks"), true, Some(stem_track), Some(tracks)));

        let chunk_size = AudioFormat::chunk_size();

        let mut channels_buffers = Vec::new();

        for _ in 0..channels {
            channels_buffers.push(vec![0.; chunk_size]);
        }

        Ok(Self {
            talker: MuteTalker::new(base),
            outputs,
            tick: 0,
            productive: false,
            record: false,
            feedback: None,
            buf: vec![0.; AudioFormat::chunk_size()],
            channels_buffers,
        })
    }

    pub fn new_ref(
        source: Option<&Mixer>,
        outputs: Vec<ROutput>,
    ) -> Result<RMixer, failure::Error> {
        Ok(Rc::new(RefCell::new(Mixer::new(source, outputs)?)))
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

    pub fn set_record(&mut self, active:bool) -> Result<(), failure::Error> {
        self.record = active;
        Ok(())
    }

    pub fn set_feedback(&mut self, active:bool) -> Result<(), failure::Error> {
        if active && self.feedback.is_none() {
            let mut feedback = Feedback::new(AudioFormat::chunk_size())?;

            if self.productive {
                feedback.open()?;
            }

            self.feedback = Some(feedback);
        }
        else if !active {
            if let Some(feedback) = &mut self.feedback {
                feedback.close()?;
                self.feedback = None;
            }
        }
        Ok(())
    }

    pub fn channels(&self) -> usize {
        self.channels_buffers.len()
    }

    pub fn open(&mut self) -> Result<(), failure::Error> {

        if self.record {
            for o in &self.outputs {
                o.borrow_mut().open()?;
            }
        }

        if let Some(feedback) = &mut self.feedback {
            feedback.open()?;
        }

        self.tick = 0;
        self.productive = true;
        Ok(())
    }

    pub fn pause(&mut self) -> Result<(), failure::Error> {

        if self.record {
            for o in &self.outputs {
                o.borrow_mut().pause()?;
            }
        }

        if let Some(feedback) = &mut self.feedback {
            feedback.pause()?;
        }
        Ok(())
    }

    pub fn run(&mut self) -> Result<(), failure::Error> {

        if self.record {
            for o in &self.outputs {
                o.borrow_mut().run()?;
            }
        }

        if let Some(feedback) = &mut self.feedback {
            feedback.run()?;
        }
        Ok(())
    }

    pub fn close(&mut self) -> Result<(), failure::Error> {
        
        let res = self.fadeout();

        if self.record {
            for o in &self.outputs {
                o.borrow_mut().close()?;
            }
        }

        if let Some(feedback) = &mut self.feedback {
            feedback.close()?;
        }
        self.productive = false;
        res
    }

    pub fn come_out(
        &mut self,
        tick: i64,
        len: usize,
        outputs_gain_buf: Option<&[f32]>,
    ) -> Result<usize, failure::Error> {
        let mut ln = self.talker.listen(tick, len);

        let tracks_ear = &self.talker.ear(TRACKS_EAR_INDEX);

        let buf = &mut self.buf;
        let channels = &mut self.channels_buffers;

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

        if let Some(gain_buf) = outputs_gain_buf {
            for cn in 0..channels.len() {
                let ch = &mut channels[cn];

                for i in 0..ln {
                    ch[i] = ch[i] * gain_buf[i];
                }
            }
        }

        if self.record {
            for o in &self.outputs {
                o.borrow_mut().write(channels, ln)?;
            }
        }

        if let Some(feedback) = &mut self.feedback {
            feedback.write(channels, ln)?;
        }

        self.tick = tick + ln as i64;

        Ok(ln)
    }

    fn fadeout(&mut self,) -> Result<(), failure::Error> {
        let len = fadeout::LEN;
        let fadeout_buf : &[f32] = &fadeout::TAB;
        
        let _ = self.come_out(self.tick, len, Some(fadeout_buf))?;
        Ok(())
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
