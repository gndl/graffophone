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

open Util

open Voice
open Ear

exception UnexpectedValue of string
exception UnexpectedAttribut of string
exception UnexpectedFormat of string
exception UndefinedTalker of string
exception UndefinedVoice of string
exception IncompatibleIO of string
exception TagNotFound of string

type valueType_t = Nil
                 | Int of int
                 | Float of float
                 | String of string
                 | Text of string
                 | File of string

let typeOfValue = function
  | Int _ -> "Int"
  | Float _ -> "Float"
  | String _ -> "String"
  | Text _ -> "Text"
  | File _ -> "File"
  | Nil -> "Nil"

let notifyIncompatibility v expectedType =
  raise(UnexpectedValue("Type "^typeOfValue v^" is incompatible with the expected "^expectedType^" type"))

let v2i  v = match v with Int i -> i    | _ -> notifyIncompatibility v "Int"
let v2f  v = match v with Float f -> f  | _ -> notifyIncompatibility v "Float"
let v2s  v = match v with String s -> s | _ -> notifyIncompatibility v "String"
let v2t  v = match v with Text t -> t   | _ -> notifyIncompatibility v "Text"
let v2fl v = match v with File fn -> fn | _ -> notifyIncompatibility v "File"

let i2v  i  = Int i
let f2v  f  = Float f
let s2v  s  = String s
let t2v  t  = Text t
let fl2v fn = File fn

let stringOfValue = function
  | Int i -> string_of_int i
  | Float f -> string_of_float f
  | String s | Text s | File s -> s
  | Nil -> ""

let valueOfString s = function
  | Int _ -> Int (int_of_string s)
  | Float _ -> Float (float_of_string s)
  | String _ -> String s
  | Text _ -> Text s
  | File _ -> File s
  | Nil -> Nil


let sTalkerCount = ref 0;

