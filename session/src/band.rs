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
use std::fmt::Write as FmtWrite;
// use std::io::BufReader;
use std::rc::Rc;
use std::str::FromStr;

use talker::ear::{Ear, Talk};
use talker::identifier::{Id, Identifier, Index};
use talker::talker::{RTalker, Talker};

use crate::factory::Factory;
use crate::mixer;
use crate::mixer::RMixer;
use crate::output;
use crate::output::ROutput;
use crate::track;
use crate::track::{RTrack, Track};

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

struct Module<'a> {
    kind: &'a str,
    mref: &'a str,
    feature: &'a str,
    attributs: Vec<(&'a str, &'a str, &'a str)>,
}
impl<'a> Module<'a> {
    pub fn new(kind: &'a str, mref: &'a str, feature: &'a str) -> Module<'a> {
        Self {
            kind,
            mref,
            feature,
            attributs: Vec::new(),
        }
    }
}

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

    fn mref(id: Id, name: &str) -> String {
        format!("{}#{}", id, name.replace(" ", "_").replace("\t", "_"))
    }

    fn name_from_mref(mref: &str) -> &str {
        let parts: Vec<&str> = mref.split('#').collect();

        if parts.len() == 2 {
            parts[1]
        } else {
            mref
        }
    }

    fn id_from_mref(mref: &str) -> Result<Id, failure::Error> {
        let parts: Vec<&str> = mref.split('#').collect();
        match Id::from_str(parts[0]) {
            Ok(id) => Ok(id),
            Err(e) => Err(failure::err_msg(format!(
                "Failed to get id from mref {} : {}!",
                mref,
                e.to_string()
            ))),
        }
    }

