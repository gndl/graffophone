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

(* let backgroundColor       = Color.ofString "0x2A2A2AFF" (\* "Dim Gray" *\) *)
let backgroundColor       = Color.ofString "0x000000FF" (* "black" *)
(* let delimitationColor     = Color.ofString "0xA9A9A9FF" (\* "darkgray" *\) *)
let delimitationColor     = Color.ofString "0x505050FF"
(*let boxColor              = Color.ofString "0xEBEBD2FF" (* "beige" *) "Wheat" "Tan" "NavajoWhite" "PapayaWhip"*)
let boxColor       = Color.ofString "0x202020FF"

(* let selectionColor        = Color.ofString "0xE0E0FF0F" (\* "light blue" *\) *)
let selectionColor        = Color.ofString "0xFF8000FF" (* "orange" *)
let reverseSelectionColor = Color.ofString "0xE0FFE0FF" (* "light green" *)
let flowColor             = Color.ofString "0x00FFFFFF" (* "cyan" *)
let markerColor           = Color.ofString "0xFF8000FF" (* "orange" *)

let strBackgroundColor   = "Dim Gray"
let strDelimitationColor = "darkgray"
let strBoxColor          = "beige" (*"Wheat" "Tan" "NavajoWhite" "PapayaWhip"*)
let strSelection         = "white"
let strFlowColor         = "cyan"
let strMarkerColor       = "orange"

let gdkBackgroundColor       = Color.gdk backgroundColor
let gdkDelimitationColor     = Color.gdk delimitationColor
let gdkBoxColor              = Color.gdk boxColor
let gdkSelectionColor        = Color.gdk selectionColor
let gdkReverseSelectionColor = Color.gdk reverseSelectionColor
let gdkFlowColor             = Color.gdk flowColor
let gdkMarkerColor           = Color.gdk markerColor
(*
let gdkBackgroundColor   = GDraw.color (`NAME strBackgroundColor)
let gdkDelimitationColor = GDraw.color (`NAME strDelimitationColor)
let gdkBoxColor          = GDraw.color (`NAME strBoxColor)
let gdkSelection  = GDraw.color (`NAME strSelection)
let gdkFlowColor         = GDraw.color (`NAME strFlowColor)
*)
(*`NORMAL | `ACTIVE | `PRELIGHT | `SELECTED | `INSENSITIVE*)
(*Color.gtk backgroundColor*)
let background = [
  (`NORMAL, Color.gtk backgroundColor(*`NAME strBackgroundColor);
                                       (`ACTIVE, `NAME strBackgroundColor);
                                       (`PRELIGHT, `NAME strBackgroundColor);
                                       (`SELECTED, `NAME strBackgroundColor);
                                       (`INSENSITIVE, `NAME strBackgroundColor*));
]

(*
let defRgb8OfVoice voice =
let v = (Voice.getTalker voice)#getId + (Voice.getPort voice lsl 14) in
(57 + (v mod 19) * 11, 68 + (v mod 18) * 11, 63 + (v mod 17) * 12)
*)

let makeVoiceColor voice =
  let v = (Voice.getTalker voice)#getId + (Voice.getPort voice lsl 14) in
  let r = 95 + (v mod 3) * 80 in let g = 95 + (v mod 5) * 40 in let b = 75 + (v mod 7) * 30 in
  (*let r = 95 + (v mod 11) * 16 in let g = 95 + (v mod 9) * 20 in let b = 90 + (v mod 3) * 80 in*)
  Color.ofRgb8 r g b


let makeVoiceGdkColor voice = Color.gdk(makeVoiceColor voice)
(*let (r, g, b) = defRgb8OfVoice voice in
  Color.gdkOfRgb8 r g b
*)
