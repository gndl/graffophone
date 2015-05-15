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
open SampleFormat

let kind = "BigarrayVsArray"

class c = object(self) inherit Talker.c as super

	val mBA1 = Talker.mkTalk ()
	val mBA2 = Talker.mkTalk ()
	val mutable mA1 = Array.make SampleFormat.chunkSize 1.
	val mutable mA2 = Array.make SampleFormat.chunkSize 1.
	val mOutput = Talker.mkVoice ()

	method getKind = kind
	method getTalks = [mBA1; mBA2]
	method getVoices = [|mOutput|]

	method talk port tick len =
		let ba1 = Listen.talk mBA1 tick len ~copy:false in
		let ba2 = Listen.talk mBA2 tick (Listen.getLength ba1) ~copy:false in
		let l = Listen.getLength ba2 in

		Voice.checkLength mOutput l;
		
		if l > SampleFormat.chunkSize then (
			mA1 <- Array.make l 1.;
			mA2 <- Array.make l 1.;
		);

		Voice.setTick mOutput tick;
		Voice.setLength mOutput l;
		
  	let ba1Start = Unix.gettimeofday() in
  	
  	for n = 0 to 1000 do
    	for i = 0 to l - 1 do
				Voice.set mOutput i (Listen.(ba1@+i) +. 1.)
(*    		Bigarray.Array1.unsafe_set ba1 i(Bigarray.Array1.unsafe_get ba1 i +. 1.)*)
    	done
  	done;
  
  	print_endline("Bigarray : "^string_of_float(Unix.gettimeofday() -. ba1Start));
  
  	let fa1Start = Unix.gettimeofday() in
  
  	for n = 0 to 1000 do
    	for i = 0 to l - 1 do
    		mA2.(i) <- mA1.(i) +. 1.;
    	done
  	done;
  
  	print_endline("Array : "^string_of_float(Unix.gettimeofday() -. fa1Start));
(*  	
  	let fa2 = Array.make len 1. in
  	let ba2 = Bigarray.Array1.create Bigarray.float32 Bigarray.c_layout len in
  	let fa2Start = Unix.gettimeofday() in
  
  	for n = 0 to 1000 do
    	for i = 0 to len - 1 do
    		fa2.(i) <- fa1.(i) +. 1.;
    	done
  	done;
  
  	print_endline("Array : "^string_of_float(Unix.gettimeofday() -. fa2Start));
  	
  	let ba2Start = Unix.gettimeofday() in
  	
  	for n = 0 to 1000 do
    	for i = 0 to len - 1 do
    		Bigarray.Array1.unsafe_set ba2 i(Bigarray.Array1.unsafe_get ba1 i +. 1.)
    	done
  	done;
  
  	print_endline("Bigarray : "^string_of_float(Unix.gettimeofday() -. ba2Start))
*)
		
end

let handler = Plugin.{kind; category = "test"; make = fun() -> new c}

(*
let make() = let tkr = new c in (tkr :> Talker.c)
Plugin.(provideHandler := (fun() -> {
	named = kind;
	talkerHandlers = [{kind; category = "test"; make = (fun() -> new c)}]
}))
*)
(*

let register = Factory.addTalkerMaker kind "test" make;

let registerPlugin fileName =
	Factory.addTalkerMaker kind "System" make;
	print_string ("Plugin "^fileName^" registered\n");
	flush stdout;;

Factory.registerPlugin := registerPlugin;
*)