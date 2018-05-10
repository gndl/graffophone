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
open Util
open SampleFormat
(*open Array*)
module Tkr = Talker

let kind = "temporalInversion"

class c = object(self) inherit Tkr.c as super

	val mInput = Tkr.mkTalk ()
	val mPeriod = Tkr.mkTalk ~tag:"period" ()
	val mOutput = Tkr.mkVoice ()

	method getKind = kind
	method getTalks = [mInput; mPeriod]
	method getVoices = [|mOutput|]

	method talk port tick len =
		let ir = Listen.talk mInput tick len in
		let gr = Listen.talk mInput tick (Listen.getLength ir) ~copy:false in
		let grl = Listen.getLength gr in

		Voice.checkLength mOutput grl;

		for i = 0 to grl - 1 do
			Voice.set mOutput i Listen.((ir@+i) *. (gr@+i))
		done;

		Voice.setTick mOutput tick;
		Voice.setLength mOutput grl;
end

let handler = Plugin.{kind; category = "Handling"; make = fun() -> new c}

(*
let registerPlugin fileName =
	Factory.addTalkerMaker kind "Handling" make;
	print_string ("Plugin "^fileName^" registered\n");
	flush stdout;;

Factory.registerPlugin := registerPlugin;
*)
