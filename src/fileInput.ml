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
open Usual
open Factory
open SampleFormat

module Bus = EventBus
module Tkr = Talker


let sndfileErrorToString = function
	|	Sndfile.Unrecognised_format -> "Unrecognised format"
	|	Sndfile.System -> "System"
	|	Sndfile.Malformed_file -> "Malformed file"
	|	Sndfile.Unsupported_encoding -> "Unsupported encoding"
	|	Sndfile.Internal -> "Internal"
	| _ -> ""


let kind = "fileInput"

class c = object(self) inherit Tkr.c as super

	val mutable mFilename = ""
	val mutable mChannelsBufs : float array array array = [||]
	val mutable mNbBufs = 0
	val mutable mChannels = [||]
	
	method getValue = Tkr.fl2v mFilename
	method setValue value = self#setFilename(Tkr.v2fl value)
	
	method setFilename fileName =
		try
		let file = Sndfile.openfile fileName in
		let nbChs = Sndfile.channels file in
		let len = Int64.to_int(Sndfile.frames file)	in

		let mkCh p = Tkr.mkVoice ~tag:("Channel_" ^ soi(p + 1))
				~port:p ~len ~talker:self#base ()
		in
		mChannels <- A.init nbChs mkCh;

		let data = A.make (sf.rate * nbChs) 0.0 in

		let pos = ref 0 and nc = ref 0 in
		let readCount = ref (Sndfile.read file data) in

		while !readCount > 0 do
			let i = ref 0 in
			while !i < !readCount && !pos < len do

				Cornet.set (Voice.getCornet(mChannels.(!nc))) !pos data.(!i);

				if !nc = nbChs - 1 then (
					nc := 0;
					pos := !pos + 1;
				)
				else nc := !nc + 1;
			done;
			readCount := Sndfile.read file data;
		done;

		Sndfile.close file;
		mFilename <- fileName;
		with Sndfile.Error(error, msg) ->
(*			let errorCode = sndfileErrorToString error in*)
			Bus.notify(Bus.Error(fileName ^ " : " ^ msg));


	method getKind = kind
	method getVoices = mChannels

	method talk port tick len = ()

end

let handler = Plugin.{kind; category = "Input"; make = fun() -> (new c)#base}
