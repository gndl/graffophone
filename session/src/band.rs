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
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::io::Write;
use std::rc::Rc;
use std::str::FromStr;

use talker::ear::{Ear, Talk};
use talker::identifier::{Id, Identifier};
use talker::talker::{RTalker, Talker};

//use crate::factory;
use crate::factory::Factory;
use crate::mixer;
use crate::mixer::RMixer;
use crate::output;
use crate::output::ROutput;
use crate::track;
use crate::track::{RTrack, Track};

pub struct Band {
    filename: String,
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
    pub fn new(
        filename: Option<&str>,
        talkers: Option<HashMap<Id, RTalker>>,
        _tracks: Option<HashMap<Id, RTrack>>,
        mixers: Option<HashMap<Id, RMixer>>,
        _outputs: Option<HashMap<Id, ROutput>>,
    ) -> Band {
        Self {
            filename: filename.unwrap_or("NewBand.gsr").to_string(),
            talkers: talkers.unwrap_or(HashMap::new()),
            mixers: mixers.unwrap_or(HashMap::new()),
        }
    }

    pub fn new_ref(
        filename: Option<&str>,
        talkers: Option<HashMap<Id, RTalker>>,
        tracks: Option<HashMap<Id, RTrack>>,
        mixers: Option<HashMap<Id, RMixer>>,
        outputs: Option<HashMap<Id, ROutput>>,
    ) -> RBand {
        Rc::new(RefCell::new(Band::new(
            filename, talkers, tracks, mixers, outputs,
        )))
    }

