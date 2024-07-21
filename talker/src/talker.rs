use std::cell::RefCell;
use std::rc::Rc;

extern crate failure;

use crate::audio_format::AudioFormat;
use crate::data::{Data, RData};
use crate::ear;
use crate::ear::{Ear, Entree};
use crate::horn::{AudioBuf, CvBuf, PortType};
use crate::identifier::{Id, Identifiable, Identifier, Index, RIdentifier};
use crate::lv2_handler::Lv2Handler;
use crate::voice::{self, Voice};

pub struct TalkerBase {
    identifier: RIdentifier,
    data: RData,
    ears: Vec<Ear>,
    voices: Vec<Voice>,
    hidden: bool,
    effective: bool,
}

impl TalkerBase {
    pub fn new_data(name: &str, model: &str, data: Data, effective: bool) -> Self {
        Self {
            identifier: RefCell::new(Identifier::new(name, model)),
            data: RefCell::new(data),
            ears: Vec::new(),
            voices: Vec::new(),
            hidden: false,
            effective,
        }
    }
    pub fn new(name: &str, model: &str, effective: bool) -> Self {
        TalkerBase::new_data(name, model, Data::Nil, effective)
    }
    pub fn clone(&self) -> Self {
        Self {
            identifier: self.identifier.clone(),
            data: self.data.clone(),
            ears: self.ears.iter().map(|elt| elt.clone()).collect(),
            voices: self.voices.iter().map(|elt| elt.clone()).collect(),
            hidden: self.hidden,
            effective: self.effective,
        }
    }

    pub fn with(
        &self,
        odata: Option<Data>,
        oears: Option<Vec<Ear>>,
        ovoices: Option<Vec<Voice>>,
    ) -> Self {
        Self {
            identifier: RefCell::new(self.identifier.borrow().clone()),
            data: RefCell::new(odata.unwrap_or(Data::Nil)),
            ears: oears.unwrap_or(Vec::new()),
            voices: ovoices.unwrap_or(Vec::new()),
            hidden: self.hidden,
            effective: self.effective,
        }
    }

