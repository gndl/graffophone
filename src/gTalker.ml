(* 
 * Copyright (C) 2015 Ga�tan Dubreil
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

module Tkr = Talker

let marge = 3.
let space = marge +. marge

let fontSize = 10.
let nameFont = "-*-*-bold-r-*-*-*-*-*-*-*-*-*-*"
let kindFont = "-*-*-medium-i-*-*-*-*-*-*-*-*-*-*"
let valueFont = "-*-*-medium-r-*-*-*-*-*-*-*-*-*-*"
let tagFont = "-adobe-courier-medium-r-*-*-12-*-*-*-*-*-*-*"
let tagFontSelected = "-adobe-courier-bold-r-*-*-12-*-*-*-*-*-*-*"

(* let earColor = "cyan" beige SaddleBrown Sienna Brown *)

(* let nameProperties = [`FILL_COLOR "black"; `SIZE_POINTS fontSize(\*; `NO_FONT*\)] *)
let nameProperties = [`FILL_COLOR "white"; `SIZE_POINTS fontSize(*; `NO_FONT*)]
let kindProperties = [`FILL_COLOR "lightgray"; `SIZE_POINTS fontSize(*; `NO_FONT*)]
let valueProperties = [`FILL_COLOR "cyan"; `SIZE_POINTS fontSize(*; `NO_FONT*)]
let earProperties = [`FILL_COLOR "yellow"; `SIZE_POINTS fontSize; `X_OFFSET 0.; `Y_OFFSET 0. (*; `FONT tagFont; `NO_FONT*)]
let voiceProperties = [`FILL_COLOR "lightgreen"; `SIZE_POINTS fontSize; `X_OFFSET 0.; `Y_OFFSET 0. (*; `NO_FONT*)]
let selectedEarProperties = [`FILL_COLOR "magenta"; `SIZE_POINTS fontSize; `X_OFFSET (1.); `Y_OFFSET (-1.) (*`FONT tagFontSelected; *)]
let selectedVoiceProperties = [`FILL_COLOR "magenta"; `SIZE_POINTS fontSize; `X_OFFSET (-1.); `Y_OFFSET (-1.)]
let addProperties = [`FILL_COLOR "lightgreen"; `SIZE_POINTS fontSize(*; `NO_FONT*)]
let supProperties = [`FILL_COLOR "red"; `SIZE_POINTS fontSize(*; `NO_FONT*)]

let boxRadius = 4.
let boxBorder = 2 * iof boxRadius

let boxColorA1 = Int32.of_string "0xF5F5DCC0"
let boxColorA2 = Int32.of_string "0xF5F5DC60"
let boxBorderColorA = Int32.of_string "0x80808060"
let roundedBoxPropertiesA = [
  `FILL_COLOR_RGBA boxColorA1; 
  `OUTLINE_COLOR_RGBA boxColorA2; 
  `WIDTH_PIXELS (boxBorder - 2); 
  `JOIN_STYLE `ROUND]

let roundedBoxBorderPropertiesA = [
  `OUTLINE_COLOR_RGBA boxBorderColorA;
  `WIDTH_PIXELS boxBorder;
  `JOIN_STYLE `ROUND]

let boxColor1 = Int32.of_string "0xF5F5DCFF"
let boxColor2 = Int32.of_string "0xF5F5DCFF"
(*let boxBorderColor = Int32.of_string "0x808080FF"*)

let boxColor = Color.rgba(Style.boxColor)
let boxBorderColor = Color.rgba(Style.delimitationColor)
let selectedBoxColor = Color.rgba(Style.selectionColor)

let roundedBoxProperties = [
  `FILL_COLOR_RGBA boxColor;
  `OUTLINE_COLOR_RGBA boxColor;
  `WIDTH_PIXELS (boxBorder - 2);
  `JOIN_STYLE `ROUND]

let roundedBoxBorderProperties = [
  `OUTLINE_COLOR_RGBA boxBorderColor;
  `WIDTH_PIXELS boxBorder;
  `JOIN_STYLE `ROUND]

let boxProperties = [
  `FILL_COLOR_RGBA boxColor;
  `OUTLINE_COLOR_RGBA boxBorderColor;
  `WIDTH_PIXELS 1]

let selectedBoxProperties = [
  `FILL_COLOR_RGBA selectedBoxColor;
  `OUTLINE_COLOR_RGBA boxBorderColor;
  `WIDTH_PIXELS 1]


let connectionProperties = [`FILL_COLOR_RGBA (Color.rgba(Style.flowColor)); `WIDTH_PIXELS 1; `JOIN_STYLE `ROUND]
let connectionBorderProperties = [`FILL_COLOR_RGBA (Color.rgba(Style.backgroundColor))(*"gray" "turquoise"*); `WIDTH_PIXELS 3; `JOIN_STYLE `ROUND; `LAST_ARROWHEAD true]

let textHeight = fontSize +. space
let boxTop = textHeight +. boxRadius
let minimumHeight = boxTop +. textHeight *. 2. +. boxRadius


let makeEarText (tag, index, ear) = tag ^ " " ^ index

type gearType_t = GWord of Ear.word_t | GTalk of Tkr.talk_t | GAdd

type gEar_t = {
  earItem : GnoCanvas.text option;
  valueItem : GnoCanvas.text option;
  addItem : GnoCanvas.text option;
  supItem : GnoCanvas.text option;
  earY : float;
  earType : gearType_t;
  rootIndex : int;
}

type gVoice_t = { voiceItem : GnoCanvas.text; voiceY : float; voiceColor : Color.t}

let editValue pValueAction = function
  | Tkr.Float f -> GuiUtility.dialogFloatEntry f
                     (fun f fly -> pValueAction (Tkr.Float f) fly)
  | Tkr.String s -> GuiUtility.dialogStringEntry s
                      (fun s fly -> pValueAction (Tkr.String s) fly)
  | Tkr.Text t -> GuiUtility.dialogTextEntry t
                    (fun t fly -> pValueAction (Tkr.Text t) fly)
  | Tkr.File _ -> (match GuiUtility.openFile (*gui#toplevel*)() with
      | None -> ()
      | Some fn -> pValueAction (Tkr.File fn) false)
  | Tkr.Int i -> GuiUtility.dialogIntEntry i
                   (fun i fly -> pValueAction (Tkr.Int i) fly)
  | Tkr.Nil -> ()


let formatLabel max_len s =
  if S.length s > max_len then S.sub s 0 max_len ^ "..." else s

let formatName s = formatLabel 12 s
let formatValue s = formatLabel 6 s


class c (talker : Tkr.c) ?group (canvas : GnoCanvas.canvas) =
  object (self)
    val mutable mRow = -1
    val mutable mColumn = -1
    val mutable mDependentRow = -1
    val mutable mWidth = 0.
    val mutable mHeight = 0.
    val mutable mBoxTop = 0.
    val mutable mKindItem : GnoCanvas.text option = None
    val mutable mNameItem : GnoCanvas.text option = None
    val mutable mMainValueItem : GnoCanvas.text option = None
    val mutable mBoxItem : GnoCanvas.rect option = None
    val mutable mGEars : gEar_t array = [||]
    val mutable mGVoices : gVoice_t array = [||]
    val mGroup = match group with Some g -> g | None -> GnoCanvas.group canvas#root;

    method base = (self :> c)

    method getRow = mRow
    method getColumn = mColumn
    method getDependentRow = mDependentRow
    method getWidth = mWidth
    method getHeight = mHeight
    method getNameItem = mNameItem
    method getGEars = mGEars
    method getGVoices = mGVoices
    method getGroup = mGroup

    method setRow row = mRow <- row
    method setColumn column = mColumn <- column
    method setDependentRow row = mDependentRow <- row
    method setWidth width = mWidth <- width
    method setHeight height = mHeight <- height
    method setNameItem nameItem = mNameItem <- Some nameItem
    method setGEars gEars = mGEars <- gEars
    method setGVoices gVoices = mGVoices <- gVoices
    method getMainValueItem = mMainValueItem
    method getTalker = talker


    (*             KIND
                   _______________________________
                   /              NAME             \
                   |            [VALUE]            |
                   |TAG_INPUT_1 [1]  [TAG_OUTPUT_1]|
                   |TAG_INPUT_2 [2]                |
                   |TAG_INPUT_3 [3]  [TAG_OUTPUT_2]|
                   \_______________________________/
    *)
    method draw ?(pX = 0.) ?(pY = 0.) () =

      self#drawHeader ~pX ~pY true true true;
      self#drawEarsVoices ~pX ~pY ();
      self#drawBox ~pX ~pY ();


    method drawHeader ?(pX = 0.) ?(pY = 0.) drawKind drawName drawMainValue =

      mBoxTop <- if drawKind then (
          mKindItem <- Some(GnoCanvas.text ~text:talker#getKind ~y:pY
                              ~props:kindProperties ~anchor: `NORTH mGroup);

          pY +. textHeight +. boxRadius
        )
        else pY +. boxRadius;

      let mainValueY = if drawName && talker#getName <> "" then (
          let text = formatName talker#getName in

          let nameItem = GnoCanvas.text ~text ~y:mBoxTop
              ~props:nameProperties ~anchor: `NORTH mGroup in

          mWidth <- nameItem#text_width;
          self#setNameItem nameItem;
          mBoxTop +. textHeight
        )
        else mBoxTop in

      let mainValueText = formatValue talker#getStringOfValue in

      mHeight <- if drawMainValue && S.length mainValueText > 0 then (

          let mainValueItem = GnoCanvas.text ~text:mainValueText
              ~y:mainValueY ~props:valueProperties ~anchor: `NORTH mGroup in

          mMainValueItem <- Some mainValueItem;
          mWidth <- max mWidth mainValueItem#text_width;
          mainValueY +. textHeight -. pY
        )
        else mainValueY -. pY;


    method drawEarsVoices ?(pX = 0.) ?(pY = 0.) () =
      let ears = talker#getEars in
      let voices = talker#getVoices in

      let earsNb = A.length ears in
      let voicesNb = A.length voices in

      let top = pY +. mHeight in
      let earTop = top +. max (0.5 *. textHeight *. foi(voicesNb - earsNb)) 0. in
      let voiceTop = top +. max (0.5 *. textHeight *. foi(earsNb - voicesNb)) 0. in

      (* draw ears *)
      let drawEar (mw, y, tis, ri) ?tag ?value ?(sup = false) earType =

        let (earItem, w) = match tag with
          | None -> (None, 0.)
          | Some text -> let item = GnoCanvas.text ~text ~y
                             ~props:earProperties ~anchor: `NW mGroup
            in
            (Some item, item#text_width +. marge)
        in
        let (addItem, w) = match earType with
          | GAdd -> (
              let item = GnoCanvas.text ~text:"+" ~x:w ~y
                  ~props:addProperties ~anchor: `NW mGroup
              in
              (Some item, item#text_width +. marge)
            )
          | _ -> (None, w)
        in
        let (valueItem, w) =
          let text = match value with Some f -> formatValue(sof f) | None -> "#"
          in
          let item = GnoCanvas.text ~text ~x:w ~y
              ~props:valueProperties ~anchor: `NW mGroup
          in
          (Some item, w +. item#text_width +. marge)
        in
        let (supItem, w) = if sup then (
            let item = GnoCanvas.text ~text:"-" ~x:w ~y
                ~props:supProperties ~anchor: `NW mGroup
            in
            (Some item, w +. item#text_width +. marge)
          )
          else (None, w)
        in
        let gEar = { earItem; valueItem; addItem; supItem;
                     earY = y +. fontSize; earType; rootIndex = ri}
        in
        (max mw w, y +. textHeight, gEar::tis, ri)
      in

      let drawWord ~sup rem wrd = Ear.(
          drawEar rem ~tag:wrd.wTag ~value:wrd.value ~sup (GWord wrd)
        ) in

      let drawTalk ~sup rem tlk = Ear.(
          drawEar rem ~tag:tlk.tTag ?value:(Talker.getTalkValue tlk) ~sup (GTalk tlk)
        ) in

      let drawBin ~sup rem bin = match bin.Ear.src with
        | Ear.Word wrd -> drawWord rem wrd ~sup
        | Ear.Talk tlk -> drawTalk rem tlk ~sup
      in

      let checkEar (mw, y, tis, ri) ear =
        let rem = (mw, y, tis, ri)
        in
        let (mw, y, tis, ri) = Ear.(
            match ear with
            | EWord wrd -> drawWord ~sup:false rem wrd
            | ETalk tlk -> drawTalk ~sup:false rem tlk
            | EBin bin -> drawBin ~sup:false rem bin
            | EWords wrds ->
              drawEar(A.fold_left wrds.words ~init:rem ~f:(drawWord ~sup:true)) GAdd
            | ETalks tlks ->
              drawEar(A.fold_left tlks.talks ~init:rem ~f:(drawTalk ~sup:true)) GAdd
            | EBins bins ->
              drawEar(A.fold_left bins.bins ~init:rem ~f:(drawBin ~sup:true)) GAdd
          )
        in (mw, y, tis, ri + 1)
      in

      let (lw, lb, gEars, _) =
        A.fold_left ears ~init:(0., earTop, [], 0) ~f:checkEar
      in

      self#setGEars(A.of_list (L.rev gEars));

      (* draw voices *)
      let (rw, rb, gVoices) = A.fold_left voices ~init:(0., voiceTop, [])
          ~f:(fun (mw, y, tis) voice ->

              let voiceText = Voice.getTag voice in

              let voiceItem = GnoCanvas.text ~text:voiceText ~y
                  ~props:voiceProperties ~anchor: `NE mGroup
              in
              let voiceColor = Style.makeVoiceColor voice in

              let gVoice = {voiceItem; voiceY = y +. fontSize; voiceColor} in

              (max mw voiceItem#text_width, y +. textHeight, gVoice::tis)
            ) in

      self#setGVoices(A.of_list (L.rev gVoices));

      mWidth <- max mWidth (lw +. space +. rw);
      mHeight <- (max lb rb) -. pY


    method drawBox ?(pX = 0.) ?(pY = 0.) () =

      let x2 = pX +. mWidth +. marge in
      let y2 = pY +. mHeight +. marge in

      let box = GnoCanvas.rect ~x1:(pX -. marge) ~y1:(mBoxTop -. marge) ~x2 ~y2
          ~props:boxProperties mGroup
      in
      box#lower_to_bottom();

      mBoxItem <- Some box;

      self#positionTags();


    method drawRoundedBox ?(pX = 0.) ?(pY = 0.) () =

      let x2 = pX +. mWidth in
      let y2 = pY +. mHeight in

      let border = GnoCanvas.rect ~x1:pX ~y1:mBoxTop ~x2 ~y2 ~props:roundedBoxBorderProperties mGroup
      in
      let box = GnoCanvas.rect ~x1:pX ~y1:mBoxTop ~x2 ~y2 ~props:roundedBoxProperties mGroup
      in
      box#lower_to_bottom();
      border#lower_to_bottom();

      self#positionTags();


    method positionTags () =
      let middle = mWidth /. 2. in

      ignore(match mKindItem with
          | None -> ()
          | Some item -> item#move middle 0.);

      ignore(match mNameItem with
          | None -> ()
          | Some item -> item#move middle 0.);

      ignore(match mMainValueItem with
          | None -> ()
          | Some item -> item#move middle 0.);

      A.iter (fun gv -> gv.voiceItem#move ~x:mWidth ~y:0.) mGVoices;


    method getEars = talker#getEars

    method move x y = mGroup#move x y

    method drawAt x y =
      self#draw();
      self#move x y


    method drawConnections (gpTalkers : (int * c)list) (canvas : GnoCanvas.canvas) =

      A.fold_left mGEars ~init:0
        ~f:(fun index gEar ->
            try match gEar.earType with
              | GWord wdr -> index + 1
              | GTalk talk ->
                let tkr = Ear.getTalkTalker talk in
                let gTkr = L.assoc tkr#getId gpTalkers in

                let port = Ear.getTalkPort talk in

                if port < A.length gTkr#getGVoices then (

                  let voice = gTkr#getGVoices.(port)  in

                  let (x1, y1) = gTkr#getGroup#i2w ~x:gTkr#getWidth ~y:voice.voiceY in
                  let (x2, y2) = mGroup#i2w ~x:0. ~y:gEar.earY in

                  let tab = boxRadius +. marge in
                  let props = [`OUTLINE_COLOR_RGBA voice.voiceColor; `WIDTH_PIXELS 2] in

                  let bpath = GnomeCanvas.PathDef.new_path ~size:4 () in

                  GnomeCanvas.PathDef.moveto bpath x1 y1;
                  GnomeCanvas.PathDef.lineto bpath (x1 +. tab) y1;

                  if x2 >= x1 then (
                    let dx = (x2 -. x1) /. 2. in
                    GnomeCanvas.PathDef.curveto bpath
                      (x1 +. dx) y1 (x2 -. dx) y2 (x2 -. tab) y2;
                  )
                  else (
                    let dx = 10. *. tab in
                    let dy = (y2 -. y1) /. 2. in
                    GnomeCanvas.PathDef.curveto bpath
                      (x1 +. dx) (y1 +. dy) (x2 -. dx) (y2 -. dy) (x2 -. tab) y2;
                  );

                  GnomeCanvas.PathDef.lineto bpath x2 y2;

                  let line = GnoCanvas.bpath ~bpath ~props canvas#root in
                  line#lower_to_bottom();
                );
                index + 1
              | GAdd -> index
            with Not_found -> index + 1
          ) |> ignore


    method select =
      ignore(match mBoxItem with
            None -> ()
          | Some item -> item#set selectedBoxProperties)

    method unselect =
      ignore(match mBoxItem with
            None -> ()
          | Some item -> item#set boxProperties)

    method selectEar index =
      if index >= 0 && index < A.length mGEars then (
        match mGEars.(index).earItem with
          None -> ()
        | Some item -> item#set selectedEarProperties
      )

    method unselectEar index =
      if index >= 0 && index < A.length mGEars then (
        match mGEars.(index).earItem with
          None -> ()
        | Some item -> item#set earProperties
      )

    method selectVoice index =
      if index >= 0 && index < A.length mGVoices then (
        mGVoices.(index).voiceItem#set selectedVoiceProperties
      )

    method unselectVoice index =
      if index >= 0 && index < A.length mGVoices then (
        mGVoices.(index).voiceItem#set voiceProperties
      )


    method setActions (pGraphCtrl : GraphControler.c) =

      let editName() =
        GuiUtility.dialogStringEntry talker#getName (pGraphCtrl#setTalkerName talker)
      in
      let deleteTalker() = pGraphCtrl#deleteTalker talker in

      (* talker click action *)
      let popupMenuEntries = [
        `I ("Name", editName);
        `I ("Delete", deleteTalker);
      ]
      in

      (match mBoxItem with
       | None -> ()
       | Some item -> (item#connect#event(fun ie -> match ie with
           | `BUTTON_PRESS ev ->
             let button = GdkEvent.Button.button ev in

             if button = 1 then (
               pGraphCtrl#setSelectedTalker talker
             )
             else if button = 3 then (
               GToolbox.popup_menu ~entries:popupMenuEntries ~button
                 ~time:(GdkEvent.Button.time ev)
             );
             true
           | _ -> false
         )) |> ignore) |> ignore;

      (* talker name action *)
      (match mNameItem with
       | None -> ()
       | Some item -> (item#connect#event(function
           | `BUTTON_RELEASE ev -> editName(); true
           | _ -> false
         )) |> ignore) |> ignore;

      (* talker value action *)
      (match mMainValueItem with
       | None -> ()
       | Some item -> (item#connect#event(function
           | `BUTTON_RELEASE ev ->
             talker#getValue |> editValue (pGraphCtrl#setTalkerValue talker);
             true
           | _ -> false
         )) |> ignore) |> ignore;

      (* ear action *)
      A.iteri mGEars ~f:(fun index gEar ->
          (* ear action *)
          (match gEar.earItem with
           | None -> ()
           | Some item -> (item#connect#event(function
               | `BUTTON_RELEASE ev -> pGraphCtrl#selectEar talker index;
                 true
               | _ -> false
             )) |> ignore) |> ignore;
          (* earvalue action *)
          (match gEar.valueItem with
           | None -> ()
           | Some item -> (item#connect#event(fun ie ->
               match ie with
               | `BUTTON_RELEASE ev ->

                 if gEar.addItem <> None then
                   GuiUtility.dialogFloatEntry 0. (fun value fly ->
                       if not fly then
                         pGraphCtrl#addTalkerEarToValueByIndex talker gEar.rootIndex value
                     )
                 else (
                   let initValue = match talker#getEarsSources.(index) with
                     | Ear.Word w -> w.Ear.value
                     | Ear.Talk tlk -> match Talker.getTalkValue tlk with
                       | Some f -> f
                       | None -> 0.
                   in
                   GuiUtility.dialogFloatEntry initValue (fun value fly ->
                       pGraphCtrl#setTalkerEarToValueByIndex talker index value fly
                     )
                 );
                 true
               | _ -> false
             )) |> ignore) |> ignore;
          (* add action *)
          (match gEar.addItem with
           | None -> ()
           | Some item -> (item#connect#event(fun ie ->
               match ie with
               | `BUTTON_RELEASE ev -> pGraphCtrl#addEar talker gEar.rootIndex;
                 true
               | _ -> false
             )) |> ignore) |> ignore;
          (* sup action *)
          (match gEar.supItem with
           | None -> ()
           | Some item -> (item#connect#event(fun ie ->
               match ie with
               | `BUTTON_RELEASE ev -> pGraphCtrl#supEar talker index;
                 true
               | _ -> false
             )) |> ignore) |> ignore;
        );

      (* voice action *)
      A.iteri mGVoices ~f:(fun index gVoice ->
          gVoice.voiceItem#connect#event(fun ie ->
              match ie with
              | `BUTTON_RELEASE ev ->
                if GdkEvent.Button.button ev = 3 then
                  pGraphCtrl#showVoice talker index
                else
                  pGraphCtrl#selectVoice talker index;
                true
              | _ -> false
            ) |> ignore
        );
  end

let make talker canvas = new c talker canvas

let makeAt talker row column canvas =

  let gTkr = new c talker canvas in

  gTkr#setRow row;
  gTkr#setColumn column;
  gTkr
