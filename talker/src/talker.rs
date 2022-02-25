use std::cell::RefCell;
use std::rc::Rc;

extern crate failure;

use crate::data::{Data, RData};
use crate::ear;
use crate::ear::Ear;
use crate::horn::{AudioBuf, CvBuf, PortType};
use crate::identifier::{Id, Identifiable, Identifier, Index, RIdentifier};
use crate::voice::Voice;

pub struct TalkerBase {
    identifier: RIdentifier,
    data: RData,
    ears: Vec<Ear>,
    voices: Vec<Voice>,
    hidden: bool,
}

impl TalkerBase {
    pub fn new_data(name: &str, model: &str, data: Data) -> Self {
        Self {
            identifier: RefCell::new(Identifier::new(name, model)),
            data: RefCell::new(data),
            ears: Vec::new(),
            voices: Vec::new(),
            hidden: false,
        }
    }
    pub fn new(name: &str, model: &str) -> Self {
        TalkerBase::new_data(name, model, Data::Nil)
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
    pub fn add_ear_hum_value_by_tag(
        &self,
        ear_tag: &str,
        set_idx: Index,
        hum_tag: &str,
        value: f32,
    ) -> Result<(), failure::Error> {
        for ear in &self.ears {
            if ear.tag() == ear_tag {
                return ear.add_hum_value_by_tag(set_idx, hum_tag, value);
            }
        }
        Err(failure::err_msg(format!(
            "Talker {} add_ear_hum_value_by_tag : ear {} not found!",
            self.name(),
            ear_tag
        )))
    }
    pub fn add_ear_hum_voice_by_tag(
        &self,
        ear_tag: &str,
        set_idx: Index,
        hum_tag: &str,
        talker: &RTalker,
        port: usize,
    ) -> Result<(), failure::Error> {
        for ear in &self.ears {
            if ear.tag() == ear_tag {
                return ear.add_hum_voice_by_tag(set_idx, hum_tag, talker, port);
            }
        }
        Err(failure::err_msg(format!(
            "Talker {} add_ear_hum_voice_by_tag : ear {} not found!",
            self.name(),
            ear_tag
        )))
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

    pub fn is_hidden(&self) -> bool {
        self.hidden
    }
    pub fn set_hidden(&mut self, hidden: bool) {
        self.hidden = hidden;
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
        let (base, core) = ctalker;
        Rc::new(Self {
            base,
            core: RefCell::new(core),
        })
    }

    pub fn identifier(&self) -> &RIdentifier {
        self.base.identifier()
    }
    pub fn depends_of(&self, id: Id) -> bool {
        self.base.identifier.borrow().id() == id
    }

    pub fn is_hidden(&self) -> bool {
        self.base.is_hidden()
    }

    pub fn data(&self) -> &RData {
        self.base.data()
    }
    pub fn set_data(&self, data: Data) {
        self.base.set_data(data);
    }
    pub fn data_string(&self) -> String {
        self.base.data.borrow().to_string()
    }
    pub fn data_float(&self) -> Result<f32, failure::Error> {
        self.base.data.borrow().to_f()
    }

    pub fn set_data_from_string(&self, s: &str) -> Result<(), failure::Error> {
        match self.base.data.borrow().birth(s) {
            Ok(d) => {
                self.set_data(d);
                Ok(())
            }
            Err(e) => Err(e),
        }
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
    /*
       pub fn voice_port_type_is(&self, port: usize, port_type: PortType) -> bool {
            self.voice_port_type(port) == port_type
        }
       pub fn voice_port_type_can_hear(&self, port: usize, port_type: PortType) -> bool {
            self.voice_port_type(port).can_hear(port_type)
        }
    */
    pub fn voice_port(&self, tag: &str) -> Result<usize, failure::Error> {
        for (port, voice) in self.base.voices.iter().enumerate() {
            if voice.tag() == tag {
                return Ok(port);
            }
        }
        Err(failure::err_msg(format!(
            "Unknow voice {} for talker {}",
            tag,
            self.name()
        )))
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

    pub fn add_ear_hum_value_by_tag(
        &self,
        ear_tag: &str,
        set_idx: Index,
        hum_tag: &str,
        value: f32,
    ) -> Result<(), failure::Error> {
        self.base
            .add_ear_hum_value_by_tag(ear_tag, set_idx, hum_tag, value)
    }
    pub fn add_ear_hum_voice_by_tag(
        &self,
        ear_tag: &str,
        set_idx: Index,
        hum_tag: &str,
        talker: &RTalker,
        port: usize,
    ) -> Result<(), failure::Error> {
        self.base
            .add_ear_hum_voice_by_tag(ear_tag, set_idx, hum_tag, talker, port)
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

    pub fn add_set_value_to_ear(
        &self,
        ear_idx: Index,
        hum_idx: Index,
        value: f32,
    ) -> Result<(), failure::Error> {
        self.base.ears[ear_idx].add_set_value(hum_idx, value)
    }
    pub fn add_set_voice_to_ear(
        &self,
        ear_idx: Index,
        hum_idx: Index,
        voice_talker: &RTalker,
        port: usize,
    ) -> Result<(), failure::Error> {
        self.base.ears[ear_idx].add_set_voice(hum_idx, voice_talker, port)
    }

    pub fn sup_ear_set(&self, ear_idx: usize, set_idx: usize) -> Result<(), failure::Error> {
        self.base.ears[ear_idx].sup_set(set_idx)
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
        self.core.borrow_mut().talk(&self.base, port, tick, len)
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
