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

open Usual
open BasicTalkers

module SF = SampleFormat
module Tkr = Talker

let sMixingConsoleCount = ref 0

let kind = "mixingConsole"

class c ?(tracks = []) ?(outputs = []) ?(nbChannels = SF.channels) () =
	object(self) inherit Tkr.c

	val mutable mTracks = tracks
	val mutable mOutputs: Output.c list = outputs
	val mutable mChannels = A.make_matrix nbChannels SF.chunkSize 0. 
	val mVolume = Tkr.mkBin ~tag:"volume" ~value:1. ()
	val mutable mTick = 0
	val mutable mProductive = false
	
	initializer
		self#setName "mixer"


	method getKind = kind

	method getTracks = mTracks
	method setTracks ts = mTracks <- ts
	method addTrack t = mTracks <- t :: mTracks

	method createTrack =
		let trackName = Track.kind ^ soi(L.length mTracks + 1) in
		let trk = new Track.c in
		trk#setName trackName;
		mTracks <- trk :: mTracks;
		trk

	method setOutputs os = mOutputs <- os
	method setChannels nb = mChannels <- A.make_matrix nb SF.chunkSize 0.
	method isProductive = mProductive
	method setProductive productive = mProductive <- productive 


	method openOutput =
		L.iter (fun o -> o#openOutput (A.length mChannels)) mOutputs;
		mTick <- 0;
		mProductive <- true;


	method comeOut tick buf len =
		let ol = ref len in
	
		if len > A.length mChannels.(0) then
			mChannels <- A.make_matrix (A.length mChannels) len 0.;

		ol := (L.hd mTracks)#put false tick buf !ol mChannels;
		L.iter(fun t -> ol := t#put true tick buf !ol mChannels)(L.tl mTracks);

		let computeMasterVolume = function
				| Ear.Word mvc -> ( A.iter (fun c ->
					for j = 0 to !ol - 1 do
						c.(j) <- inf SF.minAudio (c.(j) *. mvc.Ear.value) SF.maxAudio done;
					) mChannels;)
				| Ear.Talk mve -> (
					let mvr = Listen.talk mve tick !ol ~copy:false in
					ol := Listen.getLength mvr;

					A.iter (fun c ->
						for j = 0 to !ol - 1 do
							c.(j) <- inf SF.minAudio (c.(j) *. Listen.(mvr @+ j)) SF.maxAudio done;
					) mChannels;
				);
		in
		computeMasterVolume mVolume.Ear.src;

		L.iter (fun o -> o#write !ol mChannels) mOutputs;
		!ol


	method comeOut2 tick buf len =
		if mTick > tick then mTick - tick
		else (
			let ownLen = len + (tick - mTick) in
			let ol = ref ownLen in
	
			if ownLen > A.length mChannels.(0) then
				mChannels <- A.make_matrix (A.length mChannels) ownLen 0.;

			ol := (L.hd mTracks)#put false mTick buf !ol mChannels;
			L.iter(fun t -> ol := t#put true mTick buf !ol mChannels)(L.tl mTracks);

			let computeMasterVolume = function
					| Ear.Word mvc -> ( A.iter (fun c ->
						for j = 0 to !ol - 1 do
							c.(j) <- inf SF.minAudio (c.(j) *. mvc.Ear.value) SF.maxAudio done;
						) mChannels;)
					| Ear.Talk mve -> (
						let mvr = Listen.talk mve mTick !ol ~copy:false in
						ol := Listen.getLength mvr;

						A.iter (fun c ->
							for j = 0 to !ol - 1 do
								c.(j) <- inf SF.minAudio (c.(j) *. Listen.(mvr @+ j)) SF.maxAudio done;
						) mChannels;
					);
			in
			computeMasterVolume mVolume.Ear.src;

			L.iter (fun o -> o#write !ol mChannels) mOutputs;
			mTick <- mTick + !ol;
			!ol
		);


	method closeOutput =
		L.iter (fun o -> o#closeOutput) mOutputs;
		mProductive <- false;
(*
	method getTalks = match mVolume with
		| Ear.Talk mve -> [("volume", "", mve)]
		| Ear.Word _ -> []
*)

	method getEars = [|Ear.EBin mVolume|]


	method mixingConsoleBackup =
		let ears = self#getEars in
		let trks = L.map (fun trk -> (Track.kind, trk)) mTracks in
		let ops = L.map (fun o -> (Output.kind, o)) mOutputs in
		(kind, "", ears, trks, ops)
end

let make name attributs trks ops =
	let rec a2p ts os = function
		| (t, n, tkn)::tl when t = Track.kind -> a2p ((L.assoc n trks)::ts) os tl
		| (t, n, tkn)::tl when t = Output.kind -> a2p ts ((L.assoc n ops)::os) tl
		| [] -> (ts, os)
		| _::tl -> a2p ts os tl
	in
	let (tracks, outputs) = a2p [] [] attributs
	in
	let mxc = new c ~tracks ~outputs () in
	mxc#setName name;
	mxc