class virtual c =

  let setTalkValue talk value =
    if talk.voice.tkr#isHidden then
      talk.voice.tkr#setValue(Float value)
    else
      talk.voice <- ((new hiddenConstant ~value ())#getVoice "")
  in

  object(self)
    inherit Identifier.c sTalkerCount as identifier
    val mutable mGetEarCall = false
    val mutable mFlatEars = [||]

    initializer
      A.iter ~f:(fun vc -> vc.tkr <- self#base) self#getVoices


    method base = (self :> c)

    method virtual getKind : string
    method isHidden = false

    (*method isConstant = false*)

    method! dependsOf id =
      L.fold_left ~init:(identifier#dependsOf id)
        ~f:(fun d talk -> d || talk.voice.tkr#dependsOf id) self#getTalks

    method getValue = Nil
    method setValue (v : valueType_t) = notifyIncompatibility v "Nil"

    method getStringOfValue = stringOfValue self#getValue
    method setValueOfString s = self#setValue(valueOfString s self#getValue)

    method getFloatValue = v2f self#getValue


    method setEar (tag:string) (_:c Ear.t) = (
      raise(UnexpectedAttribut("Unexpected attribut "^tag^" for "^self#getName)) : unit)

    (*method setEarToWord (tag : string) (word : float) =*)

    method setEarToValueByTag (tag : string) (value : float) =

      if not(A.fold_left self#getEars ~init:false
               ~f:(fun found ear -> if found then true
                    else (
                      match ear with
                      | EWord wrd when wrd.wTag = tag -> wrd.value <- value; true
                      | ETalk tlk when tlk.tTag = tag -> setTalkValue tlk value; true
                      | EBin bin when tag = binTag bin ->
                        bin.src <- Word {value; wTag = tag}; true
                      | EWords wrds ->
                        if wrds.wsTag = tag then (
                          wrds.words <- A.add wrds.words {value; wTag = tag};
                          true
                        )
                        else (
                          A.fold_left wrds.words ~init:false
                            ~f:(fun found wrd ->
                                if found || wrd.wTag <> tag then found
                                else (wrd.value <- value; true))
                        )
                      | ETalks tlks ->
                        if tlks.tsTag = tag then (
                          let voice = ((new hiddenConstant ~value ())#getVoice "") in
                          tlks.talks <- A.add tlks.talks {voice; tTag = tag};
                          true
                        )
                        else (
                          A.fold_left tlks.talks ~init:false
                            ~f:(fun found tlk ->
                                if found || tlk.tTag <> tag then found
                                else (setTalkValue tlk value; true)
                              )
                        )
                      | EBins bns ->
                        if bns.bsTag = tag then (
                          bns.bins <- A.add bns.bins {src = Word{value; wTag = tag}};
                          true
                        )
                        else (
                          A.fold_left bns.bins ~init:false
                            ~f:(fun found bin ->
                                if found || tag <> binTag bin then found
                                else (bin.src <- Word{value; wTag = tag}; true))
                        )
                      | _ -> false
                    )
                  )
            ) then (raise(TagNotFound("Tag "^tag^" not found for "^self#getName^" ears talks")) : unit)


    method setEarToVoiceByTag (tag : string) (voice : c Voice.t) =

      if not(A.fold_left self#getEars ~init:false
               ~f:(fun found ear -> if found then true
                    else (
                      match ear with
                      | ETalk tlk when tlk.tTag = tag -> tlk.voice <- voice; true
                      | EBin bin when tag = binTag bin ->
                        bin.src <- Talk {voice; tTag = tag}; true
                      | ETalks tlks ->
                        if tlks.tsTag = tag then (
                          tlks.talks <- A.add tlks.talks {voice; tTag = tag};
                          true
                        )
                        else (
                          A.fold_left tlks.talks ~init:false
                            ~f:(fun found tlk ->
                                if found || tlk.tTag <> tag then found
                                else (tlk.voice <- voice; true))
                        )
                      | EBins bns ->
                        if bns.bsTag = tag then (
                          bns.bins <- A.add bns.bins {src = Talk{voice; tTag = tag}};
                          true
                        )
                        else (
                          A.fold_left bns.bins ~init:false
                            ~f:(fun found bin ->
                                if found || tag <> binTag bin then found
                                else (bin.src <- Talk {voice; tTag = tag}; true))
                        )
                      | _ -> false
                    )
                  )
            ) then (
        raise(TagNotFound("Tag "^tag^" not found for "^self#getName^" ears talks")) : unit)


    method setEarToValueByIndex (index:int) (value : float) =

      match self#getFlatEars.(index) with
      | EWord word -> word.value <- value
      | ETalk talk -> setTalkValue talk value
      | EBin bin -> let wTag = binTag bin in bin.src <- Word {value; wTag}
      | _ -> raise(IncompatibleIO("Incompatible IO of ear "^string_of_int index))


    method setEarToVoiceByIndex (index:int) (voice : c Voice.t) =

      match self#getFlatEars.(index) with
      | ETalk talk -> talk.voice <- voice
      | EBin bin -> bin.src <- Talk {voice; tTag = binTag bin}
      | _ -> raise(IncompatibleIO("Incompatible IO of ear "^string_of_int index))


    (* ear array *)
    method getEars = (
      mGetEarCall <- true;
      let ears = L.map ~f:(fun f -> ETalk f) self#getTalks in
      mGetEarCall <- false;
      A.of_list ears
      : c Ear.t array)

    method getFlatEars = Ear.flattenEars self#getEars
    method getEarsSources = Ear.earsToSources self#getEars


    method getTalks = (
      if mGetEarCall then []
      else talksOfEars self#getEars : c Ear.talk_a list)


    method addEarToValueByIndex index value =

      match self#getEars.(index) with
      | EWords wrds ->
        wrds.words <- A.add wrds.words {value; wTag = wrds.wsTag}
      | ETalks tlks ->
        let voice = ((new hiddenConstant ~value ())#getVoice "") in
        tlks.talks <- A.add tlks.talks {voice; tTag = tlks.tsTag}
      | EBins bns ->
        bns.bins <- A.add bns.bins {src = Word{value; wTag = bns.bsTag}}
      | _ -> ()


    method addEarToVoiceByIndex index voice =

      match self#getEars.(index) with
      | ETalks tlks ->
        tlks.talks <- A.add tlks.talks {voice; tTag = tlks.tsTag}
      | EBins bns ->
        bns.bins <- A.add bns.bins {src = Talk{voice; tTag = bns.bsTag}}
      | _ -> ()


    method addBin tag src =

      let add added ear =
        if added then true
        else (
          match ear with
          | EBins bns when bns.bsTag = tag ->
            bns.bins <- A.add bns.bins {src};
            true
          | _ -> false
        )
      in

      if not(A.fold_left self#getEars ~init:false ~f:add) then (
        raise(UnexpectedAttribut("Unexpected bins "^tag^" for "^self#getName)) : unit)


    method supEarByIndex index =

      ignore(A.fold_left self#getEars ~init:0
               ~f:(fun start ear -> if index < start then start else
                      match ear with
                      | EWords wrds -> let arrayEnd = start + A.length wrds.words in
                        if index < arrayEnd then
                          wrds.words <- A.sup wrds.words (index - start);
                        arrayEnd
                      | ETalks tlks -> let arrayEnd = start + A.length tlks.talks in
                        if index < arrayEnd then
                          tlks.talks <- A.sup tlks.talks (index - start);
                        arrayEnd
                      | EBins bns -> let arrayEnd = start + A.length bns.bins in
                        if index < arrayEnd then
                          bns.bins <- A.sup bns.bins (index - start);
                        arrayEnd
                      | _ -> start + 1
                  ));


    method getVoice (tag:string) = (

      let voices = self#getVoices in

      if tag = "" && A.length voices > 0 then voices.(0)
      else (
        let optVoice = A.fold_left voices ~init:None
            ~f:(fun r vc ->
                if r = None && vc.vTag = tag then Some vc
                else r)
        in
        match optVoice with Some voice -> voice
                          | None -> raise(UndefinedVoice(self#getName^" "^tag^" voice undefined"))
      )
        : c Voice.t)


    method getVoices = ([||] : c Voice.t array)

    (*method virtual voice : port_t -> int -> int -> unit*)
    method talk (_:port_t) (_:int) (_:int) = ()

    (*                       kind     value                ears *)
    method backup = (self#getKind, self#getStringOfValue, self#getEars)

  end

and hiddenConstant ?(value = 1.) () =
  object(self) inherit c
    val mutable mOutput = None

    method provideOutput =
      match mOutput with
      | Some o -> o
      | None ->
        let o = {vTag = defOutputTag; port = 0; tick = 0; len = 1;
                 tkr = self#base; cor = Cornet.init ~v:value 1}
        in
        mOutput <- Some o;
        o


    method! getValue = Float (Voice.get self#provideOutput 0)


    method! setValue v =
      let output = self#provideOutput in
      fill output 0 (dim output) (v2f v)

    method getKind = "hiddenConstant"
    method! isHidden = true
    method! getVoices = [|self#provideOutput|]

    method! talk _ tick len =
      let output = self#provideOutput in

      if output.len < len then (
        output.cor <- Cornet.init ~v:(Cornet.get output.cor 0) len;
        output.len <- len;
      );
      output.tick <- tick;
  end

and cDefault = object(self) inherit hiddenConstant ~value:1. ()
  initializer
    self#setName "default";

(*
method setValue v = ()
method getKind = "default"
method talk port tick len = raise UndefinedTalker*)
end

let defTalker() = (new hiddenConstant() :> c)
(*
let defTalker = (new cDefault :> c)
let dt = new hiddenConstant ~value:1.() in
dt#setName "default";
dt#base
*)

type voice_t = c Voice.t
type talk_t = c Ear.talk_a
type bin_t = c Ear.bin_a
type ear_t = c Ear.t

let toTkr st = (st :> c)


let defVoice() = {
  vTag = defOutputTag; port = 0; tkr = defTalker(); tick = 0;
  cor = Cornet.make 1; len = 1;
}

let mkVoice ?(tag = defOutputTag) ?(port = 0) ?(tick = 0) ?value ?valInit
    ?(len = 1(*chunkSize*)) ?(talker = defTalker()) () = {
  vTag = tag; port; tkr = talker#base; tick;
  cor = (match valInit with
      | Some f -> Cornet.init ~f len
      | None -> (
          match value with
          |Some v -> Cornet.init ~v len
          | None -> Cornet.make len));
  len;
}

let toVc ?(tag="") ?(port=0) ?(tick=0) ?(value=0.) ?(len=1) talker =
  mkVoice ~tag ~port ~tick ~value ~len ~talker

let mkConstantVoice ?(tag = defInputTag) ?(value = 1.) () =
  {voice = (new hiddenConstant ~value ())#getVoice ""; tTag = tag}

(*let initTkr tkr = A.iter (fun vc -> vc.tkr <- tkr) tkr#getVoices*)

let mkTkr talker = let tkr = toTkr talker in (*initTkr tkr;*) tkr

let talkerToTalk ?(voice="") talker tTag =
  let tkr = mkTkr talker in {voice = tkr#getVoice voice; tTag}

let getTalks tkr = talksOfEars tkr#getEars

let earToTalk = function
  | EWord word -> talkerToTalk(new hiddenConstant ~value:word.value ()) word.wTag
  | ETalk talk -> talk
  | EBin bin -> (match bin.src with Talk talk -> talk
                                  | Word word -> talkerToTalk(new hiddenConstant ~value:word.value ()) word.wTag)
  | _ -> raise(UnexpectedValue "Unexpected ear type") 


let earsToTalks ears = L.map ~f:(fun t -> earToTalk t) ears

let mkWord ?(tag = defInputTag) ?(value = 0.) () = {value; wTag = tag}

let mkTalk ?(tag = defInputTag) ?value ?voice () =
  match value with Some v -> mkConstantVoice ~tag ~value:v ()
                 | None -> match voice with Some voice -> {voice; tTag = tag}
                                          | None -> mkConstantVoice ~tag ()

let mkBin ?(tag = defInputTag) ?src ?(value = 0.) () = match src with
  | Some src -> { src }
  | None -> {src = Word {value; wTag = tag}}

let mkWordBin ?(tag = defInputTag) value = {src = Word {value; wTag = tag}}

let mkTalkBin ?(tag = defInputTag) voice = {src = Talk {voice; tTag = tag}}

let mkWords ?(tag = defInputTag) () = {words = [||]; wsTag = tag}

let mkTalks ?(tag = defInputTag) () = {talks = [||]; tsTag = tag}

let mkBins ?(tag = defInputTag) () = { bins = [||]; bsTag = tag}


let getTalkValue tlk =

  let tkr = Ear.getTalkTalker tlk in

  if tkr#isHidden then
    match tkr#getValue with
    | Float v -> Some v
    | _ -> None
  else
    None


