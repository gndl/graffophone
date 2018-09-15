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

open Usual

module Tkr = Graffophone_plugin.Talker
module GTkr = GTalker


let separatorProperties = [
  `FILL_COLOR_RGBA GTkr.boxBorderColor;
  `WIDTH_PIXELS 1]


class c (mixingConsole:MixingConsole.c) canvas = object (self)
  inherit GTkr.c mixingConsole#base canvas

  val mutable mGTracks : GTrack.c list = []

  method getGTracks = mGTracks
(*
 _______________
/     NAME      \
|---------------|
|[TRACK 1]      |
|---------------|
|[TRACK 2]      |
|---------------|
|volume #       |
\_______________/
*)
  method! draw pX pY =

    self#drawHeader pY false true false;

    let topTrackY = self#getHeight +. GTkr.space in

    let (w, h, gTrks) = L.fold_left mixingConsole#getTracks
        ~init:(self#getWidth +. 20., topTrackY, [])
        ~f:(fun (w, h, gTrks) track ->
            let gTrk = new GTrack.c track ~group:self#getGroup canvas in

            gTrk#setWidth w;

            gTrk#draw (1. -. GTkr.boxRadius) h;

            (max w gTrk#getWidth, h +. gTrk#getHeight, gTrk::gTrks)
          ) in

    self#setWidth(w -. GTkr.boxRadius);
    self#setHeight(h +. GTkr.marge -. pY);
    mGTracks <- gTrks;

    self#drawEarsVoices pY;
    self#drawBox pX pY;

    let w = self#getWidth in
    let points = [|pX; topTrackY; pX +. w; topTrackY|] in

    ignore(GnoCanvas.line ~points ~props:separatorProperties mGroup);

    ignore(L.fold_left gTrks ~init:topTrackY
             ~f:(fun y gTkr ->
                 let y = y +. gTkr#getHeight in
                 let points = [|pX; y; pX +. w; y|] in

                 ignore(GnoCanvas.line ~points ~props:separatorProperties mGroup);
                 y
               ));

end

let make mixingConsole canvas = new c mixingConsole canvas

let makeAt mixingConsole row column canvas =

  let gMc = new c mixingConsole canvas in

  gMc#setRow row;
  gMc#setColumn column;
  gMc
