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

type t = {
  bits : int; (* bits per sample *)
  rate : int; (* samples per second (in a single channel) *)
  channels : int; (* number of audio channels *)
(*  byte_format : Ao.byte_format_t; (* Byte ordering in sample, see constants below *)
*)}

let sf = {bits = 16; rate = 44100; channels = 2(*; byte_format = `LITTLE_ENDIAN*)}

let bits = sf.bits
let rate = sf.rate
let fRate = float_of_int sf.rate
let channels = sf.channels

(*
let minAudio = min_float -. 1.
let maxAudio = 1. -. min_float*)
let minAudio = (-0.999)
let maxAudio = 0.999
let maxA = (2. ** float_of_int sf.bits) /. 2. -. 1.
let minA = 1. -. maxA
let perRate = 1. /. fRate
let per2Pi = 1. /. pi2
let coefOfFrequence freq = freq *. pi2 *. perRate
let frequenceOfCoef coef = coef *. per2Pi *. fRate

let chunkSize = sf.rate / 10 (* * 10 *)