    pub fn identifier<'a>(&'a self) -> &'a RIdentifier {
        &self.identifier
    }

    pub fn data<'a>(&'a self) -> &'a RData {
        &self.data
    }
    pub fn set_data(&self, data: Data) {
        *self.data.borrow_mut() = data;
    }
    pub fn data_string(&self) -> String {
        self.data.borrow().to_string()
    }
    pub fn data_float(&self) -> Result<f32, failure::Error> {
        self.data.borrow().to_f()
    }
    pub fn mute_data<F>(&self, mut mute: F) -> Result<(), failure::Error>
    where
        F: FnMut(&Data) -> Result<Data, failure::Error>,
    {
        let data = self.data.borrow();
        self.set_data(mute(&data)?);
        Ok(())
    }

    pub fn ears<'a>(&'a self) -> &'a Vec<Ear> {
        &self.ears
    }
    pub fn ear(&self, ear_idx: Index) -> &Ear {
        &self.ears[ear_idx]
    }
    pub fn add_ear(&mut self, ear: Ear) {
        self.ears.push(ear);
    }
    pub fn sup_ear_set_with_associated_voice(&mut self, ear_idx: Index, set_idx: Index)-> Result<(), failure::Error> {
        let mut voice_idx = self.voices.len();

        while voice_idx > 0 {
            voice_idx -= 1;
            
            let (a_ear_idx, a_set_idx) = self.voices[voice_idx].get_associated_ear_set();

            if a_ear_idx == ear_idx && a_set_idx == set_idx {
                self.voices.remove(voice_idx);
            }
        }
        for voice_idx in 0..self.voices.len() {
            let (a_ear_idx, a_set_idx) = self.voices[voice_idx].get_associated_ear_set();

            if a_ear_idx == ear_idx && a_set_idx > set_idx {
                self.voices[voice_idx].set_associated_ear_set(ear_idx, a_set_idx - 1);
            }
        }
        self.ears[ear_idx].sup_set(set_idx)
    }

    pub fn buffer_len(&self) -> usize {
        if self.effective {
            AudioFormat::chunk_size()
        }
        else {
            1
        }
    }

    pub fn voice(&self, voice_idx: Index) -> &Voice {
        &self.voices[voice_idx]
    }
    pub fn voices<'a>(&'a self) -> &'a Vec<Voice> {
        &self.voices
    }
    pub fn add_voice(&mut self, voice: Voice) {
        self.voices.push(voice);
    }

    pub fn add_audio_voice(&mut self, tag: Option<&str>, value: f32) {
        self.voices.push(voice::audio(tag, value, self.buffer_len()));
    }

    pub fn add_control_voice(&mut self, tag: Option<&str>, value: f32) {
        self.voices.push(voice::control(tag, value));
    }

    pub fn add_cv_voice(&mut self, tag: Option<&str>, value: f32) {
        self.voices.push(voice::cv(tag, value, self.buffer_len()));
    }

    pub fn add_atom_voice(&mut self, tag: Option<&str>, olv2_handler: Option<&Lv2Handler>) {
        self.voices.push(voice::atom(tag, olv2_handler, self.buffer_len()));
    }

    pub fn sup_voice(&mut self, voice_idx: Index) {
        self.voices.remove(voice_idx);
    }

    pub fn is_hidden(&self) -> bool {
        self.hidden
    }
    pub fn set_hidden(&mut self, hidden: bool) {
        self.hidden = hidden;
    }

    pub fn is_effective(&self) -> bool {
        self.effective
    }

    pub fn set_effective(&mut self, effective: bool) {
        self.effective = effective;
    }

    pub fn ear_set_hum_audio_buffer(
        &self,
        ear_idx: Index,
        set_idx: Index,
        hum_idx: Index,
    ) -> AudioBuf {
        self.ears[ear_idx].get_set_hum_audio_buffer(set_idx, hum_idx)
    }
    pub fn ear_set_audio_buffer(&self, ear_idx: Index, set_idx: Index) -> AudioBuf {
        self.ear_set_hum_audio_buffer(ear_idx, set_idx, 0)
    }
    pub fn ear_audio_buffer(&self, ear_idx: Index) -> AudioBuf {
        self.ear_set_audio_buffer(ear_idx, 0)
    }

    pub fn ear_set_hum_cv_buffer(&self, ear_idx: Index, set_idx: Index, hum_idx: Index) -> CvBuf {
        self.ears[ear_idx].get_set_hum_cv_buffer(set_idx, hum_idx)
    }
    pub fn ear_set_cv_buffer(&self, ear_idx: Index, set_idx: Index) -> CvBuf {
        self.ear_set_hum_cv_buffer(ear_idx, set_idx, 0)
    }
    pub fn ear_cv_buffer(&self, ear_idx: Index) -> CvBuf {
        self.ear_set_cv_buffer(ear_idx, 0)
    }

    pub fn listen(&self, tick: i64, len: usize) -> usize {
        let mut ln = len;
        for ear in &self.ears {
            ln = ear.listen(tick, ln);
        }
        ln
    }
}
impl Identifiable for TalkerBase {
    fn id(&self) -> Id {
        self.identifier.borrow().id()
    }
    fn set_id(&self, id: Id) {
        self.identifier.borrow_mut().set_id(id);
    }
    fn name(&self) -> String {
        self.identifier.borrow().name().to_string()
    }
    fn set_name(&self, name: &str) {
        self.identifier.borrow_mut().set_name(name);
    }
}

pub trait Talker {
    fn activate(&mut self) {}
    fn deactivate(&mut self) {}

    fn set_data_update(
        &mut self,
        base: &TalkerBase,
        data: Data,
    ) -> Result<Option<TalkerBase>, failure::Error> {
        base.set_data(data);
        Ok(None)
    }

    fn add_set_to_ear_update(
        &mut self,
        base: &TalkerBase,
        ear_idx: Index,
        hum_idx: Index,
        entree: Entree,
    ) -> Result<Option<TalkerBase>, failure::Error> {
        base.ears[ear_idx].add_set(hum_idx, entree)?;
        Ok(None)
    }

    fn sup_ear_set_update(
        &mut self,
        base: &TalkerBase,
        ear_idx: usize,
        set_idx: usize,
    ) -> Result<Option<TalkerBase>, failure::Error> {
        base.ears[ear_idx].sup_set(set_idx)?;
        Ok(None)
    }

    fn talk(&mut self, _base: &TalkerBase, _port: usize, _tick: i64, _len: usize) -> usize {
        0
    }
}

pub type CTalker = (TalkerBase, Box<dyn Talker>);

#[macro_export]
macro_rules! ctalker {
    ($base:expr, $core:expr) => {
        ($base, Box::new($core))
    };
}

pub struct NilTalker {}
impl NilTalker {
    pub fn new() -> Self {
        Self {}
    }
}
impl Talker for NilTalker {}

pub struct TalkerCab {
    base: TalkerBase,
    core: RefCell<Box<dyn Talker>>,
}

pub type RTalker = Rc<TalkerCab>;

#[macro_export]
macro_rules! rtalker {
    ($ctalker:expr) => {
        TalkerCab::new_ref($ctalker)
    };
}

impl TalkerCab {
    pub fn new(ctalker: CTalker) -> Self {
        let (base, core) = ctalker;
        Self {
            base,
            core: RefCell::new(core),
        }
    }
    pub fn new_ref(ctalker: CTalker) -> RTalker {
        Rc::new(TalkerCab::new(ctalker))
    }

