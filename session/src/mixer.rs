use std::cell::RefCell;
use std::rc::Rc;
use std::collections::HashSet;

use talker::audio_format::AudioFormat;
use talker::ear;
use talker::ear::Ear;
use talker::ear::Init;
use talker::ear::Set;
use talker::horn::PortType;
use talker::identifier::{Id, Identifiable, Index, RIdentifier};
use talker::talker::{MuteTalker, RTalker, TalkerBase};

use crate::audio_data::Vector;
use crate::output::ROutput;
use tables;
use crate::track;

pub const KIND: &str = "Mixer";

pub const VOLUME_EAR_INDEX: Index = 0;
pub const TRACKS_EAR_INDEX: Index = 1;
const INPUT_HUM_INDEX: Index = 0;
const GAIN_HUM_INDEX: Index = 1;
const CHANNELS_HUM_INDEX: Index = 2;

pub struct Mixer {
    talker: RTalker,
    outputs: Vec<ROutput>,
    is_open: bool,
    record: bool,
    buf: Vector,
    tracks_count: usize,
    channels_count: usize,
    channels_buffers: Vec<Vector>,
    feedback_buffers: Vec<Vector>,
    audible_tracks: Vec<Index>,
}

pub type RMixer = Rc<RefCell<Mixer>>;

impl Mixer {
    pub fn new_ref(
        id: Id,
        name: &str,
        oparent: Option<&RMixer>,
        outputs: Vec<ROutput>,
        effective: bool,
    ) -> Result<RMixer, failure::Error> {
        let mut channels_count = 0;
        let mut output_idx = usize::MAX;

        for (idx, out) in outputs.iter().enumerate() {
            let ocs = out.borrow().channels_count();

            if ocs > channels_count {
                channels_count = ocs;
                output_idx = idx;
            }
        }

        let mut hums_attributs = vec![
            ("in", PortType::Audio, AudioFormat::MIN_AUDIO, AudioFormat::MAX_AUDIO, AudioFormat::DEF_AUDIO, Init::Empty),
            ("gain", PortType::Cv, 0., 4., 1., Init::DefValue),
        ];

        if output_idx < outputs.len() {
            for chan_name in outputs[output_idx].borrow().channels_names() {
                hums_attributs.push((chan_name, PortType::Cv, 0., 1., 1., Init::DefValue));
            }
        }
        else {
            hums_attributs.push(("left", PortType::Cv, 0., 1., 1., Init::DefValue));
            hums_attributs.push(("right", PortType::Cv, 0., 1., 1., Init::DefValue));
            channels_count = 2;
        }
        let stem_track = Set::from_attributs(&hums_attributs)?;

        let mut tracks = Vec::new();
        let mut audible_tracks = Vec::new();

        let mut base = TalkerBase::new(id, name, KIND, true);

        if let Some(rparent) = oparent {
            let parent = rparent.borrow();

            base.add_ear(parent.talker.ear(VOLUME_EAR_INDEX).clone());

            let channels_hums_end = channels_count.min(parent.channels_count()) + CHANNELS_HUM_INDEX;

            for src_track in parent.talker.ear(TRACKS_EAR_INDEX).sets() {
                let track = stem_track.clone();
                let input_hum = &src_track.hums()[INPUT_HUM_INDEX];

                let track = if effective {
                    track.with_hum(INPUT_HUM_INDEX, |_| Ok(input_hum.clone()))?
                }
                else {
                    let mut input_tag = "in".to_string();
                    
                    for talk in input_hum.talks() {
                        if let Some(tag) = talk.talker().fetch_tag(talk.port()) {
                            input_tag = tag;
                            break;
                        }
                    }
                    track.with_hum(INPUT_HUM_INDEX, |_| Ok(input_hum.with_tag(&input_tag)))?
                };
                let mut track = track.with_hum(GAIN_HUM_INDEX, |_| Ok(src_track.hums()[GAIN_HUM_INDEX].clone()))?;

                for hum_idx in CHANNELS_HUM_INDEX..channels_hums_end {
                    track = track.with_hum(hum_idx, |h| Ok(src_track.hums()[hum_idx].with_tag(h.tag())))?;
                }
                tracks.push(track);
            }
            audible_tracks.extend_from_slice(&parent.audible_tracks);
        }
        else {
            base.add_ear(ear::cv(Some("volume"), 0., 4., 0.1, &Init::DefValue)?);
            tracks.push(stem_track.clone());
            audible_tracks.push(0);
        }
        let tracks_count = tracks.len();

        base.add_ear(Ear::new(Some("Tracks"), true, Some(stem_track), Some(tracks)));

        let chunk_size = AudioFormat::chunk_size();

        let (buf, channels_buffers, feedback_buffers) = if effective {
            let mut channels_buffers = Vec::with_capacity(channels_count);
            let mut feedback_buffers = Vec::with_capacity(channels_count);
    
            for _ in 0..channels_count {
                channels_buffers.push(vec![0.; chunk_size]);
                feedback_buffers.push(vec![0.; chunk_size]);
            }
            (vec![0.; AudioFormat::chunk_size()], channels_buffers, feedback_buffers)
        }
        else {
            (Vec::new(), Vec::new(), Vec::new())
        };

        Ok(Rc::new(RefCell::new(Self {
            talker: MuteTalker::new(base),
            outputs,
            is_open: false,
            record: false,
            buf,
            tracks_count,
            channels_count,
            channels_buffers,
            feedback_buffers,
            audible_tracks,
        })))
    }

