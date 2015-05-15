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
(*
open SampleFormat
*)
module Tkr = Talker

let kind = "inversion"

class c = object(self) inherit Tkr.c as super

	val mInput = Tkr.mkTalk ()
	val mOutput = Tkr.mkVoice ()

	method getKind = kind
	method getTalks = [mInput]
	method getVoices = [|mOutput|]

	method talk port tick len =
		let ir = Listen.talk mInput tick len ~copy:false in
		let irl = Listen.getLength ir in

		Voice.checkLength mOutput irl;

		for i = 0 to irl - 1 do
			let v = Listen.(ir@+i) *. SampleFormat.maxA
			in
			if v = 0. then Voice.set mOutput i 1.
			else Voice.set mOutput i (minf (1. /. v) 1.)
		done;

		Voice.setTick mOutput tick;
		Voice.setLength mOutput irl;
end

let handler = Plugin.{kind; category = "Handling"; make = fun() -> new c}
