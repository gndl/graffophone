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
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::io::Write;
use std::rc::Rc;
use std::str::FromStr;

use gpplugin::ear::{Ear, Talk};
use gpplugin::identifier::Identifier;
use gpplugin::talker::{RTalker, Talker};

use crate::audio_data::Vector;
use crate::factory::Factory;
use crate::mixer;
use crate::mixer::RMixer;
use crate::output;
use crate::output::ROutput;
use crate::playback;
use crate::track;
use crate::track::{RTrack, Track};

pub struct Session {
    filename: String,
    talkers: HashMap<u32, RTalker>,
    mixers: HashMap<u32, RMixer>,
}

pub type RSession = Rc<RefCell<Session>>;

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

impl Session {
    pub fn new(
        filename: Option<&str>,
        talkers: Option<HashMap<u32, RTalker>>,
        _tracks: Option<HashMap<u32, RTrack>>,
        mixers: Option<HashMap<u32, RMixer>>,
        _outputs: Option<HashMap<u32, ROutput>>,
    ) -> Session {
        Self {
            filename: filename.unwrap_or("NewSession.gsr").to_string(),
            talkers: talkers.unwrap_or(HashMap::new()),
            mixers: mixers.unwrap_or(HashMap::new()),
        }
    }

    pub fn new_ref(
        filename: Option<&str>,
        talkers: Option<HashMap<u32, RTalker>>,
        tracks: Option<HashMap<u32, RTrack>>,
        mixers: Option<HashMap<u32, RMixer>>,
        outputs: Option<HashMap<u32, ROutput>>,
    ) -> RSession {
        Rc::new(RefCell::new(Session::new(
            filename, talkers, tracks, mixers, outputs,
        )))
    }