    fn parse_talk_tag<'a>(tag: &'a str) -> (&'a str, Index, &'a str) {
        let mut set_idx: Index = 0;
        let mut hum_tag = "";
        let parts: Vec<&str> = tag.split('.').collect();

        if parts.len() > 1 {
            set_idx = Index::from_str(parts[1]).unwrap_or(0);
        }
        if parts.len() > 2 {
            hum_tag = parts[2];
        }
        (parts[0], set_idx, hum_tag)
    }

    fn tidy_decs<'a>(
        module: Module<'a>,
        (tkr_decs, trk_decs, mxr_decs, otp_decs): &mut (
            HashMap<&'a str, Module<'a>>,
            HashMap<&'a str, Module<'a>>,
            HashMap<&'a str, Module<'a>>,
            HashMap<&'a str, Module<'a>>,
        ),
    ) {
        match module.kind {
            "" => None,
            track::KIND => trk_decs.insert(module.mref, module),
            mixer::KIND => mxr_decs.insert(module.mref, module),
            output::KIND => otp_decs.insert(module.mref, module),
            _ => tkr_decs.insert(module.mref, module),
        };
    }

    fn make_decs<'a>(
        description: &'a String,
    ) -> Result<
        (
            HashMap<&'a str, Module<'a>>,
            HashMap<&'a str, Module<'a>>,
            HashMap<&'a str, Module<'a>>,
            HashMap<&'a str, Module<'a>>,
        ),
        failure::Error,
    > {
        let mut decs = (
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
        );
        let mut module = Module::new("", "", "");

        for line in description.lines() {
            let words: Vec<&str> = line.trim().split(|c| c == ' ' || c == '\t').collect();

            match (words.get(0), words.get(1), words.get(2)) {
                (None, None, None) => (),
                (Some(c), _, _) if c.chars().next() == Some('/') => (),
                (Some(p), Some(tag), Some(tlk)) if p == &">" => {
                    let tlk_p: Vec<&str> = tlk.split(':').collect();
                    let tkr = tlk_p.get(0).unwrap_or(tlk);
                    let sp = tlk_p.get(1).unwrap_or(&"");

                    if module.kind == "" {
                        return Err(failure::err_msg(format!(
                            "Found module attribut {} {} before module!",
                            tag, tlk
                        )));
                    }
                    module.attributs.push((tag, tkr, sp));
                }
                (Some(kind), Some(mref), Some(feature)) => {
                    Band::tidy_decs(module, &mut decs);
                    module = Module::new(kind, mref, feature);
                }
                (Some(kind), Some(mref), None) => {
                    Band::tidy_decs(module, &mut decs);
                    module = Module::new(kind, mref, "");
                }
                _ => (),
            }
        }
        Band::tidy_decs(module, &mut decs);
        Ok(decs)
    }

    fn set_talker_ears(
        talkers: &HashMap<&str, RTalker>,
        talker: &RTalker,
        module: &Module,
    ) -> Result<(), failure::Error> {
        for (tag, dpn, tkn) in &module.attributs {
            let (ear_tag, set_idx, hum_tag) = Band::parse_talk_tag(tag);

            match f32::from_str(&dpn) {
                Ok(value) => talker
                    .borrow_mut()
                    .add_ear_hum_value_by_tag(ear_tag, set_idx, hum_tag, value)?,
                Err(_) => match talkers.get(dpn) {
                    Some(tkr) => talker.borrow_mut().add_ear_hum_voice_by_tag(
                        ear_tag,
                        set_idx,
                        hum_tag,
                        tkr,
                        tkr.borrow().voice_port(&tkn)?,
                    )?,
                    None => {
                        return Err(failure::err_msg(format!("Talker {} not found!", dpn)));
                    }
                },
            }
        }
        Ok(())
    }

    fn make_track(
        &mut self,
        factory: &Factory,
        talkers: &HashMap<&str, RTalker>,
        module: &Module,
    ) -> Result<RTrack, failure::Error> {
        let rtrack = factory.make_track(
            Some(Band::id_from_mref(module.mref)?),
            Some(Band::name_from_mref(module.mref)),
        )?;
        {
            let track = rtrack.borrow_mut();

            for (tag, dpn, tkn) in &module.attributs {
                let (ear_tag, set_idx, hum_tag) = Band::parse_talk_tag(&tag);

                match f32::from_str(&dpn) {
                    Ok(value) => {
                        track.add_ear_hum_value_by_tag(ear_tag, set_idx, hum_tag, value)?
                    }
                    Err(_) => match talkers.get(dpn) {
                        Some(tkr) => track.add_ear_hum_voice_by_tag(
                            ear_tag,
                            set_idx,
                            hum_tag,
                            tkr,
                            tkr.borrow().voice_port(&tkn)?,
                        )?,
                        None => {
                            return Err(failure::err_msg(format!("Talker {} not found!", dpn)));
                        }
                    },
                }
            }
        }
        let id = rtrack.borrow().id();
        self.talkers.insert(id, rtrack.clone());
        Ok(rtrack)
    }

    fn make_output(
        &mut self,
        factory: &Factory,
        module: &Module,
    ) -> Result<ROutput, failure::Error> {
        factory.make_output(
            Some(Band::id_from_mref(module.mref)?),
            Some(Band::name_from_mref(module.mref)),
            module.feature,
            Some(&module.attributs),
        )
    }

    fn make_mixer(
        &mut self,
        factory: &Factory,
        talkers: &HashMap<&str, RTalker>,
        trk_decs: &HashMap<&str, Module>,
        otp_decs: &HashMap<&str, Module>,
        module: &Module,
    ) -> Result<RMixer, failure::Error> {
        let mut tracks = Vec::new();
        let mut outputs = Vec::new();

        for (tag, dpn, _) in &module.attributs {
            if tag == &Track::kind() {
                match trk_decs.get(dpn) {
                    Some(trk) => tracks.push(self.make_track(factory, &talkers, trk)?),
                    None => return Err(failure::err_msg(format!("Track {} not found!", dpn))),
                }
            } else if tag == &output::KIND {
                match otp_decs.get(dpn) {
                    Some(otp) => outputs.push(self.make_output(factory, otp)?),
                    None => return Err(failure::err_msg(format!("Output {} not found!", dpn))),
                }
            }
        }

        let rmixer = factory.make_mixer(
            Some(Band::id_from_mref(module.mref)?),
            Some(Band::name_from_mref(module.mref)),
            Some(tracks),
            Some(outputs),
        )?;
        {
            let mixer = rmixer.borrow_mut();

            for (tag, dpn, tkn) in &module.attributs {
                if tag != &Track::kind() && tag != &output::KIND {
                    let (ear_tag, set_idx, hum_tag) = Band::parse_talk_tag(&tag);

                    match f32::from_str(&dpn) {
                        Ok(value) => {
                            mixer.add_ear_hum_value_by_tag(ear_tag, set_idx, hum_tag, value)?
                        }
                        Err(_) => match talkers.get(dpn) {
                            Some(tkr) => mixer.add_ear_hum_voice_by_tag(
                                ear_tag,
                                set_idx,
                                hum_tag,
                                tkr,
                                tkr.borrow().voice_port(&tkn)?,
                            )?,
                            None => {
                                return Err(failure::err_msg(format!("Talker {} not found!", dpn)));
                            }
                        },
                    }
                }
            }
        }
        Ok(rmixer)
    }

    pub fn build(factory: &Factory, description: &String) -> Result<Band, failure::Error> {
        Identifier::initialize_id_count();
        let mut band = Band::empty();

        let (tkr_decs, trk_decs, mxr_decs, otp_decs) = Band::make_decs(&description)?;

        let mut talkers = HashMap::new();
        let mut talkers_modules = Vec::new();

        for (mref, module) in tkr_decs {
            let tkr = band.build_talker(
                factory,
                module.kind,
                Some(Band::id_from_mref(mref)?),
                Some(Band::name_from_mref(mref)),
            )?;

            if module.feature.len() > 0 {
                tkr.borrow_mut().set_data_from_string(module.feature)?;
            }
            talkers_modules.push((tkr.clone(), module));
            talkers.insert(mref, tkr.clone());
        }

        for (talker, module) in talkers_modules {
            Band::set_talker_ears(&talkers, &talker, &module)?;
        }

        for (_, module) in mxr_decs {
            let rmixer = band.make_mixer(factory, &talkers, &trk_decs, &otp_decs, &module)?;
            band.add_mixer(rmixer);
        }

        Ok(band)
    }
    pub fn make(description_buffer: &String) -> Result<Band, failure::Error> {
        Factory::visit(|factory| Band::build(factory, description_buffer))
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
        let tkr = &talk.talker().borrow();

        if tkr.is_hidden() {
            writeln!(buf, "{} {}", talk_tag, tkr.data_string())?;
        } else {
            let voice_tag = tkr.voice_tag(talk.port())?;

            if voice_tag == "" {
                writeln!(buf, "{} {}", talk_tag, Band::mref(tkr.id(), &tkr.name()))?;
            } else {
                writeln!(
                    buf,
                    "{} {}:{}",
                    talk_tag,
                    Band::mref(tkr.id(), &tkr.name()),
                    voice_tag
                )?;
            }
        }
        Ok(buf)
    }

    pub fn serialize(&self) -> Result<String, failure::Error> {
        let mut buf = String::new();

        for rtkr in self.talkers.values() {
            let tkr = rtkr.borrow();
            let (model, feature, ears): (&str, String, &Vec<Ear>) = tkr.backup();

            writeln!(
                buf,
                "\n{} {} {}",
                model,
                Band::mref(tkr.id(), &tkr.name()),
                feature
            )?;

            for ear in ears {
                ear.fold_talks(Band::talk_dep_line, &mut buf)?;
            }
        }

        for rmixer in self.mixers.values() {
            let mixer = rmixer.borrow();
            /*
                        for trk in mixer.tracks() {
                            writeln!(
                                buf,
                                "\n{} {}",
                                track::KIND,
                                Band::mref(trk.borrow().id(), &trk.borrow().name())
                            )?;

                            for ear in trk.borrow().ears() {
                                ear.fold_talks(Band::talk_dep_line, &mut buf)?;
                            }
                        }
            */
            writeln!(
                buf,
                "\n{} {}",
                mixer::KIND,
                Band::mref(mixer.id(), &mixer.name())
            )?;

            for ear in mixer.ears() {
                ear.fold_talks(Band::talk_dep_line, &mut buf)?;
            }
            /*
                        for trk in mixer.tracks() {
                            writeln!(
                                buf,
                                "> {} {}",
                                track::KIND,
                                Band::mref(trk.borrow().id(), &trk.borrow().name())
                            )?;
                        }
            */
            for routput in mixer.outputs() {
                let output = routput.borrow();
                let (kind, _, _) = output.backup();
                writeln!(
                    buf,
                    "> {} {}",
                    kind,
                    Band::mref(output.id(), &output.name())
                )?;
            }

            for routput in mixer.outputs() {
                let output = routput.borrow();
                let (kind, model, properties) = output.backup();
                writeln!(
                    buf,
                    "\n{} {} {}",
                    kind,
                    Band::mref(output.id(), &output.name()),
                    model
                )?;

                for (tag, value) in properties {
                    writeln!(buf, "> {} {}", tag, value)?;
                }
            }
        }
        Ok(buf)
    }

    pub fn add_mixer(&mut self, rmixer: RMixer) {
        let id = rmixer.borrow().id();
        self.talkers.insert(id, rmixer.clone());

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
        self.talkers.insert(tkr.borrow().id(), tkr.clone());
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

    pub fn sup_talker(&self, _talker_id: &Id) -> Result<(), failure::Error> {
        // TODO
        //                let tkr = self.fetch_talker(talker_id)?;
        Ok(())
    }

    pub fn add_output(&mut self, model: &str) -> Result<(), failure::Error> {
        Factory::visit(|factory| {
            for rmixer in self.mixers.values() {
                rmixer
                    .borrow_mut()
                    .add_output(factory.make_output(None, None, model, None)?);
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
                let tkr = self.fetch_talker(tkr_id)?;

                tkr.borrow_mut().deactivate();

                tkr.borrow_mut().set_data_from_string(&data)?;

                tkr.borrow_mut().activate();
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

                ear_tkr.borrow_mut().deactivate();

                ear_tkr.borrow().set_ear_hum_voice(
                    *ear_idx,
                    *set_idx,
                    *hum_idx,
                    &voice_tkr,
                    *voice_port,
                )?;

                ear_tkr.borrow_mut().activate();
            }
            Operation::SetEarHumValue(ear_tkr_id, ear_idx, set_idx, hum_idx, value) => {
                let ear_tkr = self.fetch_talker(ear_tkr_id)?;

                ear_tkr.borrow_mut().deactivate();

                ear_tkr
                    .borrow()
                    .set_ear_hum_value(*ear_idx, *set_idx, *hum_idx, *value)?;

                ear_tkr.borrow_mut().activate();
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

                ear_tkr.borrow_mut().deactivate();

                ear_tkr.borrow().set_ear_talk_voice(
                    *ear_idx,
                    *set_idx,
                    *hum_idx,
                    *talk_idx,
                    &voice_tkr,
                    *voice_port,
                )?;

                ear_tkr.borrow_mut().activate();
            }
            Operation::SetEarTalkValue(ear_tkr_id, ear_idx, set_idx, hum_idx, talk_idx, value) => {
                let ear_tkr = self.fetch_talker(ear_tkr_id)?;

                ear_tkr.borrow_mut().deactivate();

                ear_tkr
                    .borrow()
                    .set_ear_talk_value(*ear_idx, *set_idx, *hum_idx, *talk_idx, *value)?;

                ear_tkr.borrow_mut().activate();
            }
            Operation::AddValueToEarHum(ear_tkr_id, ear_idx, set_idx, hum_idx, value) => {
                let ear_tkr = self.fetch_talker(ear_tkr_id)?;

                ear_tkr.borrow_mut().deactivate();

                ear_tkr
                    .borrow()
                    .add_value_to_ear_hum(*ear_idx, *set_idx, *hum_idx, *value)?;

                ear_tkr.borrow_mut().activate();
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

                ear_tkr.borrow_mut().deactivate();

                ear_tkr.borrow().add_voice_to_ear_hum(
                    *ear_idx,
                    *set_idx,
                    *hum_idx,
                    &voice_tkr,
                    *voice_port,
                )?;

                ear_tkr.borrow_mut().activate();
            }
            Operation::SupEarTalk(ear_tkr_id, ear_idx, set_idx, hum_idx, talk_idx) => {
                let ear_tkr = self.fetch_talker(ear_tkr_id)?;

                ear_tkr.borrow_mut().deactivate();

                ear_tkr
                    .borrow()
                    .sup_ear_talk(*ear_idx, *set_idx, *hum_idx, *talk_idx)?;

                ear_tkr.borrow_mut().activate();
            }
            Operation::AddSetValueToEar(ear_tkr_id, ear_idx, hum_idx, value) => {
                let ear_tkr = self.fetch_talker(ear_tkr_id)?;

                ear_tkr.borrow_mut().deactivate();

                ear_tkr
                    .borrow()
                    .add_set_value_to_ear(*ear_idx, *hum_idx, *value)?;

                ear_tkr.borrow_mut().activate();
            }
            Operation::AddSetVoiceToEar(ear_tkr_id, ear_idx, hum_idx, voice_tkr_id, voice_port) => {
                let ear_tkr = self.fetch_talker(ear_tkr_id)?;
                let voice_tkr = self.fetch_talker(voice_tkr_id)?;

                ear_tkr.borrow_mut().deactivate();

                ear_tkr.borrow().add_set_voice_to_ear(
                    *ear_idx,
                    *hum_idx,
                    &voice_tkr,
                    *voice_port,
                )?;

                ear_tkr.borrow_mut().activate();
            }
            Operation::SupEarSet(ear_tkr_id, ear_idx, set_idx) => {
                let ear_tkr = self.fetch_talker(ear_tkr_id)?;

                ear_tkr.borrow_mut().deactivate();

                ear_tkr.borrow().sup_ear_set(*ear_idx, *set_idx)?;

                ear_tkr.borrow_mut().activate();
            }
        }
        Ok(())
    }

    pub fn activate_talkers(&self) {
        for tkr in self.talkers.values() {
            tkr.borrow_mut().activate();
        }
    }
    pub fn deactivate_talkers(&self) {
        for tkr in self.talkers.values() {
            tkr.borrow_mut().deactivate();
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
