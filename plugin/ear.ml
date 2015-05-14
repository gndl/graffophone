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

let defInputTag = "I"

type word_t = {
	mutable value : float;
  wTag : string
}

type 'a talk_a = {
	mutable voice : 'a Voice.t;
  tTag : string
}

type 'a src_a = Word of word_t | Talk of 'a talk_a

type 'a bin_a = {
	mutable src : 'a src_a;
}

type words_t = {
	mutable words : word_t array;
  wsTag : string
}

type 'a talks_a = {
	mutable talks : 'a talk_a array;
  tsTag : string
}

type 'a bins_a = {
	mutable bins : 'a bin_a array;
  bsTag : string
}

type 'a t =
	| EWord of word_t | ETalk of 'a talk_a | EBin of 'a bin_a
	| EWords of words_t | ETalks of 'a talks_a | EBins of 'a bins_a


let defWord = {value = 0.; wTag = defInputTag}
let defSrc = Word defWord
let defEar = EWord defWord
(*let defBin = {src = defSrc}*)

let voicesToTalks voices talksTag =
	ListLabels.mapi ~f:(fun i voice ->
		{voice; tTag = talksTag ^ " " ^ string_of_int(i + 1)}) voices


let binTag bin = match bin.src with Word wrd -> wrd.wTag | Talk tlk -> tlk.tTag


let countEars ears =
	ArrayLabels.fold_left ~init:0 ears
		~f:(fun cnt ear -> match ear with
    		| EWord _ | ETalk _ | EBin _ -> cnt + 1
				| EWords e -> cnt + ArrayLabels.length e.words
    		| ETalks e -> cnt + ArrayLabels.length e.talks
    		| EBins e -> cnt + ArrayLabels.length e.bins
		)


let flattenEars ears =

	let flatEars = ArrayLabels.make (countEars ears) defEar in

	let blit src dstI =
		let len = ArrayLabels.length src in
		ArrayLabels.blit ~src ~src_pos:0 ~dst:flatEars ~dst_pos:dstI ~len;
		dstI + len
	in

	ignore(ArrayLabels.fold_left ~init:0 ears
	~f:(fun dstI ear -> match ear with
		| EWords e -> blit (ArrayLabels.map e.words ~f:(fun n -> EWord n)) dstI
		| ETalks e -> blit (ArrayLabels.map e.talks ~f:(fun f -> ETalk f)) dstI
		| EBins e -> blit (ArrayLabels.map e.bins ~f:(fun b -> EBin b)) dstI
		| e -> flatEars.(dstI) <- e; dstI + 1
		)
	);
	flatEars


let earsToSources ears =

	let earsSources = ArrayLabels.make (countEars ears) defSrc in

	let blit src dstI =
		let len = ArrayLabels.length src in
		ArrayLabels.blit ~src ~src_pos:0 ~dst:earsSources ~dst_pos:dstI ~len;
		dstI + len
	in

	ignore(ArrayLabels.fold_left ~init:0 ears
  	~f:(fun dstI ear -> match ear with
  		| EWord wrd -> earsSources.(dstI) <- Word wrd; dstI + 1
  		| ETalk tlk -> earsSources.(dstI) <- Talk tlk; dstI + 1
  		| EBin bin -> earsSources.(dstI) <- bin.src; dstI + 1
  		| EWords ews -> blit (ArrayLabels.map ews.words ~f:(fun w -> Word w)) dstI
  		| ETalks ets -> blit (ArrayLabels.map ets.talks ~f:(fun t -> Talk t)) dstI
  		| EBins ebs -> blit (ArrayLabels.map ebs.bins ~f:(fun b -> b.src)) dstI
		)
	);
	earsSources


let talksOfEars ears =

	ArrayLabels.fold_right ears ~init:[] ~f:(fun ear talks ->
		match ear with
		| EWord _ | EWords _ -> talks
		| ETalk talk -> talk::talks
		| EBin bin -> (
			match bin.src with Talk talk -> talk::talks | Word _ -> talks)
		| ETalks efs -> ListLabels.rev_append (ArrayLabels.to_list efs.talks) talks
		| EBins ebs -> ArrayLabels.fold_right ebs.bins ~init:talks ~f:(fun bin talks ->
			match bin.src with Talk talk -> talk::talks | Word _ -> talks)
	)


let getTalkTag talk = Voice.(talk.voice.vTag)
let getTalkPort talk = Voice.(talk.voice.port)
let getTalkTalker talk = Voice.(talk.voice.tkr)
let getTalkTick talk = Voice.(talk.voice.tick)
let getTalkLength talk = Voice.(talk.voice.len)
let getTalkCornet talk = Voice.(talk.voice.cor)

let voiceToTalk voice tTag = {voice; tTag}