    fn update(&self, obase: Option<TalkerBase>) -> Result<Option<RTalker>, failure::Error> {
        match obase {
            Some(base) => {
                if base.id() != self.base.id() {
                    base.set_id(self.base.id());
                }
                let core = self.core.replace(Box::new(NilTalker::new()));
                Ok(Some(TalkerCab::new_ref((base, core))))
            }
            None => Ok(None),
        }
    }

    pub fn identifier(&self) -> &RIdentifier {
        self.base.identifier()
    }

    pub fn is(&self, id: Id) -> bool {
        self.base.identifier.borrow().is(id)
    }

    pub fn depends_of(&self, id: Id) -> bool {
        for ear in &self.base.ears {
            if ear.depends_of(id) {
                return true;
            }
        }
        false
    }

    pub fn is_hidden(&self) -> bool {
        self.base.is_hidden()
    }

    pub fn set_effective(&mut self, effective: bool) {
        self.base.effective = effective;
    }

    pub fn data(&self) -> &RData {
        self.base.data()
    }
    pub fn data_string(&self) -> String {
        self.base.data.borrow().to_string()
    }
    pub fn data_float(&self) -> Result<f32, failure::Error> {
        self.base.data.borrow().to_f()
    }

    pub fn set_data_from_string_update(&self, s: &str) -> Result<Option<RTalker>, failure::Error> {
        let data = self.base.data.borrow().birth(s)?;
        let obase = self.core.borrow_mut().set_data_update(&self.base, data)?;
        self.update(obase)
    }

