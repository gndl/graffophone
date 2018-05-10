(* 
 * Copyright (C) 2015 Gaëtan Dubreil
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
 *)

open Graffophone_plugin
open Util
open Usual
open SampleFormat

module Tkr = Talker

type t = {
  filename : string;
  mutable talkers : (int * Tkr.c) list;
  mutable tracks : (int * Track.c) list;
  mutable mixCons : (int * MixingConsole.c) list;
  mutable outputs : (int * Output.c) list
}

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
  !gInstance.talkers <- L.filter (fun (_, t) -> t <> tkr) !gInstance.talkers

let supTrack tkr =
  !gInstance.tracks <- L.filter (fun (_, t) -> t <> tkr) !gInstance.tracks

let supMixingConsole tkr =
  !gInstance.mixCons <- L.filter (fun (_, t) -> t <> tkr) !gInstance.mixCons

let supOutput op =
  !gInstance.outputs <- L.filter (fun (_, t) -> t <> op) !gInstance.outputs


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


(* recover constant talker created by Talker in order to set a word on a talk *)
let recoverDefaultTalkers session =
  let talkers = ref [] in
  let rec recDefTkr deps =
    L.iter deps
      ~f:(fun talk ->
          let tkr = Ear.getTalkTalker talk in

          recDefTkr tkr#getTalks;

          if not(L.mem_assoc tkr#getId session.talkers) then
            talkers := (tkr#getId, tkr) :: !talkers)
  in
  L.iter (fun (n, tkr) -> recDefTkr (tkr#getTalks)) session.talkers;
  L.iter (fun (n, tkr) -> recDefTkr (tkr#getTalks)) session.tracks;
  L.iter (fun (n, tkr) -> recDefTkr (tkr#getTalks)) session.mixCons;

  make ~filename:session.filename ~talkers:(session.talkers @ !talkers)
    ~tracks:session.tracks ~mixingConsoles:session.mixCons
    ~outputs:session.outputs ()


type attribut_t = {tag : string; dpn : string; tkn : string}
type propertys_t = {kind : string; feature : string; attributs : attribut_t list}

let mkId tkr = (soi tkr#getId ^ "#" ^ tkr#getName)

let getNameFromId id =
  try String.(
      let pos = 1 + index id '#' in
      let idLen = length id in
      if pos = idLen then ""
      else sub id pos (idLen - pos) )
  with Not_found -> id

let formatId id = Str.global_replace (Str.regexp "[ \t]+") "_" id


let makeDecs lines =
  let reg = Str.regexp "[ \t]+" in
  let lns = L.map (fun s -> Str.split reg s) lines
  in
  let splitTlk tlk =
    try let p = String.index tlk ':' in
      (Str.string_before tlk p, Str.string_after tlk (p + 1))
    with Not_found -> (tlk, "")
  in
  let rec mkDs k n f dl al = function
    | [] -> (n, {kind = k; feature = f; attributs = al})::dl
    | l::tl -> (
        match l with
        | c::e when c.[0] = '/' -> mkDs k n f dl al tl (* commentaires *)
        | p::t::tlk::e when p = ">" -> let (tkr, sp) = splitTlk tlk in
          mkDs k n f dl ({tag = t; dpn = tkr; tkn = sp}::al) tl
        | nk::nn::nf::e ->
          mkDs nk nn nf ((n, {kind = k; feature = f; attributs = al})::dl) [] tl
        | nk::nn::e ->
          mkDs nk nn "" ((n, {kind = k; feature = f; attributs = al})::dl) [] tl
        | _ -> mkDs k n f dl al tl
      )
  in
  L.tl (L.rev (mkDs "" "" "" [] [] lns))


let load filename =
  let decs = makeDecs (readFileLines filename) in

  let (trkDecs, l1) = L.partition (fun (n, p) -> p.kind = Track.kind) decs in
  let (outpDecs, l2) = L.partition (fun (n, p) -> p.kind = Output.kind) l1 in
  let (mxcDecs, tkrDecs) = L.partition (fun (n, p) -> p.kind = MixingConsole.kind) l2 in

  let tkrOfDec (id, prop) =

    let tkr = Factory.makeTalker prop.kind ~name:(getNameFromId id)
    in
    trace("talkers := ("^id^", tkr) :: !talkers;");
    if S.length prop.feature > 0 then (tkr#setValueOfString prop.feature;);
    ((id, tkr), (tkr, prop))
  in
  let (talkers, talkersProps) = L.split(L.map tkrDecs ~f:tkrOfDec) in

  let setTalkerEars (talker, properties) =

    L.iter properties.attributs
      ~f:(fun att -> try
             try
               let value = fos att.dpn in
               talker#setEarToValueByTag att.tag value;
             with Failure s -> (
                 try
                   let tkr = L.assoc att.dpn talkers in

                   talker#setEarToVoiceByTag att.tag (tkr#getVoice att.tkn);

                 with Not_found -> trace("talker |"^att.dpn^"| not found")
               )
           with Tkr.TagNotFound msg -> (
               trace(msg^"\nDependence "^att.dpn^" not found!");
               (*raise Tkr.TagNotFound*)
             )
              | x -> traceMagenta(Printexc.to_string x)
         )
  in

  trace "> L.iter tkrDecs ~f:setTalkerEars;";
  L.iter talkersProps ~f:setTalkerEars;
  trace "< L.iter tkrDecs ~f:setTalkerEars;";

  let tracks = L.map trkDecs
      ~f:(fun (id, properties) ->
          let trk = Track.make() in
          trk#setName (getNameFromId id);
          setTalkerEars (trk, properties);
          (id, trk)
        )
  in
  trace "> outputs = L.map outpDecs";
  let outputs = L.map outpDecs
      ~f:(fun (id, p) -> (id, Factory.makeOutput (getNameFromId id) p.feature (
          L.map (fun a -> (a.tag, a.dpn, a.tkn)) p.attributs)))
  in
  trace "< outputs = L.map outpDecs";
  let mixingConsoles = L.map mxcDecs
      ~f:(fun (id, properties) ->
          let mixCon = MixingConsole.make (getNameFromId id)
              (L.map (fun a -> (a.tag, a.dpn, a.tkn)) properties.attributs)
              tracks outputs
          in
          setTalkerEars (mixCon, properties);
          (id, mixCon)
        )
  in
  (*recoverDefaultTalkers () *)
  make ~filename ~talkers ~tracks ~mixingConsoles ~outputs ()


let save session =
  let headLine id knd ftr = [""; knd ^ " " ^ formatId id ^ " " ^ ftr] in
  let depLine tag dep = "> " ^ tag ^ " " ^ formatId dep
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
  let decToLines id (knd, ftr, ears) =
    (headLine id knd ftr) @ L.map srcToL (A.to_list(Ear.earsToSources ears))
  in
  let aToL (tag, id) = depLine tag (mkId id)
  in
  let mcDecToLines id (knd, ftr, ears, trks, ops) =
    (headLine id knd ftr)
    @ L.map srcToL (A.to_list(Ear.earsToSources ears)) @ L.map aToL trks @ L.map aToL ops
  in
  let opDecToLines id (knd, ftr, al) =
    (headLine id knd ftr) @ L.map (fun (tag, dep) -> depLine tag dep) al
  in
  (*let sn = recoverDefaultTalkers session*)
  let sn = session
  in
  let lines = L.flatten (
      L.map (fun (n, e) -> decToLines (mkId e) e#backup) sn.talkers
      @ L.map (fun (n, e) -> decToLines (mkId e) e#backup) sn.tracks
      @ L.map (fun (n, e) -> mcDecToLines (mkId e) e#mixingConsoleBackup) sn.mixCons
      @ L.map (fun (n, e) -> opDecToLines (mkId e) e#backup) sn.outputs)
  in
  writeFileLines session.filename lines


let saveAs filename session =
  let ns = make ~filename:filename ~talkers:session.talkers
      ~tracks:session.tracks ~mixingConsoles:session.mixCons
      ~outputs:session.outputs ()
  in
  save ns;
  ns



(*
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
*)
