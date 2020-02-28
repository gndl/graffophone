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
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::rc::Rc;
use std::str::FromStr;

use gpplugin::talker::{RTalker, Talker};

use crate::factory::Factory;
use crate::mixer;
use crate::mixer::{Mixer, RMixer};
use crate::output;
use crate::output::ROutput;
use crate::track;
use crate::track::{RTrack, Track};

pub struct Session {
    filename: String,
    talkers: HashMap<u32, RTalker>,
    mixers: HashMap<u32, Mixer>,
    factory: Factory,
}

pub type RSession = Rc<RefCell<Session>>;

struct Properties<'a> {
    kind: &'a str,
    id: &'a str,
    feature: &'a str,
    attributs: Vec<(&'a str, &'a str, &'a str)>,
}
impl<'a> Properties<'a> {
    pub fn new(kind: &'a str, id: &'a str, feature: &'a str) -> Properties<'a> {
        Self {
            kind,
            id,
            feature,
            attributs: Vec::new(),
        }
    }
}

impl Session {
    pub fn new(
        filename: Option<&str>,
        talkers: Option<HashMap<u32, RTalker>>,
        tracks: Option<HashMap<u32, RTrack>>,
        mixers: Option<HashMap<u32, Mixer>>,
        outputs: Option<HashMap<u32, ROutput>>,
        factory: Option<Factory>,
    ) -> Session {
        Self {
            filename: filename.unwrap_or("NewSession.gsr").to_string(),
            talkers: talkers.unwrap_or(HashMap::new()),
            mixers: mixers.unwrap_or(HashMap::new()),
            factory: factory.unwrap_or(Factory::new()),
        }
    }

    pub fn new_ref(
        filename: Option<&str>,
        talkers: Option<HashMap<u32, RTalker>>,
        tracks: Option<HashMap<u32, RTrack>>,
        mixers: Option<HashMap<u32, Mixer>>,
        outputs: Option<HashMap<u32, ROutput>>,
        factory: Option<Factory>,
    ) -> RSession {
        Rc::new(RefCell::new(Session::new(
            filename, talkers, tracks, mixers, outputs, factory,
        )))
    }

    pub fn add_talker(&mut self, id: &str, name: Option<&str>) -> Result<RTalker, failure::Error> {
        let tkr = self.factory.make_talker(id, name)?;
        self.talkers.insert(tkr.borrow().id(), tkr.clone());
        Ok(tkr)
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

    fn mk_id(tkr: &RTalker) -> String {
        format!("{}#{}", tkr.borrow().id(), tkr.borrow().name())
    }

    fn name_from_id(id: &str) -> &str {
        let parts: Vec<&str> = id.split('#').collect();

        if parts.len() == 2 {
            parts[1]
        } else {
            id
        }
    }

    fn format_id(id: &str) -> String {
        id.replace(" ", "_").replace("\t", "_")
    }

    fn tidy_decs<'a>(
        properties: Properties<'a>,
        (tkr_decs, trk_decs, mxr_decs, otp_decs): &mut (
            HashMap<&'a str, Properties<'a>>,
            HashMap<&'a str, Properties<'a>>,
            HashMap<&'a str, Properties<'a>>,
            HashMap<&'a str, Properties<'a>>,
        ),
    ) {
        match properties.kind {
            "" => None,
            track::KIND => trk_decs.insert(properties.id, properties),
            mixer::KIND => mxr_decs.insert(properties.id, properties),
            output::KIND => otp_decs.insert(properties.id, properties),
            _ => tkr_decs.insert(properties.id, properties),
        };
    }

    fn make_decs<'a>(
        lines: &'a Vec<String>,
    ) -> Result<
        (
            HashMap<&'a str, Properties<'a>>,
            HashMap<&'a str, Properties<'a>>,
            HashMap<&'a str, Properties<'a>>,
            HashMap<&'a str, Properties<'a>>,
        ),
        failure::Error,
    > {
        let mut decs = (
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
        );
        let mut properties = Properties::new("", "", "");

        for line in lines {
            let words: Vec<&str> = line.trim().split(|c| c == ' ' || c == '\t').collect();

            match (words.get(0), words.get(1), words.get(2)) {
                (None, None, None) => (),
                (Some(c), _, _) if c.chars().next() == Some('/') => (),
                (Some(p), Some(tag), Some(tlk)) if p == &">" => {
                    let tlk_p: Vec<&str> = tlk.split(':').collect();
                    let tkr = tlk_p.get(0).unwrap_or(tlk);
                    let sp = tlk_p.get(1).unwrap_or(&"");

                    if properties.kind == "" {
                        return Err(failure::err_msg(format!(
                            "Found properties attribut {} {} before properties!",
                            tag, tlk
                        )));
                    }
                    properties.attributs.push((tag, tkr, sp));
                }
                (Some(kind), Some(id), Some(feature)) => {
                    Session::tidy_decs(properties, &mut decs);
                    properties = Properties::new(kind, id, feature);
                }
                (Some(kind), Some(id), None) => {
                    Session::tidy_decs(properties, &mut decs);
                    properties = Properties::new(kind, id, "");
                }
                _ => (),
            }
        }
        Session::tidy_decs(properties, &mut decs);
        Ok(decs)
    }

