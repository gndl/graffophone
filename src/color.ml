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

type t = Int32.t

let locale = GtkMain.Main.init()

(* "0xRRGGBBAA" *)
let ofString rgbaString = Int32.of_string rgbaString


let ofRgb8 r g b = Int32.(
    let m = of_int 0xFF in
    let r = of_int r in
    let g = of_int g in
    let b = of_int b in
    logor
      (logor (shift_left (logand m r) 24) (shift_left (logand m g) 16))
      (logor (shift_left (logand m b) 8) m)
  )

let gdkRgb c = Int32.(
    let to16 c = c * 65535 / 255 in
    let m = of_int 0xFF in
    let r = to_int(logand m (shift_right_logical c 24)) in
    let g = to_int(logand m (shift_right_logical c 16)) in
    let b = to_int(logand m (shift_right_logical c 8)) in
    (*assert ( r < 255); assert ( g < 255); assert ( b < 255);(**)*)
    `RGB (to16 r, to16 g, to16 b)
  )

(*
let gdk c = Gdk.Color.alloc (Gdk.Color.get_system_colormap()) (rgb c)
let gdk c = GDraw.color (rgb c)
*)
let gdk c = Gdk.Color.alloc ~colormap:(Gdk.Rgb.get_cmap()) (gdkRgb c)


let gdkRgbOfRgb8 r g b =
  let to16 c = (c land 0xFF) * 65535 / 255 in
  `RGB (to16 r, to16 g, to16 b)

let gdkOfRgb8 r g b = Gdk.Color.alloc ~colormap:(Gdk.Rgb.get_cmap()) (gdkRgbOfRgb8 r g b)


let gtk c = gdkRgb c

let gtkOfRgb8 r g b = gdkRgbOfRgb8 r g b


let rgba c = c
(*
let rgba c = Int32.of_int(c.r lor (c.g lsl 8) lor (c.b lsl 16) lor (c.a lsl 24))
let rgba c = Int32.of_int(c.a lor (c.b lsl 8) lor (c.g lsl 16) lor (c.r lsl 24))
let rgba c = Int32.of_int((c.b land 0xFF) lor ((c.g land 0xFF) lsl 8) lor ((c.r land 0xFF) lsl 16) lor ((c.a land 0xFF) lsl 24))
*)

(*
type t = {r:int; g:int; b:int; a:int}
let make r g b a = {r; g; b; a}
*)
