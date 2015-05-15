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
open SampleFormat
open Draft
open Talker
open InnerTalkers
open Track
open Output
open MixingConsole
open Factory

let play time session =
	let sd = chunkSize in
	let buf = Array.make sd 0. in
	let t = ref 0 in let d = ref 1 in (*let tini = Sys.time() in*)
	trace("Open output at "^sof (Sys.time()));
	List.iter (fun (n, mc) -> mc#openOutput) (Session.mixCons session);

	let atLeastOneProductive = ref true in
	let i = ref 0 in
	while !i < time && !atLeastOneProductive do
		d := sd;
		atLeastOneProductive := false;

		ListLabels.iter (Session.mixCons session)
			~f:(fun (n, mc) ->
				if mc#isProductive then (
					try
						d := mc#comeOut !t buf !d;
						atLeastOneProductive := true
					with VoiceEnd -> mc#setProductive false;
				);
			);

		t := !t + !d;
		i := !i + !d / sf.rate;
		(*trace(soi !d ^" sample on "^soi !t^" played at "^sof (Sys.time()));*)
		flush stdout;
	done;

	trace(soi !t ^" sample played at "^sof (Sys.time()));
	trace("Duration : "^soi(!t / (60 * sf.rate))^" min "^soi((!t / sf.rate) mod 60)^" sec");
	List.iter (fun (n, mc) -> mc#closeOutput) (Session.mixCons session);
	trace("Close output at "^sof (Sys.time()));;

let () =
	
	let time = ref max_int in

	for i = 1 to Array.length Sys.argv - 1 do
		let param = Sys.argv.(i) in
		if Sys.file_exists param then play !time (Session.load param)
		else
			try time := ios param with Failure s -> ()
	done;

	if Array.length Sys.argv = 1 then
	(
		let ss_01 = mkTkr(new StaticSine.c ~freq:440. ()) in
		let a_01 = mkTkr(new Amplifier.c ~input:(toEar ss_01) ()) in
		let ss_02 =	mkTkr(new StaticSine.c ~freq:0.3 ()) in
		let ss_03 =	mkTkr(new StaticSine.c ~freq:0.5 ()) in
		let ss_04 =	mkTkr(new StaticSine.c ~freq:0.2 ()) in
		let ss_05 =	mkTkr(new StaticSine.c ~freq:21. ()) in
		let ips = [toEar ss_02; toEar ss_03; toEar ss_04; toEar ss_05; Con 0.5] in
		let p_01 = mkTkr(new Sum.c ~inputs:ips ()) in
		let am_01 = mkTkr(new AmplitudeModulator.c ~input:(toEar a_01) ~gain:(toEar p_01) ()) in
		let fm_01 = mkTkr(new FrequencyModulator.c ~input:(toEar am_01) ~freq:(toEar ss_04) ()) in
		let fm_02 = mkTkr(new FrequencyModulator.c ~input:(toEar am_01) ~freq:(toEar ss_02) ()) in
		let trk1 = new Track.c ~input:(toEar fm_01) ~channelsGains:[toEar ss_02; Con 0.] () in
		let trk2 = new Track.c ~input:(toEar fm_02) ~channelsGains:[Con 0.; toEar ss_04] () in
		let pbo = new PlaybackOutput.c () in
		let fo = new FileOutput.c ~filename:"sessions/default_session.wav" () in
		let mc = new MixingConsole.c ~tracks:[trk1; trk2] ~outputs:[(*pbo; *)fo] () in

		let session = Session.make [
				(ss_01#getName, ss_01);
				(a_01#getName, a_01);
				(ss_02#getName, ss_02);
				(ss_03#getName, ss_03);
				(ss_04#getName, ss_04);
				(ss_05#getName, ss_05);
				(p_01#getName, p_01);
				(am_01#getName, am_01);
				(fm_01#getName, fm_01);
				(fm_02#getName, fm_02);
				]
				[(trk1#getName, trk1); (trk2#getName, trk2)]
				[(mc#getName, mc)]
				[(pbo#getName, pbo); (fo#getName, fo)]
				~filename:"sessions/default_session.es"
		in
		play 60 session;
		Session.save session;
	);
	(*Unix.sleep 10;*)
	trace("End at "^sof (Sys.time()));
