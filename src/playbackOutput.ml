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

open Printf
open Usual
open Identifier
open SampleFormat
open Output

class c ?(name = "Playback Output(") () =
	let channelSize = sf.bits / 8 in
	object(self) inherit Output.c name
	val mutable mSampleSize = channelSize * sf.channels
	val mutable mDevice = None

		method openOutput nbChannels =
			mSampleSize <- channelSize * nbChannels;
			let drv = (*Ao.find_driver "pulse"*) Ao.get_default_driver() in
			try
				printf "\nOpen default device driver: %s\n" (Ao.driver_name drv);
				mDevice <- Some (Ao.open_live ~driver:drv ~bits:sf.bits ~rate:sf.rate ~channels:nbChannels ~byte_format:`LITTLE_ENDIAN ())
			with x -> printf "\nFailed to open device driver: %s\n" (Ao.driver_name drv)

	method write lg channels =
		let wrt device =
			let buf = String.create (lg * mSampleSize) in

			Array.iteri (fun nch ch -> 
					for i = 0 to lg - 1 do
						let sample = iof (ch.(i) *. maxA) in
						let pfb = coi (sample land 0xff) in
						let pfr = coi ((sample lsr 8) land 0xff) in
						let p = mSampleSize * i + nch * channelSize in

						buf.[p] <- pfb;
						buf.[p + 1] <- pfr;
					done;
				) channels;
			Ao.play device buf;
		in
		match mDevice with
			| Some dvc -> wrt dvc
			| None -> (self#openOutput (Array.length channels); match mDevice with
				| Some dvc -> wrt dvc
				| None -> ())

	method closeOutput =
		match mDevice with
			| None -> printf "No device to close"
			| Some device -> Ao.close device

	method backup = (Output.kind, "playback", [])
end

let make name attributs = toO(new c ~name ())

let register = Factory.addOutputMaker "playback" make


(*
	method write time buffer lg channels =
		let wrt device =
			let n = self#getMixingConsole#comeOut time buffer lg channels in
			let buf = String.create (lg * sampleSize) in

			Array.iteri (fun nch ch -> 
					for i = 0 to lg - 1 do
						let sample = iof ch.(i) in
						let pfb = coi (sample land 0xff) in
						let pfr = coi ((sample lsr 8) land 0xff) in
		
						buf.[sampleSize * i + nch * channelSize] <- pfb;
						buf.[sampleSize * i + nch * channelSize + 1] <- pfr;
					done;
					Ao.play device buf;
				) channels; n
		in
		match mDevice with
			| Some dvc -> wrt dvc
			| None -> (self#openOutput; match mDevice with 
				| Some dvc -> wrt dvc
				| None -> 0)
*)
