/*
 * Copyright (C) 2015 Gaetan Dubreil
 *
 *  All rights reserved.This file is distributed under the terms of the
 *  GNU General Public License version 3.0.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Lesser General Public License for more details.
 *
 * You should have received a copy of the GNU Lesser General Public License
 * along with this program; if not, write to the Free Software
 * Foundation, Inc., 59 Temple Place - Suite 330, Boston, MA 02111-1307, USA.
 */

use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt::Write as FmtWrite;
use std::rc::Rc;

use talker::ear::{Ear, Talk};
use talker::identifier::{Id, Identifiable, Identifier, Index};
use talker::talker::RTalker;

use crate::factory::Factory;
use crate::mixer;
use crate::mixer::RMixer;
use crate::output::ROutput;
use crate::parser;
use crate::parser::{PMixer, POutput, PTalk, PTalker};

#[derive(PartialEq, Debug, Clone)]
pub enum Operation {
    AddTalker(Id, String),
    SupTalker(Id),
    SetTalkerData(Id, String),
    SetEarHumVoice(Id, Index, Index, Index, Id, Index),
    SetEarHumValue(Id, Index, Index, Index, f32),
    SetEarTalkVoice(Id, Index, Index, Index, Index, Id, Index),
    SetEarTalkValue(Id, Index, Index, Index, Index, f32),
    AddValueToEarHum(Id, Index, Index, Index, f32),
    AddVoiceToEarHum(Id, Index, Index, Index, Id, Index),
    SupEarTalk(Id, Index, Index, Index, Index),
    AddSetValueToEar(Id, Index, Index, f32),
    AddSetVoiceToEar(Id, Index, Index, Id, Index),
    SupEarSet(Id, Index, Index),
}

pub struct Band {
    talkers: HashMap<Id, RTalker>,
    mixers: HashMap<Id, RMixer>,
}

pub type RBand = Rc<RefCell<Band>>;

impl Band {
    pub fn new(talkers: Option<HashMap<Id, RTalker>>, mixers: Option<HashMap<Id, RMixer>>) -> Band {
        Self {
            talkers: talkers.unwrap_or(HashMap::new()),
            mixers: mixers.unwrap_or(HashMap::new()),
        }
    }

    pub fn empty() -> Band {
        Self {
            talkers: HashMap::new(),
            mixers: HashMap::new(),
        }
    }

    pub fn new_ref(
        talkers: Option<HashMap<Id, RTalker>>,
        mixers: Option<HashMap<Id, RMixer>>,
    ) -> RBand {
        Rc::new(RefCell::new(Band::new(talkers, mixers)))
    }

