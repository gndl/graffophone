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
use crate::output::ROutput;
use tables;
use crate::track;

pub const KIND: &str = "Mixer";

const VOLUME_EAR_INDEX: Index = 0;
const TRACKS_EAR_INDEX: Index = 1;
const INPUT_HUM_INDEX: Index = 0;
const GAIN_HUM_INDEX: Index = 1;
const CHANNELS_HUM_INDEX: Index = 2;

pub struct Mixer {
    talker: RTalker,
    outputs: Vec<ROutput>,
    is_open: bool,
    record: bool,
    buf: Vector,
    channels_buffers: Vec<Vector>,
}

pub type RMixer = Rc<RefCell<Mixer>>;

impl Mixer {
    pub fn new_ref(
        oparent: Option<&RMixer>,
        outputs: Vec<ROutput>,
    ) -> Result<RMixer, failure::Error> {
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
            ("gain", PortType::Audio, 0., 1., 1., Init::DefValue),
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

        let mut base = TalkerBase::new("", KIND, true);

        if let Some(rparent) = oparent {
            let parent = rparent.borrow();

            base.add_ear(parent.talker.ear(VOLUME_EAR_INDEX).clone());

            let channels_hums_end = usize::min(channels, parent.channels()) + CHANNELS_HUM_INDEX;

            for src_track in parent.talker.ear(TRACKS_EAR_INDEX).sets() {
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

        Ok(Rc::new(RefCell::new(Self {
            talker: MuteTalker::new(base),
            outputs,
            is_open: false,
            record: false,
            buf: vec![0.; AudioFormat::chunk_size()],
            channels_buffers,
        })))
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

    pub fn record(&self) -> bool {
        self.record
    }
    pub fn set_record(&mut self, active:bool) -> Result<(), failure::Error> {
        self.record = active;
        Ok(())
    }

    pub fn channels(&self) -> usize {
        self.channels_buffers.len()
    }

    pub fn channels_buffers<'a>(&'a self) -> &'a Vec<Vector> {
        &self.channels_buffers
    }

    pub fn is_open(&self) -> bool {
        self.is_open
    }
    pub fn open(&mut self) -> Result<(), failure::Error> {

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