    fn set_talker_ears(
        talkers: &HashMap<&str, RTalker>,
        talker: &RTalker,
        properties: &Properties,
    ) -> Result<(), failure::Error> {
        for (tag, dpn, tkn) in &properties.attributs {
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
        talkers: &HashMap<&str, RTalker>,
        properties: &Properties,
    ) -> Result<Track, failure::Error> {
        let mut track = Track::new();
        track.set_name(Session::name_from_id(properties.id));

        for (tag, dpn, tkn) in &properties.attributs {
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

    fn make_output(factory: &Factory, properties: &Properties) -> Result<ROutput, failure::Error> {
        factory.make_output(
            Session::name_from_id(properties.id),
            properties.feature,
            &properties.attributs,
        )
    }

    fn make_mixer(
        factory: &Factory,
        talkers: &HashMap<&str, RTalker>,
        trk_decs: &HashMap<&str, Properties>,
        otp_decs: &HashMap<&str, Properties>,
        properties: &Properties,
    ) -> Result<Mixer, failure::Error> {
        let mut mixer = Mixer::new(None, None);
        mixer.set_name(Session::name_from_id(properties.id));

        for (tag, dpn, tkn) in &properties.attributs {
            if tag == &Track::kind() {
                match trk_decs.get(dpn) {
                    Some(trk) => mixer.add_track(Session::make_track(&talkers, trk)?),
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
                        Some(tkr) => {
                            mixer.set_ear_voice_by_tag(&tag, tkr, tkr.borrow().voice_port(&tkn)?)?
                        }
                        None => {
                            return Err(failure::err_msg(format!("Talker {} not found!", dpn)));
                        }
                    },
                }
            }
        }
        Ok(mixer)
    }

    fn load(filename: &str) -> Result<Session, failure::Error> {
        let mut session = Session::new(Some(filename), None, None, None, None, None);

        let br = BufReader::new(File::open(filename)?);

        let lines = br.lines().map(|l| l.unwrap()).collect();
        let (tkr_decs, trk_decs, mxr_decs, otp_decs) = Session::make_decs(&lines)?;

        let mut talkers = HashMap::new();
        let mut talkers_props = Vec::new();

        for (id, prop) in tkr_decs {
            let tkr = session.add_talker(prop.kind, Some(Session::name_from_id(id)))?;

            if prop.feature.len() > 0 {
                tkr.borrow_mut().set_data_from_string(prop.feature)?;
            }
            talkers_props.push((tkr.clone(), prop));
            talkers.insert(id, tkr.clone());
        }

        for (talker, properties) in talkers_props {
            Session::set_talker_ears(&talkers, &talker, &properties)?;
        }

        for (id, properties) in mxr_decs {
            let mixer = Session::make_mixer(
                &session.factory,
                &talkers,
                &trk_decs,
                &otp_decs,
                &properties,
            )?;
            session.mixers.insert(mixer.id(), mixer);
        }

        Ok(session)
    }
    /*
        let headLine id knd ftr = [""; knd ^ " " ^ formatId id ^ " " ^ ftr] in
    fn depLine tag dep = "> " ^ tag ^ " " ^ formatId dep
      in
      let wordDepLine wrd =  Ear.(depLine wrd.wTag (sof wrd.value))
      in
      let talkDepLine tlk = Ear.(
          let tkr = Ear.getTalkTalker tlk in

          if tkr#isHidden then depLine tlk.tTag tkr#getStringOfValue
          else (
            let l = depLine tlk.tTag (mkId tkr)
            in
            if tlk.voice.Voice.vTag = "" then l
            else l ^ ":" ^ tlk.voice.Voice.vTag
          )
        )
      in
      let srcToL src =
        match src with
        | Ear.Word wrd -> wordDepLine wrd
        | Ear.Talk tlk -> talkDepLine tlk
      in
    fn dec2lines( id: (knd, ftr, ears) =
        (headLine id knd ftr) @ L.map ~f:srcToL (A.to_list(Ear.earsToSources ears))
      in
    fn a2l (tag, id) = depLine tag (mkId id)
      in

        pub fn save(&self)-> Result<(), failure::Error>{//std::io::Result<()>

      let mcDecToLines id (knd, ftr, ears, trks, ops) =
        (headLine id knd ftr)
        @ L.map ~f:srcToL (A.to_list(Ear.earsToSources ears)) @ L.map ~f:aToL trks @ L.map ~f:aToL ops
      in
      let opDecToLines id (knd, ftr, al) =
        (headLine id knd ftr) @ L.map ~f:(fun (tag, dep) -> depLine tag dep) al
      in
      // let sn = recoverDefaultTalkers session
      let sn = session
      in
      let lines = L.flatten (
          L.map ~f:(fun (_, e) -> decToLines (mkId e) e#backup) sn.talkers
          @ L.map ~f:(fun (_, e) -> decToLines (mkId e) e#backup) sn.tracks
          @ L.map ~f:(fun (_, e) -> mcDecToLines (mkId e) e#mixingConsoleBackup) sn.mixCons
          @ L.map ~f:(fun (_, e) -> opDecToLines (mkId e) e#backup) sn.outputs)
      in
      writeFileLines session.filename lines
            let mut file = File::create(self.filename)?;

            for tkr in self.talkers.values() {
                let lines = dec2lines(mk_id(tkr), tkr.backup());
        file.write_all(lines)?;
            }

        Ok(())
    }

    pub fn saveAs(&self, filename:String){
      let ns = make ~filename:filename ~talkers:session.talkers
          ~tracks:session.tracks ~mixingConsoles:session.mixCons
          ~outputs:session.outputs ()
      in
      save ns;
      ns
    }
                  */
}

/*
let talkers session = session.talkers
let tracks session = session.tracks
let mixCons session = session.mixCons
let outputs session = session.outputs

let gInstance = ref {
    filename = ""; talkers = []; tracks = []; mixCons = []; outputs = []
  }

let getInstance() = !gInstance

let getTalkers() = !gInstance.talkers
let getTracks() = !gInstance.tracks
let getMixingConsoles() = !gInstance.mixCons
let getOutputs() = !gInstance.outputs

let findTalker id = L.assoc id !gInstance.talkers
let findTrack id = L.assoc id !gInstance.tracks
let findMixingConsole id = L.assoc id !gInstance.mixCons
let findOutput id = L.assoc id !gInstance.outputs

let addTalker tkr = !gInstance.talkers <- (tkr#getId, tkr)::!gInstance.talkers
let addTrack tkr = !gInstance.tracks <- (tkr#getId, tkr)::!gInstance.tracks
let addMixingConsole tkr = !gInstance.mixCons <- (tkr#getId, tkr)::!gInstance.mixCons
let addOutput op = !gInstance.outputs <- (op#getId, op)::!gInstance.outputs

let supTalker tkr =
  !gInstance.talkers <- L.filter ~f:(fun (_, t) -> t <> tkr) !gInstance.talkers

let supTrack tkr =
  !gInstance.tracks <- L.filter ~f:(fun (_, t) -> t <> tkr) !gInstance.tracks

let supMixingConsole tkr =
  !gInstance.mixCons <- L.filter ~f:(fun (_, t) -> t <> tkr) !gInstance.mixCons

let supOutput op =
  !gInstance.outputs <- L.filter ~f:(fun (_, t) -> t <> op) !gInstance.outputs


let make ?(filename = "NewSession.es") ?(talkers = [])
    ?(tracks = []) ?(mixingConsoles = []) ?(outputs = []) () =

  gInstance := {
    filename = filename;
    talkers = L.map talkers ~f:(fun(_, tkr) -> (tkr#getId, tkr));
    tracks = L.map tracks ~f:(fun(_, tkr) -> (tkr#getId, tkr));
    mixCons = L.map mixingConsoles ~f:(fun(_, tkr) -> (tkr#getId, tkr));
    outputs = L.map outputs ~f:(fun(_, op) -> (op#getId, op))
  };
  !gInstance


// recover constant talker created by Talker in order to set a word on a talk
let recoverDefaultTalkers session =
  let talkers = ref [] in
  let rec recDefTkr deps =
    L.iter deps
      ~f:(fun talk ->
          let tkr = Ear.getTalkTalker talk in

          recDefTkr tkr#getTalks;

          if not(L.mem_assoc tkr#getId ~map:session.talkers) then
            talkers := (tkr#getId, tkr) :: !talkers)
  in
  L.iter ~f:(fun (_, tkr) -> recDefTkr (tkr#getTalks)) session.talkers;
  L.iter ~f:(fun (_, tkr) -> recDefTkr (tkr#getTalks)) session.tracks;
  L.iter ~f:(fun (_, tkr) -> recDefTkr (tkr#getTalks)) session.mixCons;

  make ~filename:session.filename ~talkers:(session.talkers @ !talkers)
    ~tracks:session.tracks ~mixingConsoles:session.mixCons
    ~outputs:session.outputs ()


*/

/*
let mcOfDec (name, prop) =
let rec tomOfAtts ts os mv = function
| [] -> (ts, os, mv)
| a::tl -> (
if a.tag = Track.kind then tomOfAtts ((assoc a.name tracks)::ts) os mv tl
else
if a.tag = "out" then tomOfAtts ts ((assoc a.name outputs)::os) mv tl
else
if a.tag = "volume" then
tomOfAtts ts os (Some(assoc a.name !talkers)) tl
else tomOfAtts ts os mv tl
)
in
let (ts, os, mv) = tomOfAtts [] [] None prop.attributs in
let tracks = L.rev ts and outputs = L.rev os in
let mc = match mv with
| Some v -> new cMixingConsole ~tracks ~outputs ~volume:v ~name ()
| None -> new cMixingConsole ~tracks ~outputs ~name ()
in (name, mc)
in
*/
