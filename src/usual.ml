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

module A = ArrayLabels
module L = ListLabels
module S = String

let appName = "Graffophone"

let mini (x:int) (y:int) = if x < y then x else y
let maxi (x:int) (y:int) = if x > y then x else y
let minf (x:float) (y:float) = if x < y then x else y
let maxf (x:float) (y:float) = if x > y then x else y

let ini (min:int) (v:int) (max:int) = if v < min then min else if v > max then max else v
let inf (min:float) (v:float) (max:float) = if v < min then min else if v > max then max else v

let pi = 4.0 *. atan 1.0
let pi2 = 2. *. pi
let maxFloatArrLen = Sys.max_array_length / 2

let soi i = string_of_int i
let ios s = int_of_string s
let sof f = string_of_float f
let fos s = float_of_string s
let sob b = string_of_bool b
let soc c = S.make 1 c

let foi i = float_of_int i
let iof f = int_of_float f

let coi i = char_of_int i
let ioc c = int_of_char c

(* TRACE *)
(* Définition de quelques codes de couleurs pour les terminaux ANSI :
*)
let df_c = "\x1b[0m" (* clears all colors and styles (to white on black) *)

let fg_bk = "\x1b[30m"(* set foreground color to black *)
let fg_rd = "\x1b[31m"(* set foreground color to red *)
let fg_gr = "\x1b[32m"(* set foreground color to green *)
let fg_yl = "\x1b[33m"(* set foreground color to yellow *)
let fg_bl = "\x1b[34m"(* set foreground color to blue *)
let fg_mg = "\x1b[35m"(* set foreground color to magenta (purple) *)
let fg_cy = "\x1b[36m"(* set foreground color to cyan *)
let fg_wt = "\x1b[37m"(* set foreground color to white *)
let fg_df = "\x1b[39m"(* set foreground color to default (white) *)
let bg_bk = "\x1b[40m"(* set background color to black *)
let bg_rd = "\x1b[41m"(* set background color to red *)
let bg_gr = "\x1b[42m"(* set background color to green *)
let bg_yl = "\x1b[43m"(* set background color to yellow *)
let bg_bl = "\x1b[44m"(* set background color to blue *)
let bg_mg = "\x1b[45m"(* set background color to magenta (purple) *)
let bg_cy = "\x1b[46m"(* set background color to cyan *)
let bg_wt = "\x1b[47m"(* set background color to white *)
let bg_df = "\x1b[49m"(* set background color to default (black) *)

(*
let trace s = ()
*)
let trace s = print_endline s
let traceRed s = print_endline(fg_rd ^ s ^ fg_df)
let traceGreen s = print_endline(fg_gr ^ s ^ fg_df)
let traceBlue s = print_endline(fg_bl ^ s ^ fg_df)
let traceYellow s = print_endline(fg_yl ^ s ^ fg_df)
let traceMagenta s = print_endline(fg_mg ^ s ^ fg_df)
let traceCyan s = print_endline(fg_cy ^ s ^ fg_df)



let default d = function
  | None -> d
  | Some v -> v

let readFileLines filename =
  let lines = ref [] in
  let chan = open_in filename in
  try
    while true; do
      lines := input_line chan :: !lines
    done; []
  with End_of_file -> close_in chan; L.rev !lines

let writeFileLines filename lines =
  let chan = open_out filename in
  L.iter ~f:(fun l -> output_string chan l; output_char chan '\n') lines;
  close_out chan


(*
let declareUnexpectedValue name v =
if false then () else (
trace("Unexpected value "^v^" for "^name);
raise UnexpectedValue)

let declareUnexpectedAttribut name t =
if false then () else (
trace("Unexpected attribut "^t^" for "^name);
raise UnexpectedAttribut)
*)

let floatStrongWeight f =

  if f = 0. then 0.
  else (
    let absFloatStrongWeight f =
      if f < 1. then (
        let rec mul f d =
          if f < 1. then mul (f *. 10.) (d *. 10.)
          else floor f /. d
        in
        mul f 1.
      )
      else (
        let rec div f d =
          if f < 10. then floor f *. d
          else div (f /. 10.) (d *. 10.)
        in
        div f 1.
      )
    in
    copysign (absFloatStrongWeight(abs_float f)) f
  )