    pub fn talkers<'a>(&'a self) -> &'a HashMap<Id, RTalker> {
        &self.talkers
    }
    pub fn mixers<'a>(&'a self) -> &'a HashMap<Id, RMixer> {
        &self.mixers
    }

    fn set_talker_ears(
        talkers: &HashMap<Id, RTalker>,
        talker: &RTalker,
        ptalker: &PTalker,
    ) -> Result<(), failure::Error> {
        for cnx in &ptalker.connections {
            match &cnx.talk {
                PTalk::Value(value) => talker.set_ear_hum_value_by_tag(
                    cnx.ear_tag,
                    cnx.set_idx,
                    cnx.hum_tag,
                    *value,
                )?,
                PTalk::TalkerVoice(talker_voice) => match talkers.get(&talker_voice.talker) {
                    Some(tkr) => talker.set_ear_hum_voice_by_tag(
                        cnx.ear_tag,
                        cnx.set_idx,
                        cnx.hum_tag,
                        tkr,
                        tkr.voice_port(&talker_voice.voice)?,
                    )?,
                    None => {
                        return Err(failure::err_msg(format!(
                            "Talker {} not found!",
                            talker_voice.talker
                        )));
                    }
                },
            }
        }
        Ok(())
    }

    fn make_output(
        &mut self,
        factory: &Factory,
        poutput: &POutput,
    ) -> Result<ROutput, failure::Error> {
        factory.make_output(
            poutput.model,
            Some(poutput.id),
            Some(poutput.name),
            poutput.data,
        )
    }

    fn make_mixer(
        &mut self,
        factory: &Factory,
        talkers: &HashMap<Id, RTalker>,
        poutputs: &HashMap<Id, POutput>,
        pmixer: &PMixer,
    ) -> Result<RMixer, failure::Error> {
        let mut outputs = Vec::new();

        for output_id in &pmixer.outputs {
            match poutputs.get(output_id) {
                Some(otp) => outputs.push(self.make_output(factory, otp)?),
                None => return Err(failure::err_msg(format!("Output {} not found!", output_id))),
            }
        }

        let rmixer = factory.make_mixer(Some(pmixer.id), Some(pmixer.name), None, Some(outputs))?;
        {
            let mixer = rmixer.borrow_mut();

            for cnx in &pmixer.connections {
                match &cnx.talk {
                    PTalk::Value(value) => mixer.talker().set_ear_hum_value_by_tag(
                        cnx.ear_tag,
                        cnx.set_idx,
                        cnx.hum_tag,
                        *value,
                    )?,
                    PTalk::TalkerVoice(talker_voice) => match talkers.get(&talker_voice.talker) {
                        Some(tkr) => mixer.talker().set_ear_hum_voice_by_tag(
                            cnx.ear_tag,
                            cnx.set_idx,
                            cnx.hum_tag,
                            tkr,
                            tkr.voice_port(&talker_voice.voice)?,
                        )?,
                        None => {
                            return Err(failure::err_msg(format!(
                                "Talker {} not found!",
                                talker_voice.talker
                            )));
                        }
                    },
                }
            }
        }
        Ok(rmixer)
    }

    pub fn build(factory: &Factory, source: &String) -> Result<Band, failure::Error> {
        Identifier::initialize_id_count();
        let mut band = Band::empty();

        let (ptalkers, pmixers, poutputs) = parser::parse(&source)?;

        let mut talkers = HashMap::new();
        let mut talkers_ptalkers = Vec::new();

        for ptalker in ptalkers.values() {
            let tkr =
                band.build_talker(factory, ptalker.model, Some(ptalker.id), Some(ptalker.name))?;

            if let Some(data) = ptalker.data {
                tkr.set_data_from_string(data)?;
            }
            talkers_ptalkers.push((tkr.clone(), ptalker));
            talkers.insert(ptalker.id, tkr.clone());
        }

        for (talker, ptalker) in talkers_ptalkers {
            Band::set_talker_ears(&talkers, &talker, &ptalker)?;
        }

        for pmixer in pmixers.values() {
            let rmixer = band.make_mixer(factory, &talkers, &poutputs, &pmixer)?;
            band.add_mixer(rmixer);
        }

        Ok(band)
    }
    pub fn make(source_buffer: &String) -> Result<Band, failure::Error> {
        Factory::visit(|factory| Band::build(factory, source_buffer))
    }

    pub fn to_ref(self) -> RBand {
        Rc::new(RefCell::new(self))
    }

    fn talk_dep_line<'a>(
        ear_tag: &str,
        set_idx: Index,
        hum_tag: &str,
        talk: &Talk,
        buf: &'a mut String,
    ) -> Result<&'a mut String, failure::Error> {
        let talk_tag = format!("> {}.{}.{}", ear_tag, set_idx, hum_tag);
        let tkr = &talk.talker();

        if tkr.is_hidden() {
            writeln!(buf, "{} {}", talk_tag, tkr.data_string())?;
        } else {
            let voice_tag = tkr.voice_tag(talk.port())?;

            if voice_tag == "" {
                writeln!(buf, "{} {}", talk_tag, tkr.id())?;
            } else {
                writeln!(buf, "{} {}:{}", talk_tag, tkr.id(), voice_tag)?;
            }
        }
        Ok(buf)
    }

    pub fn serialize(&self) -> Result<String, failure::Error> {
        let mut buf = String::new();

        for tkr in self.talkers.values() {
            //            let tkr = rtkr;
            if tkr.model() != mixer::KIND {
                let (model, data, ears): (String, String, &Vec<Ear>) = tkr.backup();

                writeln!(buf, "\n{} {}#{}", model, tkr.id(), &tkr.name(),)?;

                if !data.is_empty() {
                    writeln!(buf, "[:{}:]", data)?;
                }
                for ear in ears {
                    ear.fold_talks(Band::talk_dep_line, &mut buf)?;
                }
            }
        }

        for rmixer in self.mixers.values() {
            let mixer = rmixer.borrow();
            writeln!(buf, "\n{} {}#{}", mixer::KIND, mixer.id(), &mixer.name())?;

            for ear in mixer.talker().ears() {
                ear.fold_talks(Band::talk_dep_line, &mut buf)?;
            }
            for routput in mixer.outputs() {
                let output = routput.borrow();
                writeln!(buf, "< {}", output.id())?;
            }

            for routput in mixer.outputs() {
                let output = routput.borrow();
                let (kind, model, configuration) = output.backup();
                writeln!(
                    buf,
                    "\n{} {} {}#{}",
                    kind,
                    model,
                    output.id(),
                    &output.name(),
                )?;
                if !configuration.is_empty() {
                    writeln!(buf, "[:{}:]", configuration)?;
                }
            }
        }
        Ok(buf)
    }

    pub fn add_mixer(&mut self, rmixer: RMixer) {
        let id = rmixer.borrow().id();
        self.talkers.insert(id, rmixer.borrow().talker().clone());

        self.mixers.insert(id, rmixer);
    }

    pub fn nb_channels(&self) -> usize {
        let mut nb_channels = 0;

        for rmixer in self.mixers.values() {
            for routput in rmixer.borrow().outputs() {
                let nc = routput.borrow().nb_channels();

                if nc > nb_channels {
                    nb_channels = nc
                }
            }
        }
        nb_channels
    }

    fn build_talker(
        &mut self,
        factory: &Factory,
        model: &str,
        oid: Option<Id>,
        oname: Option<&str>,
    ) -> Result<RTalker, failure::Error> {
        let tkr = factory.make_talker(model, oid, oname)?;
        self.talkers.insert(tkr.id(), tkr.clone());
        Ok(tkr)
    }

    pub fn add_talker(
        &mut self,
        model: &str,
        oid: Option<Id>,
        oname: Option<&str>,
    ) -> Result<RTalker, failure::Error> {
        if let Some(id) = oid {
            println!("Band.add_talker {}", id);
        }
        Factory::visit(|factory| {
            let tkr = self.build_talker(factory, model, oid, oname)?;
            Ok(tkr)
        })
    }

    pub fn replace_talker(
        &mut self,
        talker_id: &Id,
        new_talker: RTalker,
    ) -> Result<(), failure::Error> {
        let mut invalidated_ports;
        {
            let old_talker = self.fetch_talker(talker_id)?;
            let old_voices = old_talker.voices();
            let new_voices = new_talker.voices();
            let old_ports_count = old_voices.len();
            let new_ports_count = new_voices.len();
            invalidated_ports = HashSet::with_capacity(old_ports_count);

            let less_ports = usize::min(old_ports_count, new_ports_count);

            for p in 0..less_ports {
                if new_voices[p].port_type() != old_voices[p].port_type() {
                    invalidated_ports.insert(p);
                }
            }
            for p in less_ports..new_ports_count {
                invalidated_ports.insert(p);
            }
        }
        self.talkers.remove(talker_id);

        for tkr in self.talkers.values() {
            for ear in tkr.ears() {
                if ear.is_listening_talker(*talker_id) {
                    ear.sup_talker_ports(*talker_id, &invalidated_ports)?;

                    if ear.is_listening_talker(*talker_id) {
                        ear.replace_talker(*talker_id, &new_talker)?;
                    }
                }
            }
        }
        self.talkers.insert(new_talker.id(), new_talker);
        Ok(())
    }

    pub fn sup_talker(&mut self, talker_id: &Id) -> Result<(), failure::Error> {
        self.talkers.remove(talker_id);

        for tkr in self.talkers.values() {
            for ear in tkr.ears() {
                if ear.is_listening_talker(*talker_id) {
                    ear.sup_talker(*talker_id)?;
                }
            }
        }
        Ok(())
    }

    pub fn add_output(&mut self, model: &str) -> Result<(), failure::Error> {
        Factory::visit(|factory| {
            for rmixer in self.mixers.values() {
                rmixer
                    .borrow_mut()
                    .add_output(factory.make_output(model, None, None, None)?);
            }
            Ok(())
        })
    }

    pub fn sup_output(&mut self, model: &str) -> Result<(), failure::Error> {
        for rmixer in self.mixers.values() {
            rmixer.borrow_mut().remove_output(model);
        }
        Ok(())
    }

    pub fn fetch_talker<'a>(&'a self, talker_id: &Id) -> Result<&'a RTalker, failure::Error> {
        match self.talkers.get(talker_id) {
            Some(tkr) => Ok(tkr),
            None => Err(failure::err_msg(format!("Talker {} not found!", talker_id))),
        }
    }

    pub fn modify(&mut self, operation: &Operation) -> Result<(), failure::Error> {
        match operation {
            Operation::AddTalker(tkr_id, model) => {
                self.add_talker(&model, Some(*tkr_id), None)?;
            }
            Operation::SupTalker(tkr_id) => {
                self.sup_talker(tkr_id)?;
            }
            Operation::SetTalkerData(tkr_id, data) => {
                let onew_tkr;
                {
                    let tkr = self.fetch_talker(tkr_id)?;

                    tkr.deactivate();
                    onew_tkr = tkr.update_with_data_string(&data)?;
                    if onew_tkr.is_none() {
                        tkr.activate();
                    }
                }

                if let Some(new_tkr) = onew_tkr {
                    new_tkr.activate();
                    self.replace_talker(tkr_id, new_tkr)?;
                }
            }
            Operation::SetEarHumVoice(
                ear_tkr_id,
                ear_idx,
                set_idx,
                hum_idx,
                voice_tkr_id,
                voice_port,
            ) => {
                let ear_tkr = self.fetch_talker(ear_tkr_id)?;
                let voice_tkr = self.fetch_talker(voice_tkr_id)?;

                ear_tkr.deactivate();

                ear_tkr.set_ear_hum_voice(*ear_idx, *set_idx, *hum_idx, &voice_tkr, *voice_port)?;

                ear_tkr.activate();
            }
            Operation::SetEarHumValue(ear_tkr_id, ear_idx, set_idx, hum_idx, value) => {
                let ear_tkr = self.fetch_talker(ear_tkr_id)?;

                ear_tkr.deactivate();

                ear_tkr.set_ear_hum_value(*ear_idx, *set_idx, *hum_idx, *value)?;

                ear_tkr.activate();
            }
            Operation::SetEarTalkVoice(
                ear_tkr_id,
                ear_idx,
                set_idx,
                hum_idx,
                talk_idx,
                voice_tkr_id,
                voice_port,
            ) => {
                let ear_tkr = self.fetch_talker(ear_tkr_id)?;
                let voice_tkr = self.fetch_talker(voice_tkr_id)?;

                ear_tkr.deactivate();

                ear_tkr.set_ear_talk_voice(
                    *ear_idx,
                    *set_idx,
                    *hum_idx,
                    *talk_idx,
                    &voice_tkr,
                    *voice_port,
                )?;

                ear_tkr.activate();
            }
            Operation::SetEarTalkValue(ear_tkr_id, ear_idx, set_idx, hum_idx, talk_idx, value) => {
                let ear_tkr = self.fetch_talker(ear_tkr_id)?;

                ear_tkr.deactivate();

                ear_tkr.set_ear_talk_value(*ear_idx, *set_idx, *hum_idx, *talk_idx, *value)?;

                ear_tkr.activate();
            }
            Operation::AddValueToEarHum(ear_tkr_id, ear_idx, set_idx, hum_idx, value) => {
                let ear_tkr = self.fetch_talker(ear_tkr_id)?;

                ear_tkr.deactivate();

                ear_tkr.add_value_to_ear_hum(*ear_idx, *set_idx, *hum_idx, *value)?;

                ear_tkr.activate();
            }
            Operation::AddVoiceToEarHum(
                ear_tkr_id,
                ear_idx,
                set_idx,
                hum_idx,
                voice_tkr_id,
                voice_port,
            ) => {
                let ear_tkr = self.fetch_talker(ear_tkr_id)?;
                let voice_tkr = self.fetch_talker(voice_tkr_id)?;

                ear_tkr.deactivate();

                ear_tkr.add_voice_to_ear_hum(
                    *ear_idx,
                    *set_idx,
                    *hum_idx,
                    &voice_tkr,
                    *voice_port,
                )?;

                ear_tkr.activate();
            }
            Operation::SupEarTalk(ear_tkr_id, ear_idx, set_idx, hum_idx, talk_idx) => {
                let ear_tkr = self.fetch_talker(ear_tkr_id)?;

                ear_tkr.deactivate();

                ear_tkr.sup_ear_talk(*ear_idx, *set_idx, *hum_idx, *talk_idx)?;

                ear_tkr.activate();
            }
            Operation::AddSetValueToEar(ear_tkr_id, ear_idx, hum_idx, value) => {
                let ear_tkr = self.fetch_talker(ear_tkr_id)?;

                ear_tkr.deactivate();

                ear_tkr.add_set_value_to_ear(*ear_idx, *hum_idx, *value)?;

                ear_tkr.activate();
            }
            Operation::AddSetVoiceToEar(ear_tkr_id, ear_idx, hum_idx, voice_tkr_id, voice_port) => {
                let ear_tkr = self.fetch_talker(ear_tkr_id)?;
                let voice_tkr = self.fetch_talker(voice_tkr_id)?;

                ear_tkr.deactivate();

                ear_tkr.add_set_voice_to_ear(*ear_idx, *hum_idx, &voice_tkr, *voice_port)?;

                ear_tkr.activate();
            }
            Operation::SupEarSet(ear_tkr_id, ear_idx, set_idx) => {
                let ear_tkr = self.fetch_talker(ear_tkr_id)?;

                ear_tkr.deactivate();

                ear_tkr.sup_ear_set(*ear_idx, *set_idx)?;

                ear_tkr.activate();
            }
        }
        Ok(())
    }

    pub fn activate_talkers(&self) {
        for tkr in self.talkers.values() {
            tkr.activate();
        }
    }
    pub fn deactivate_talkers(&self) {
        for tkr in self.talkers.values() {
            tkr.deactivate();
        }
    }

    pub fn open(&mut self) -> Result<(), failure::Error> {
        self.activate_talkers();
        for rmixer in self.mixers.values() {
            rmixer.borrow_mut().open()?;
        }
        Ok(())
    }

    pub fn pause(&mut self) -> Result<(), failure::Error> {
        for rmixer in self.mixers.values() {
            rmixer.borrow_mut().pause()?;
        }
        Ok(())
    }

    pub fn run(&mut self) -> Result<(), failure::Error> {
        for rmixer in self.mixers.values() {
            rmixer.borrow_mut().run()?;
        }
        Ok(())
    }

    pub fn close(&mut self) -> Result<(), failure::Error> {
        for rmixer in self.mixers.values() {
            rmixer.borrow_mut().close()?;
        }
        self.deactivate_talkers();
        Ok(())
    }
}
