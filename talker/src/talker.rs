use std::cell::RefCell;
use std::rc::Rc;

extern crate failure;

use crate::data::{Data, RData};
use crate::ear;
use crate::ear::Ear;
use crate::horn::{AudioBuf, CvBuf};
use crate::identifier::{Id, Identifiable, Identifier, Index, RIdentifier};
use crate::voice::MVoice;
use crate::voice::PortType;

pub struct TalkerBase {
    identifier: RIdentifier,
    data: RData,
    ears: Vec<Ear>,
    voices: Vec<MVoice>,
    //    ear_call: bool,
    hidden: bool,
}

impl TalkerBase {
    pub fn new_data(name: &str, model: &str, data: Data) -> Self {
        Self {
            identifier: RefCell::new(Identifier::new(name, model)),
            data: RefCell::new(data),
            ears: Vec::new(),
            voices: Vec::new(),
            //            ear_call: false,
            hidden: false,
        }
    }
    pub fn new(name: &str, model: &str) -> Self {
        TalkerBase::new_data(name, model, Data::Nil)
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

    pub fn ears<'a>(&'a self) -> &'a Vec<Ear> {
        &self.ears
    }
    pub fn add_ear<'a>(&'a mut self, ear: Ear) {
        self.ears.push(ear);
    }
    pub fn voices<'a>(&'a self) -> &'a Vec<MVoice> {
        &self.voices
    }
    pub fn add_voice<'a>(&'a mut self, voice: MVoice) {
        self.voices.push(voice);
    }
    pub fn identifier<'a>(&'a self) -> &'a RIdentifier {
        &self.identifier
    }
    pub fn is_hidden(&self) -> bool {
        self.hidden
    }
    pub fn set_hidden(&mut self, hidden: bool) {
        self.hidden = hidden;
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
    fn base<'a>(&'a self) -> &'a TalkerBase;
    fn identifier<'a>(&'a self) -> &'a RIdentifier {
        self.base().identifier()
    }

    fn id(&self) -> Id {
        self.base().id()
    }
    fn set_id(&self, id: Id) {
        self.base().set_id(id);
    }
    fn name(&self) -> String {
        self.base().name()
    }
    fn set_name(&self, name: &str) {
        self.base().set_name(name);
    }

    fn model(&self) -> &str;
    fn is_hidden(&self) -> bool {
        self.base().is_hidden()
    }
    fn depends_of(&self, id: Id) -> bool {
        self.base().id() == id
    }

    fn data_string(&self) -> String {
        self.base().data_string()
    }
    fn data_float(&self) -> Result<f32, failure::Error> {
        self.base().data_float()
    }
    fn set_data(&mut self, data: Data) -> Result<(), failure::Error> {
        Err(data.notify_incompatibility("Nil"))
    }
    fn set_data_from_string(&mut self, s: &str) -> Result<(), failure::Error> {
        match self.base().data().borrow().birth(s) {
            Ok(d) => {
                self.base().set_data(d);
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    fn ears<'a>(&'a self) -> &'a Vec<Ear> {
        &self.base().ears
    }
    fn voices<'a>(&'a self) -> &'a Vec<MVoice> {
        &self.base().voices
    }
    fn voice<'a>(&'a self, port: usize) -> &'a MVoice {
        &self.voices().get(port).unwrap()
    }
    fn voice_port_type(&self, port: usize) -> PortType {
        self.voice(port).borrow().port_type()
    }
    /*
        fn voice_port_type_is(&self, port: usize, port_type: PortType) -> bool {
            self.voice_port_type(port) == port_type
        }
        fn voice_port_type_can_hear(&self, port: usize, port_type: PortType) -> bool {
            self.voice_port_type(port).can_hear(port_type)
        }
    */
    fn voice_port(&self, tag: &str) -> Result<usize, failure::Error> {
        for (port, voice) in self.voices().iter().enumerate() {
            if voice.borrow().tag() == tag {
                return Ok(port);
            }
        }
        Err(failure::err_msg(format!(
            "Unknow voice {} for talker {}",
            tag,
            self.base().name()
        )))
    }

    fn voice_tag(&self, port: usize) -> Result<String, failure::Error> {
        match self.voices().get(port) {
            Some(voice) => Ok(voice.borrow().tag().to_string()),
            None => Err(failure::err_msg(format!(
                "Unknow voice {} for talker {}",
                port,
                self.base().name()
            ))),
        }
    }

    fn ear_set_hum_audio_buffer(
        &self,
        ear_idx: Index,
        set_idx: Index,
        hum_idx: Index,
    ) -> Option<AudioBuf> {
        (self.ears().get(ear_idx)?).get_set_hum_audio_buffer(set_idx, hum_idx)
    }
    fn ear_set_audio_buffer(&self, ear_idx: Index, set_idx: Index) -> Option<AudioBuf> {
        self.ear_set_hum_audio_buffer(ear_idx, set_idx, 0)
    }
    fn ear_audio_buffer(&self, ear_idx: Index) -> Option<AudioBuf> {
        self.ear_set_audio_buffer(ear_idx, 0)
    }

    fn ear_set_hum_cv_buffer(
        &self,
        ear_idx: Index,
        set_idx: Index,
        hum_idx: Index,
    ) -> Option<CvBuf> {
        (self.ears().get(ear_idx)?).get_set_hum_cv_buffer(set_idx, hum_idx)
    }
    fn ear_set_cv_buffer(&self, ear_idx: Index, set_idx: Index) -> Option<CvBuf> {
        self.ear_set_hum_cv_buffer(ear_idx, set_idx, 0)
    }
    fn ear_cv_buffer(&self, ear_idx: Index) -> Option<CvBuf> {
        self.ear_set_cv_buffer(ear_idx, 0)
    }
    /*
       fn ear_talks(&self, port: usize) -> Option<MTalks> {
           let ear = self.ears().get(port)?;
           ear.talks()
       }
    */

    fn voice_value(&self, _port: usize) -> Option<f32> {
        None
    }

    fn add_ear_hum_value_by_tag(
        &self,
        ear_tag: &str,
        set_idx: Index,
        hum_tag: &str,
        value: f32,
    ) -> Result<(), failure::Error> {
        for ear in self.ears() {
            if ear.tag() == ear_tag {
                return ear.add_hum_value_by_tag(set_idx, hum_tag, value);
            }
        }
        Err(failure::err_msg(format!(
            "Talker {} ear {} not found!",
            self.base().name(),
            ear_tag
        )))
    }
    fn add_ear_hum_voice_by_tag(
        &self,
        ear_tag: &str,
        set_idx: Index,
        hum_tag: &str,
        talker: &RTalker,
        port: usize,
    ) -> Result<(), failure::Error> {
        for ear in self.ears() {
            if ear.tag() == ear_tag {
                return ear.add_hum_voice_by_tag(set_idx, hum_tag, talker, port);
            }
        }
        Err(failure::err_msg(format!(
            "Talker {} ear {} not found!",
            self.base().name(),
            ear_tag
        )))
    }

    fn set_ear_hum_value(
        &self,
        ear_idx: Index,
        set_idx: Index,
        hum_idx: Index,
        value: f32,
    ) -> Result<(), failure::Error> {
        self.ears()[ear_idx].set_hum_value(set_idx, hum_idx, value)
    }

    fn set_ear_hum_voice(
        &self,
        ear_idx: Index,
        set_idx: Index,
        hum_idx: Index,
        talker: &RTalker,
        port: usize,
    ) -> Result<(), failure::Error> {
        self.ears()[ear_idx].set_hum_voice(set_idx, hum_idx, talker, port)
    }

    fn set_ear_talk_value(
        &self,
        ear_idx: Index,
        set_idx: Index,
        hum_idx: Index,
        talk_idx: Index,
        value: f32,
    ) -> Result<(), failure::Error> {
        self.ears()[ear_idx].set_talk_value(set_idx, hum_idx, talk_idx, value)
    }

    fn set_ear_talk_voice(
        &self,
        ear_idx: Index,
        set_idx: Index,
        hum_idx: Index,
        talk_idx: Index,
        talker: &RTalker,
        port: usize,
    ) -> Result<(), failure::Error> {
        self.ears()[ear_idx].set_talk_voice(set_idx, hum_idx, talk_idx, talker, port)
    }

    fn add_value_to_ear_hum(
        &self,
        ear_idx: Index,
        set_idx: Index,
        hum_idx: Index,
        value: f32,
    ) -> Result<(), failure::Error> {
        self.ears()[ear_idx].add_value_to_hum(set_idx, hum_idx, value)
    }

    fn add_voice_to_ear_hum(
        &self,
        ear_idx: Index,
        set_idx: Index,
        hum_idx: Index,
        voice_talker: &RTalker,
        port: usize,
    ) -> Result<(), failure::Error> {
        self.ears()[ear_idx].add_voice_to_hum(set_idx, hum_idx, voice_talker, port)
    }
    fn sup_ear_talk(
        &self,
        ear_idx: Index,
        set_idx: Index,
        hum_idx: Index,
        talk_idx: Index,
    ) -> Result<(), failure::Error> {
        self.ears()[ear_idx].sup_talk(set_idx, hum_idx, talk_idx)
    }

    fn add_set_value_to_ear(
        &self,
        ear_idx: Index,
        hum_idx: Index,
        value: f32,
    ) -> Result<(), failure::Error> {
        self.ears()[ear_idx].add_set_value(hum_idx, value)
    }
    fn add_set_voice_to_ear(
        &self,
        ear_idx: Index,
        hum_idx: Index,
        voice_talker: &RTalker,
        port: usize,
    ) -> Result<(), failure::Error> {
        self.ears()[ear_idx].add_set_voice(hum_idx, voice_talker, port)
    }
    /*
    fn add_ear_value(&self, ear_idx: Index, value: f32) -> Result<(), failure::Error> {
        self.add_ear_hum_value(ear_idx, 0, value)
    }
    fn add_ear_voice(
        &self,
        ear_idx: Index,
        voice_talker: &RTalker,
        port: usize,
    ) -> Result<(), failure::Error> {
        self.add_ear_hum_voice(ear_idx, 0, voice_talker, port)
    }
    */
    fn sup_ear_set(&self, ear_idx: usize, set_idx: usize) -> Result<(), failure::Error> {
        self.ears()[ear_idx].sup_set(set_idx)
    }

    fn activate(&mut self) {}
    fn deactivate(&mut self) {}

    fn talk(&mut self, _port: usize, _tick: i64, _len: usize) -> usize {
        0
    }

    fn listen_ears(&self, tick: i64, len: usize) -> usize {
        let mut ln = len;
        for ear in self.ears() {
            ln = ear.listen(tick, ln);
        }
        ln
    }

    fn backup<'a>(&'a self) -> (&str, String, &Vec<ear::Ear>) {
        (self.model(), self.base().data_string(), self.ears())
    }
}

pub type CTalker = RefCell<dyn Talker>;
pub type RTalker = Rc<CTalker>;
