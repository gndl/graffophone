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

exception End
exception Ok

let defOutputTag = "O"


type port_t = int

type 'a t = {
  mutable tick : int;
  mutable len : int;
  mutable cor : Cornet.t;
	mutable tkr : 'a;
  port : port_t;
  vTag : string
}

let init l f = Cornet.init l ~f
let get voice i = Cornet.get voice.cor i
let set voice i v = Cornet.set voice.cor i v
let fill voice ofs len v = Cornet.fill voice.cor ofs len v
let add voice i v = Cornet.set voice.cor i (Cornet.get voice.cor i +. v)
let sub voice i v = Cornet.set voice.cor i (Cornet.get voice.cor i -. v)
let mul voice i v = Cornet.set voice.cor i (Cornet.get voice.cor i *. v)
let div voice i v = Cornet.set voice.cor i (Cornet.get voice.cor i /. v)
let dim voice = Cornet.dim voice.cor

let getTag voice = voice.vTag
let getIdentity voice = voice.tkr#getName ^ "." ^ voice.vTag
let getPort voice = voice.port
let getTalker voice = voice.tkr
let getTick voice = voice.tick
let getLength voice = voice.len
let getCornet voice = voice.cor

let setTalker voice v = voice.tkr <- v
let setTick voice v = voice.tick <- v
let setLength voice v = voice.len <- v
let setCornet voice v = voice.cor <- v

let checkLength voice len =
	if Cornet.dim voice.cor < len then voice.cor <- Cornet.make len
