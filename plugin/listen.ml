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

open Voice
open Ear
open Talker

type t = { offset : int; length : int; cornet : Cornet.t;}

let getOffset t = t.offset
let getLength t = t.length
let getCornet t = t.cornet



let voice voice ?(copy = true) tick len =
	
	if tick < voice.tick
	|| tick + len > voice.tick + voice.len
	then (
		voice.tkr#talk voice.port tick len;
	);
	let ofs = tick - voice.tick in
	let l = Util.mini len (voice.len - ofs) in
	
	if l < 1 then raise Voice.End;
	 
	if copy then {cornet = Cornet.sub voice.cor ofs l; offset = 0; length = l}
	else {cornet = voice.cor; offset = ofs; length = l}


let talk talk ?(copy = true) tick len = voice talk.voice ~copy tick len


let (@+) r i = Cornet.get r.cornet (i + r.offset)
let (@.) r i = Cornet.get r.cornet i


let blit src dest offset =
	Cornet.blit src.cornet src.offset dest.cor offset src.length;
	src.length
