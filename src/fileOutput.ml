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

class c ?(filename="") ?(name = "Sound File Output (") () =
	object(self) inherit Output.c name
	val mutable mFilename = filename
	val mutable mFile = None
	val mutable mNbChannels = sf.channels
	val mutable mBuf = A.make (chunkSize * sf.channels) 0.0

		method openOutput nbChannels =
			mNbChannels <- nbChannels;
			try
				let fmt = Sndfile.format Sndfile.MAJOR_WAV Sndfile.MINOR_PCM_16 in
				let info = (Sndfile.WRITE, fmt, nbChannels, sf.rate) in
				mFile <- Some(Sndfile.openfile ~info mFilename)
			with x -> trace ("faile to open file "^mFilename)

	method write lg channels =
		let wrt file =
			
			let bufSize = lg * mNbChannels in
			
			if A.length mBuf <> bufSize then mBuf <- A.make bufSize 0.0;

			for numCh = 0 to mNbChannels - 1 do
				let ch = channels.(numCh) in
				let p = ref numCh in
				
				for i = 0 to lg - 1 do
					mBuf.(!p) <- ch.(i);
					p := !p + mNbChannels;
				done;
			done;

			ignore(Sndfile.write file mBuf);
		in
		match mFile with
			| Some file -> wrt file
			| None -> (self#openOutput (A.length channels); match mFile with 
				| Some file -> wrt file
				| None -> ())

	method closeOutput =
		match mFile with
			| None -> printf "No file to close"
			| Some file -> Sndfile.close file

	method backup = (Output.kind, "file", [("filename", mFilename)])
end;;

let make name attributs =
	let filename = try
		let (t, d, _) = List.find (fun (t, d, _) -> t = "filename") attributs in d
	with Not_found -> "" in
	toO(new c ~filename ~name ())

let register = Factory.addOutputMaker "file" make