    pub fn kind() -> &'static str {
        KIND
    }

    pub fn initialize_talkers(&self) -> Result<(), failure::Error> {
        let mut initialized_talkers = HashSet::new();

        self.talker.initialize(&mut initialized_talkers)
    }

    pub fn initialize(rmixer: RMixer, effective: bool) -> Result<RMixer, failure::Error> {
        {
            let mut mixer = rmixer.borrow_mut();
            let tracks_ear = mixer.talker.ear(TRACKS_EAR_INDEX);
            
            mixer.tracks_count = tracks_ear.sets_len();
            
            for trk_idx in mixer.audible_tracks.len()..mixer.tracks_count {
                mixer.audible_tracks.push(trk_idx);
            }
        }

        if effective {
            rmixer.borrow().initialize_talkers()?;
            Ok(rmixer)
        }
        else {
            let id = rmixer.borrow().id();
            let name = rmixer.borrow().name();
            let outputs = std::mem::take(&mut rmixer.borrow_mut().outputs);

            // Create a new mixer with fetched tracks tags
            Mixer::new_ref(id, &name, Some(&rmixer), outputs, effective)
        }
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

    pub fn record(&self) -> bool {
        self.record
    }
    pub fn set_record(&mut self, active:bool) -> Result<(), failure::Error> {
        self.record = active;
        Ok(())
    }

    pub fn channels_count(&self) -> usize {
        self.channels_count
    }

    pub fn channels_buffers<'a>(&'a self) -> &'a Vec<Vector> {
        &self.channels_buffers
    }
    
    pub fn feedback<'a>(&'a self) -> &'a Vec<Vector> {
        match self.audible_tracks.len() < self.tracks_count {
            true => &self.feedback_buffers,
            false => &self.channels_buffers
       }
    }
    pub fn set_audible_tracks(&mut self, audible_tracks: Vec<Index>) -> Result<(), failure::Error> {

        if audible_tracks.is_empty() {
            for buf in &mut self.feedback_buffers {
                buf.fill(0.);
            }
        }
        self.audible_tracks = audible_tracks;
        Ok(())
    }
    pub fn is_open(&self) -> bool {
        self.is_open
    }
    pub fn open(&mut self) -> Result<(), failure::Error> {
        self.initialize_talkers()?;

        if self.record {
            for o in &self.outputs {
                o.borrow_mut().open()?;
            }
        }

        self.is_open = true;
        Ok(())
    }

    pub fn pause(&mut self) -> Result<(), failure::Error> {

        if self.record {
            for o in &self.outputs {
                o.borrow_mut().pause()?;
            }
        }
        Ok(())
    }

    pub fn run(&mut self) -> Result<(), failure::Error> {

        if self.record {
            for o in &self.outputs {
                o.borrow_mut().run()?;
            }
        }
        Ok(())
    }

    pub fn close(&mut self) -> Result<(), failure::Error> {
        
        if self.record {
            for o in &self.outputs {
                o.borrow_mut().close()?;
            }
        }

        self.is_open = false;
        Ok(())
    }

    pub fn come_out(
        &mut self,
        tick: i64,
        len: usize,
    ) -> Result<usize, failure::Error> {
        let mut ln = self.talker.listen(tick, len);

        let tracks_ear = &self.talker.ear(TRACKS_EAR_INDEX);

        let tracks_count = tracks_ear.sets_len();
        self.tracks_count = tracks_count;

        if tracks_count == 0 {
            return Ok(0);
        }

        let buf = &mut self.buf;
        let channels = &mut self.channels_buffers;

        ln = tracks_ear.visit_set(
            0,
            |set, ln| Ok(track::set(set, tick, buf, ln, channels)),
            ln,
        )?;

        for i in 1..tracks_count {
            ln = tracks_ear.visit_set(
                i,
                |set, ln| Ok(track::add(set, tick, buf, ln, channels)),
                ln,
            )?;
        }

        let master_volume_buf = self.talker.ear_cv_buffer(VOLUME_EAR_INDEX);
        let average_coef = 1. / tracks_count as f32;

        for ch in &mut *channels {
            for i in 0..ln {
                ch[i] = ch[i] * master_volume_buf[i] * average_coef;
            }
        }

        if self.record {
            for o in &self.outputs {
                o.borrow_mut().write(channels, ln)?;
            }
        }

        // Compute feedback
        if self.audible_tracks.len() < tracks_count && !self.audible_tracks.is_empty() {
            let channels = &mut self.feedback_buffers;
            let trk_idx = self.audible_tracks[0];

            if trk_idx < tracks_count {
                let _ = tracks_ear.visit_set(
                    trk_idx,
                    |set, ln| Ok(track::set(set, tick, buf, ln, channels)),
                    ln,
                )?;
            }

            for i in 1..self.audible_tracks.len() {
                let trk_idx = self.audible_tracks[i];

                if trk_idx < tracks_count {
                    let _ = tracks_ear.visit_set(
                        trk_idx,
                        |set, ln| Ok(track::add(set, tick, buf, ln, channels)),
                        ln,
                    )?;
                }
            }

            for ch in &mut *channels {
                for i in 0..ln {
                    ch[i] = ch[i] * master_volume_buf[i] * average_coef;
                }
            }
        }

        Ok(ln)
    }

    pub fn fadeout(&mut self, tick: i64) -> Result<usize, failure::Error> {
        let fadeout_tab = tables::create_fadeout(AudioFormat::sample_rate());
        let record = self.record;
        self.record = false;
        
        let ln = self.come_out(tick, fadeout_tab.len())?;

        for ch in &mut self.channels_buffers {
            for i in 0..ln {
                ch[i] = ch[i] * fadeout_tab[i];
            }
        }

        if record {
            for o in &self.outputs {
                o.borrow_mut().write(&self.channels_buffers, ln)?;
            }
        }

        self.record = record;

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