    pub fn ear(&self, ear_idx: Index) -> &Ear {
        self.base.ear(ear_idx)
    }
    pub fn ears<'a>(&'a self) -> &'a Vec<Ear> {
        &self.base.ears
    }

    pub fn voice(&self, voice_idx: Index) -> &Voice {
        self.base.voice(voice_idx)
    }
    pub fn voices<'a>(&'a self) -> &'a Vec<Voice> {
        &self.base.voices
    }
    pub fn voice_port_type(&self, port: usize) -> PortType {
        self.voice(port).port_type()
    }

    pub fn voice_tag(&self, port: usize) -> Result<String, failure::Error> {
        match self.base.voices.get(port) {
            Some(voice) => Ok(voice.tag().to_string()),
            None => Err(failure::err_msg(format!(
                "Unknow voice {} for talker {}",
                port,
                self.name()
            ))),
        }
    }

    pub fn ear_set_hum_audio_buffer(
        &self,
        ear_idx: Index,
        set_idx: Index,
        hum_idx: Index,
    ) -> AudioBuf {
        self.base.ears[ear_idx].get_set_hum_audio_buffer(set_idx, hum_idx)
    }
    pub fn ear_set_audio_buffer(&self, ear_idx: Index, set_idx: Index) -> AudioBuf {
        self.ear_set_hum_audio_buffer(ear_idx, set_idx, 0)
    }
    pub fn ear_audio_buffer(&self, ear_idx: Index) -> AudioBuf {
        self.ear_set_audio_buffer(ear_idx, 0)
    }

    pub fn ear_set_hum_control_buffer(
        &self,
        ear_idx: Index,
        set_idx: Index,
        hum_idx: Index,
    ) -> AudioBuf {
        self.base.ears[ear_idx].get_set_hum_control_buffer(set_idx, hum_idx)
    }
    pub fn ear_set_control_buffer(&self, ear_idx: Index, set_idx: Index) -> AudioBuf {
        self.ear_set_hum_control_buffer(ear_idx, set_idx, 0)
    }
    pub fn ear_control_buffer(&self, ear_idx: Index) -> AudioBuf {
        self.ear_set_control_buffer(ear_idx, 0)
    }

    pub fn ear_set_hum_cv_buffer(&self, ear_idx: Index, set_idx: Index, hum_idx: Index) -> CvBuf {
        self.base.ears[ear_idx].get_set_hum_cv_buffer(set_idx, hum_idx)
    }
    pub fn ear_set_cv_buffer(&self, ear_idx: Index, set_idx: Index) -> CvBuf {
        self.ear_set_hum_cv_buffer(ear_idx, set_idx, 0)
    }
    pub fn ear_cv_buffer(&self, ear_idx: Index) -> CvBuf {
        self.ear_set_cv_buffer(ear_idx, 0)
    }

    pub fn voice_value(&self, port: usize) -> Option<f32> {
        if self.base.is_hidden() {
            if let Some(voice) = self.base.voices().get(port) {
                if voice.can_have_a_value() {
                    return Some(voice.value(0));
                }
            }
        }
        None
    }

    pub fn set_ear_hum_value(
        &self,
        ear_idx: Index,
        set_idx: Index,
        hum_idx: Index,
        value: f32,
    ) -> Result<(), failure::Error> {
        self.base.ears[ear_idx].set_hum_value(set_idx, hum_idx, value)
    }

    pub fn set_ear_hum_voice(
        &self,
        ear_idx: Index,
        set_idx: Index,
        hum_idx: Index,
        talker: &RTalker,
        port: usize,
    ) -> Result<(), failure::Error> {
        self.base.ears[ear_idx].set_hum_voice(set_idx, hum_idx, talker, port)
    }

    pub fn set_ear_talk_value(
        &self,
        ear_idx: Index,
        set_idx: Index,
        hum_idx: Index,
        talk_idx: Index,
        value: f32,
    ) -> Result<(), failure::Error> {
        self.base.ears[ear_idx].set_talk_value(set_idx, hum_idx, talk_idx, value)
    }

    pub fn set_ear_talk_voice(
        &self,
        ear_idx: Index,
        set_idx: Index,
        hum_idx: Index,
        talk_idx: Index,
        talker: &RTalker,
        port: usize,
    ) -> Result<(), failure::Error> {
        self.base.ears[ear_idx].set_talk_voice(set_idx, hum_idx, talk_idx, talker, port)
    }

    pub fn add_value_to_ear_hum(
        &self,
        ear_idx: Index,
        set_idx: Index,
        hum_idx: Index,
        value: f32,
    ) -> Result<(), failure::Error> {
        self.base.ears[ear_idx].add_value_to_hum(set_idx, hum_idx, value)
    }

    pub fn add_voice_to_ear_hum(
        &self,
        ear_idx: Index,
        set_idx: Index,
        hum_idx: Index,
        voice_talker: &RTalker,
        port: usize,
    ) -> Result<(), failure::Error> {
        self.base.ears[ear_idx].add_voice_to_hum(set_idx, hum_idx, voice_talker, port)
    }
    pub fn sup_ear_talk(
        &self,
        ear_idx: Index,
        set_idx: Index,
        hum_idx: Index,
        talk_idx: Index,
    ) -> Result<(), failure::Error> {
        self.base.ears[ear_idx].sup_talk(set_idx, hum_idx, talk_idx)
    }

    pub fn add_set_value_to_ear_update(
        &self,
        ear_idx: Index,
        hum_idx: Index,
        value: f32,
    ) -> Result<Option<RTalker>, failure::Error> {
        let obase = self
            .core
            .borrow_mut()
            .add_set_to_ear_update(&self.base, ear_idx, hum_idx, Entree::Value(value))?;
        self.update(obase)
    }
    pub fn add_set_voice_to_ear_update(
        &self,
        ear_idx: Index,
        hum_idx: Index,
        voice_talker: &RTalker,
        port: usize,
    ) -> Result<Option<RTalker>, failure::Error> {
        let obase = self.core.borrow_mut().add_set_to_ear_update(
            &self.base,
            ear_idx,
            hum_idx,
            Entree::Voice(voice_talker, port),
        )?;
        self.update(obase)
    }

    pub fn sup_ear_set_update(
        &self,
        ear_idx: usize,
        set_idx: usize,
    ) -> Result<Option<RTalker>, failure::Error> {
        let obase = self
            .core
            .borrow_mut()
            .sup_ear_set_update(&self.base, ear_idx, set_idx)?;
        self.update(obase)
    }

    pub fn listen(&self, tick: i64, len: usize) -> usize {
        self.base.listen(tick, len)
    }

    pub fn model(&self) -> String {
        self.base.identifier.borrow().model().to_string()
    }

    pub fn activate(&self) {
        self.core.borrow_mut().activate()
    }
    pub fn deactivate(&self) {
        self.core.borrow_mut().deactivate()
    }

    pub fn talk(&self, port: usize, tick: i64, len: usize) -> usize {
        let ln = self.core.borrow_mut().talk(&self.base, port, tick, len);

        if ln > 0 {
            self.base.voices[port].set_tick_len(tick, ln);
        }
        ln
    }

    pub fn backup<'a>(&'a self) -> (String, String, &Vec<ear::Ear>) {
        (self.model(), self.data_string(), &self.base.ears)
    }
}

impl Identifiable for TalkerCab {
    fn id(&self) -> Id {
        self.base.id()
    }
    fn set_id(&self, id: Id) {
        self.base.set_id(id);
    }
    fn name(&self) -> String {
        self.base.name()
    }
    fn set_name(&self, name: &str) {
        self.base.set_name(name);
    }
}

pub struct MuteTalker {}

impl MuteTalker {
    pub fn new(base: TalkerBase) -> RTalker {
        TalkerCab::new_ref(ctalker!(base, Self {}))
    }
}

impl Talker for MuteTalker {}