    pub fn filename<'a>(&'a self) -> &'a str {
        &self.filename
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
        lines: &'a Vec<String>,
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

        for line in lines {
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
            match f32::from_str(&dpn) {
                Ok(value) => talker.borrow_mut().set_ear_talk_value_by_tag(&tag, value)?,
                Err(_) => match talkers.get(dpn) {
                    Some(tkr) => talker.borrow_mut().set_ear_talk_voice_by_tag(
                        &tag,
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
            let mut track = rtrack.borrow_mut();

            for (tag, dpn, tkn) in &module.attributs {
                match f32::from_str(&dpn) {
                    Ok(value) => track.set_ear_talk_value_by_tag(&tag, value)?,
                    Err(_) => match talkers.get(dpn) {
                        Some(tkr) => track.set_ear_talk_voice_by_tag(
                            &tag,
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
        // let id = rtrack.borrow().id();
        // self.talkers.insert(id, rtrack.clone());
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
        let rmixer = factory.make_mixer(
            Some(Band::id_from_mref(module.mref)?),
            Some(Band::name_from_mref(module.mref)),
            None,
            None,
        )?;
        {
            let mut mixer = rmixer.borrow_mut();

            for (tag, dpn, tkn) in &module.attributs {
                if tag == &Track::kind() {
                    match trk_decs.get(dpn) {
                        Some(trk) => mixer.add_track(self.make_track(factory, &talkers, trk)?),
                        None => return Err(failure::err_msg(format!("Track {} not found!", dpn))),
                    }
                } else if tag == &output::KIND {
                    match otp_decs.get(dpn) {
                        Some(otp) => mixer.add_output(self.make_output(factory, otp)?),
                        None => return Err(failure::err_msg(format!("Output {} not found!", dpn))),
                    }
                } else {
                    match f32::from_str(&dpn) {
                        Ok(value) => mixer.set_ear_talk_value_by_tag(&tag, value)?,
                        Err(_) => match talkers.get(dpn) {
                            Some(tkr) => mixer.set_ear_talk_voice_by_tag(
                                &tag,
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

    pub fn build(factory: &Factory, description_buffer: &[u8]) -> Result<Band, failure::Error> {
        Identifier::initialize_id_count();
        let mut band = Band::new(None, None, None, None, None);
        let description_reader = BufReader::new(description_buffer);
        let lines = description_reader.lines().map(|l| l.unwrap()).collect();
        let (tkr_decs, trk_decs, mxr_decs, otp_decs) = Band::make_decs(&lines)?;

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
    pub fn make(description_buffer: &[u8]) -> Result<Band, failure::Error> {
        Factory::visit(|factory| Band::build(factory, description_buffer))
    }

    pub fn load_file(filename: &str) -> Result<Band, failure::Error> {
        let description_buffer = fs::read(filename)?;

        let mut band = Factory::visit(|factory| Band::build(factory, &description_buffer))?;

        band.filename = filename.to_string();

        Ok(band)
    }

    pub fn to_ref(self) -> RBand {
        Rc::new(RefCell::new(self))
    }

    fn talk_dep_line<'a>(talk: &Talk, mut file: &'a File) -> Result<&'a File, failure::Error> {
        let tkr = &talk.talker().borrow();

        if tkr.is_hidden() {
            writeln!(file, "> {} {}", talk.tag(), tkr.data_string())?;
        } else {
            let voice_tag = tkr.voice_tag(talk.port())?;

            if voice_tag == "" {
                writeln!(
                    file,
                    "> {} {}",
                    talk.tag(),
                    Band::mref(tkr.id(), &tkr.name())
                )?;
            } else {
                writeln!(
                    file,
                    "> {} {}:{}",
                    talk.tag(),
                    Band::mref(tkr.id(), &tkr.name()),
                    voice_tag
                )?;
            }
        }
        Ok(file)
    }

    pub fn save(&self) -> Result<(), failure::Error> {
        let mut file = File::create(&self.filename)?;

        for rtkr in self.talkers.values() {
            let tkr = rtkr.borrow();
            let (model, feature, ears): (&str, String, &Vec<Ear>) = tkr.backup();

            writeln!(
                file,
                "\n{} {} {}",
                model,
                Band::mref(tkr.id(), &tkr.name()),
                feature
            )?;

            for ear in ears {
                ear.fold_talks(Band::talk_dep_line, &file)?;
            }
        }

        for rmixer in self.mixers.values() {
            let mixer = rmixer.borrow();

            for trk in mixer.tracks() {
                writeln!(
                    file,
                    "\n{} {}",
                    track::KIND,
                    Band::mref(trk.borrow().id(), &trk.borrow().name())
                )?;

                for ear in trk.borrow().ears() {
                    ear.fold_talks(Band::talk_dep_line, &file)?;
                }
            }

            writeln!(
                file,
                "\n{} {}",
                mixer::KIND,
                Band::mref(mixer.id(), &mixer.name())
            )?;

            for ear in mixer.ears() {
                ear.fold_talks(Band::talk_dep_line, &file)?;
            }

            for trk in mixer.tracks() {
                writeln!(
                    file,
                    "> {} {}",
                    track::KIND,
                    Band::mref(trk.borrow().id(), &trk.borrow().name())
                )?;
            }

            for routput in mixer.outputs() {
                let output = routput.borrow();
                let (kind, _, _) = output.backup();
                writeln!(
                    file,
                    "> {} {}",
                    kind,
                    Band::mref(output.id(), &output.name())
                )?;
            }

            for routput in mixer.outputs() {
                let output = routput.borrow();
                let (kind, model, properties) = output.backup();
                writeln!(
                    file,
                    "\n{} {} {}",
                    kind,
                    Band::mref(output.id(), &output.name()),
                    model
                )?;

                for (tag, value) in properties {
                    writeln!(file, "> {} {}", tag, value)?;
                }
            }
        }
        Ok(())
    }

    pub fn save_as(&mut self, filename: &str) -> Result<(), failure::Error> {
        self.filename = filename.to_string();
        self.save()?;
        Ok(())
    }

    pub fn add_mixer(&mut self, rmixer: RMixer) {
        let id = rmixer.borrow().id();
        self.talkers.insert(id, rmixer.clone());

        //        self.talkers.insert(id, rmixer.borrow().talker().clone());
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
        Factory::visit(|factory| self.build_talker(factory, model, oid, oname))
    }

    pub fn sup_talker(&mut self, talker: &RTalker) -> Result<(), failure::Error> {
        Ok(()) // TODO
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
    /*
        pub fn open_outputs(&mut self) -> Result<(), failure::Error> {
            for rmixer in self.mixers.values() {
                rmixer.borrow_mut().open_outputs()?;
            }
            Ok(())
        }
        pub fn close_outputs(&mut self) -> Result<(), failure::Error> {
            for rmixer in self.mixers.values() {
                rmixer.borrow_mut().close_outputs()?;
            }
            Ok(())
        }
    */
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