    fn mref(id: u32, name: &str) -> String {
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

    fn id_from_mref(mref: &str) -> Result<u32, failure::Error> {
        let parts: Vec<&str> = mref.split('#').collect();
        match u32::from_str(parts[0]) {
            Ok(id) => Ok(id),
            Err(e) => Err(failure::err_msg(format!(
                "Failed to get id from mref {} : {}!",
                mref,
                e.description()
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
                    Session::tidy_decs(module, &mut decs);
                    module = Module::new(kind, mref, feature);
                }
                (Some(kind), Some(mref), None) => {
                    Session::tidy_decs(module, &mut decs);
                    module = Module::new(kind, mref, "");
                }
                _ => (),
            }
        }
        Session::tidy_decs(module, &mut decs);
        Ok(decs)
    }

    fn set_talker_ears(
        talkers: &HashMap<&str, RTalker>,
        talker: &RTalker,
        module: &Module,
    ) -> Result<(), failure::Error> {
        for (tag, dpn, tkn) in &module.attributs {
            match f32::from_str(&dpn) {
                Ok(value) => talker.borrow_mut().set_ear_value_by_tag(&tag, value)?,
                Err(_) => match talkers.get(dpn) {
                    Some(tkr) => talker.borrow_mut().set_ear_voice_by_tag(
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
        factory: &Factory,
        talkers: &HashMap<&str, RTalker>,
        module: &Module,
    ) -> Result<Track, failure::Error> {
        let mut track = factory.make_track(
            Some(Session::id_from_mref(module.mref)?),
            Some(Session::name_from_mref(module.mref)),
        )?;

        for (tag, dpn, tkn) in &module.attributs {
            match f32::from_str(&dpn) {
                Ok(value) => track.set_ear_value_by_tag(&tag, value)?,
                Err(_) => match talkers.get(dpn) {
                    Some(tkr) => {
                        track.set_ear_voice_by_tag(&tag, tkr, tkr.borrow().voice_port(&tkn)?)?
                    }
                    None => {
                        return Err(failure::err_msg(format!("Talker {} not found!", dpn)));
                    }
                },
            }
        }
        Ok(track)
    }

    fn make_output(factory: &Factory, module: &Module) -> Result<ROutput, failure::Error> {
        factory.make_output(
            Some(Session::id_from_mref(module.mref)?),
            Some(Session::name_from_mref(module.mref)),
            module.feature,
            Some(&module.attributs),
        )
    }

    fn make_mixer(
        factory: &Factory,
        talkers: &HashMap<&str, RTalker>,
        trk_decs: &HashMap<&str, Module>,
        otp_decs: &HashMap<&str, Module>,
        module: &Module,
    ) -> Result<RMixer, failure::Error> {
        let rmixer = factory.make_mixer(
            Some(Session::id_from_mref(module.mref)?),
            Some(Session::name_from_mref(module.mref)),
            None,
            None,
        )?;
        {
            let mut mixer = rmixer.borrow_mut();

            for (tag, dpn, tkn) in &module.attributs {
                if tag == &Track::kind() {
                    match trk_decs.get(dpn) {
                        Some(trk) => mixer.add_track(Session::make_track(factory, &talkers, trk)?),
                        None => return Err(failure::err_msg(format!("Track {} not found!", dpn))),
                    }
                } else if tag == &output::KIND {
                    match otp_decs.get(dpn) {
                        Some(otp) => mixer.add_output(Session::make_output(factory, otp)?),
                        None => return Err(failure::err_msg(format!("Output {} not found!", dpn))),
                    }
                } else {
                    match f32::from_str(&dpn) {
                        Ok(value) => mixer.set_ear_value_by_tag(&tag, value)?,
                        Err(_) => match talkers.get(dpn) {
                            Some(tkr) => mixer.set_ear_voice_by_tag(
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

    pub fn load(factory: &Factory, filename: &str) -> Result<Session, failure::Error> {
        Identifier::initialize_id_count();
        let mut session = Session::new(Some(filename), None, None, None, None);

        let br = BufReader::new(File::open(filename)?);

        let lines = br.lines().map(|l| l.unwrap()).collect();
        let (tkr_decs, trk_decs, mxr_decs, otp_decs) = Session::make_decs(&lines)?;

        let mut talkers = HashMap::new();
        let mut talkers_modules = Vec::new();

        for (mref, module) in tkr_decs {
            let tkr = session.add_talker(
                factory,
                module.kind,
                Some(Session::id_from_mref(mref)?),
                Some(Session::name_from_mref(mref)),
            )?;

            if module.feature.len() > 0 {
                tkr.borrow_mut().set_data_from_string(module.feature)?;
            }
            talkers_modules.push((tkr.clone(), module));
            talkers.insert(mref, tkr.clone());
        }

        for (talker, module) in talkers_modules {
            Session::set_talker_ears(&talkers, &talker, &module)?;
        }

        for (_, module) in mxr_decs {
            let rmixer = Session::make_mixer(factory, &talkers, &trk_decs, &otp_decs, &module)?;
            session.add_mixer(rmixer);
        }

        Ok(session)
    }

    pub fn load_ref(factory: &Factory, filename: &str) -> Result<RSession, failure::Error> {
        Ok(Rc::new(RefCell::new(Session::load(factory, filename)?)))
    }

    fn talk_dep_line<'a>(mut file: &'a File, talk: &Talk) -> Result<&'a File, failure::Error> {
        let tkr = &talk.talker().borrow();

        if tkr.is_hidden() {
            writeln!(file, "> {} {}", talk.tag(), tkr.get_data_string())?;
        } else {
            let voice_tag = tkr.voice_tag(talk.port())?;

            if voice_tag == "" {
                writeln!(
                    file,
                    "> {} {}",
                    talk.tag(),
                    Session::mref(tkr.id(), &tkr.name())
                )?;
            } else {
                writeln!(
                    file,
                    "> {} {}:{}",
                    talk.tag(),
                    Session::mref(tkr.id(), &tkr.name()),
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
                Session::mref(tkr.id(), &tkr.name()),
                feature
            )?;

            for ear in ears {
                ear.fold_talks(Session::talk_dep_line, &file)?;
            }
        }

        for rmixer in self.mixers.values() {
            let mut mixer = rmixer.borrow_mut();

            for trk in mixer.tracks() {
                writeln!(
                    file,
                    "\n{} {}",
                    track::KIND,
                    Session::mref(trk.id(), &trk.name())
                )?;

                for ear in trk.ears() {
                    ear.fold_talks(Session::talk_dep_line, &file)?;
                }
            }

            writeln!(
                file,
                "\n{} {}",
                mixer::KIND,
                Session::mref(mixer.id(), &mixer.name())
            )?;

            for ear in mixer.ears() {
                ear.fold_talks(Session::talk_dep_line, &file)?;
            }

            for trk in mixer.tracks() {
                writeln!(
                    file,
                    "> {} {}",
                    track::KIND,
                    Session::mref(trk.id(), &trk.name())
                )?;
            }

            for routput in mixer.outputs() {
                let output = routput.borrow();
                let (kind, _, _) = output.backup();
                writeln!(
                    file,
                    "> {} {}",
                    kind,
                    Session::mref(output.id(), &output.name())
                )?;
            }

            for routput in mixer.outputs() {
                let output = routput.borrow();
                let (kind, model, properties) = output.backup();
                writeln!(
                    file,
                    "\n{} {} {}",
                    kind,
                    Session::mref(output.id(), &output.name()),
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
        let id = rmixer.borrow_mut().id();
        self.mixers.insert(id, rmixer);
    }

    pub fn add_talker(
        &mut self,
        factory: &Factory,
        model: &str,
        oid: Option<u32>,
        oname: Option<&str>,
    ) -> Result<RTalker, failure::Error> {
        let tkr = factory.make_talker(model, oid, oname)?;
        self.talkers.insert(tkr.borrow().id(), tkr.clone());
        Ok(tkr)
    }

    pub fn add_playback(&mut self, factory: &Factory) -> Result<(), failure::Error> {
        for rmixer in self.mixers.values() {
            rmixer.borrow_mut().add_output(factory.make_output(
                None,
                None,
                playback::MODEL,
                None,
            )?);
        }
        Ok(())
    }

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
    pub fn play_chunk(
        &mut self,
        tick: i64,
        buf: &mut Vector,
        channels: &mut Vec<Vector>,
        len: usize,
    ) -> Result<usize, failure::Error> {
        let mut ln = len;

        for rmixer in self.mixers.values() {
            ln = rmixer.borrow_mut().come_out(tick, buf, channels, ln)?;
        }
        Ok(ln)
    }
}
