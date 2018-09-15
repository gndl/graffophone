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

module Tkr = Talker
module Trk = Track
module GTkr = GTalker

let boxProperties = [
  `FILL_COLOR_RGBA (Color.rgba(Style.boxColor));
  `OUTLINE_COLOR_RGBA (Color.rgba(Style.delimitationColor));
  `WIDTH_PIXELS 1]

class c (track:Track.c) ?group canvas =
  object (self) inherit GTkr.c (track :> Tkr.c) ?group canvas


    method! getTalker = (track :> Tkr.c)
(*
 _____________________
|        NAME         |
|in #                 |
|gain #               |
|channelGain 1 #      |
|channelGain 2 #      |
|_____________________|
*)
    method! draw _ pY =

      self#drawHeader pY false true false;

      self#drawEarsVoices (pY +. GTkr.marge);

      self#setWidth(self#getWidth +. GTkr.space);
      self#setHeight(self#getHeight +. GTkr.space);

      (*self#drawBox ~pX ~pY ();*)
      self#positionTags();


    method! drawBox pX pY =

      let x2 = pX +. mWidth in
      let y2 = pY +. mHeight in

      let box = GnoCanvas.rect ~x1:pX ~y1:pY ~x2 ~y2 ~props:boxProperties mGroup
      in
      box#lower_to_bottom();

  end

let make mixingConsole canvas = new c mixingConsole canvas

let makeAt mixingConsole row column canvas =

  let gMc = new c mixingConsole canvas in

  gMc#setRow row;
  gMc#setColumn column;
  gMc
