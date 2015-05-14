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
open Ear

module Tkr = Talker
module SF = SampleFormat

module Constant = struct
	let kind = "constant"
class c = object(self) inherit Tkr.hiddenConstant()
	method getKind = kind
	method isHidden = false
end

	let make() = (new c)#base
end

module Sum = struct
	let kind = "sum"

class c = object(self) inherit Tkr.c as super

	val mInputs = Tkr.mkBins ()
	val mOutput = Tkr.mkVoice ()

	method getKind = kind
	method getEars = [|EBins mInputs|]
	method getVoices = [|mOutput|]

	method talk port tick len =
		let nbIn = A.length mInputs.bins in
		
		if nbIn < 1 then raise Voice.End;

		Voice.checkLength mOutput len;

		let computeFirstInput bin = match bin.src with
			| Word word -> Voice.fill mOutput 0 len word.value; len
			| Talk talk -> (
				let r = Listen.talk talk tick len ~copy:false in
				Listen.blit r mOutput 0)
		in
		let l = ref(computeFirstInput mInputs.bins.(0)) in

		for i = 1 to nbIn - 1 do
			match (mInputs.bins.(i).src) with
				| Word word ->
					let value = word.value in

					for j = 0 to !l - 1 do
						Voice.add mOutput j value done;
				| Talk talk -> (
					let r = Listen.talk talk tick !l ~copy:false in

					for j = 0 to Listen.getLength r - 1 do
						Voice.add mOutput j Listen.(r @+ j) done;
					l := Listen.getLength r;)
		done;

		Voice.setTick mOutput tick;
		Voice.setLength mOutput !l;
end

let make() = (new c)#base
end


module Product = struct
	let kind = "product"

class c = object(self) inherit Tkr.c as super

	val mInputs = Tkr.mkBins ()
	val mOutput = Tkr.mkVoice ()

	method getKind = kind
	method getEars = [|EBins mInputs|]
	method getVoices = [|mOutput|]

	method talk port tick len =
		let nbIn = A.length mInputs.bins in
		
		if nbIn < 1 then raise Voice.End;

		Voice.checkLength mOutput len;

		let computeFirstInput bin = match bin.src with
			| Word word -> Voice.fill mOutput 0 len word.value; len
			| Talk talk -> (
				let r = Listen.talk talk tick len ~copy:false in
				Listen.blit r mOutput 0)
		in
		let l = ref(computeFirstInput mInputs.bins.(0)) in

		for i = 1 to nbIn - 1 do
			match (mInputs.bins.(i).src) with
				| Word word ->
					let value = word.value in

					for j = 0 to !l - 1 do
						Voice.mul mOutput j value done;
				| Talk talk -> (
					let r = Listen.talk talk tick !l ~copy:false in

					l := Listen.getLength r;
					
					for j = 0 to !l - 1 do
						Voice.mul mOutput j Listen.(r @+ j) done;
				)
		done;
		Voice.setTick mOutput tick;
		Voice.setLength mOutput !l;

end

let make() = (new c)#base
end


module Average = struct
	let kind = "average"

class c = object(self) inherit Sum.c as super

	method getKind = kind

	method talk port tick len =
		
		super#talk port tick len;
		
		let invNbIn = 1. /. foi(A.length mInputs.bins) in

		for i = 0 to Voice.getLength mOutput - 1 do
			Voice.mul mOutput i invNbIn
		done;
end

let make() = (new c)#base
end


module StaticSine = struct
	let kind = "staticSinusoidal"

class c = object(self) inherit Tkr.c as super

	val mutable mFreqCoef = 0.
	val mutable mPeriod = 1
	val mutable mLargeOrDecimalPeriod = true
	val mOutput = Tkr.mkVoice ()

	method setFreq freq =
		mFreqCoef <- SF.coefOfFrequence freq;
		let period = SF.fRate /. freq in
		mPeriod <- iof period;
		
		if mPeriod = 0 then (
			mPeriod <- 1);
		
		mLargeOrDecimalPeriod <- mPeriod > SF.chunkSize || floor period <> period;
		self#setValues (Voice.dim mOutput);

	method setValues len =
		let l = mini len maxFloatArrLen in
		Voice.setCornet mOutput (
			Voice.init l (fun i -> sin(foi(i + Voice.getTick mOutput) *. mFreqCoef))
		);
		Voice.setLength mOutput l;

	method getValue = Tkr.f2v(SF.frequenceOfCoef mFreqCoef)
	method takeValue = true
	method setValue v = self#setFreq (Tkr.v2f v)

	method getKind = kind
	method getVoice name = mOutput
	method getVoices = [|mOutput|]

	method talk port tick len =

		if mLargeOrDecimalPeriod then
		(
			Voice.setTick mOutput tick;
			if Voice.dim mOutput < len then self#setValues len
			else (
				for i = 0 to len - 1 do
					Voice.set mOutput i (sin(foi(i + tick) *. mFreqCoef));
				done;
				Voice.setLength mOutput len;
			);
		)
		else (
			let ofs = tick mod mPeriod in
			let l = ofs + len in

			if Voice.dim mOutput < l then self#setValues (l + mPeriod);

			Voice.setTick mOutput (tick - ofs);
		);
end

let make() = (new c)#base
end


module Sine = struct
	let kind = "sinusoidal"

class c = object(self) inherit Tkr.c as super

	val mFreq = Tkr.mkBin ~tag:"frequence" ()
	val mPhase = Tkr.mkBin ~tag:"phase" ()
	val mOutput = Tkr.mkVoice ()
	
	val mutable mLastTick = 0
	val mutable mLastFreq = 0.
	val mutable mLastAngle = 0.

	method getEars = [|EBin mFreq; EBin mPhase|]
	method getKind = kind
	method getVoice name = mOutput
	method getVoices = [|mOutput|]

	method talk port tick len =
		Voice.checkLength mOutput len;
		
		let compute = function
			| Word freq -> (
				let freqCoef = SF.coefOfFrequence freq.value in
				let phsCoef = SF.fRate /. (pi *. freq.value) in
				match mPhase.src with
					| Word phs -> (
						let p = foi tick +. (phs.value *. phsCoef)
						in
						for i = 0 to len - 1 do
							let fti = foi i +. p in
							Voice.set mOutput i (sin(fti *. freqCoef));
						done; len)
					| Talk phs -> (
						let pr = Listen.talk phs tick len ~copy:false in
						let l = Listen.getLength pr
						in
						for i = 0 to l - 1 do
							let t = foi(tick + i) *. freqCoef in
							let p = Listen.(pr@+i) *. pi in
							Voice.set mOutput i (sin(t +. p));
						done; l))
			| Talk fTalk -> (
				if mLastTick <> tick then (
					mLastFreq <- 0.;
					mLastAngle <- 0.;
				);
				
				let fr = Listen.talk fTalk tick len ~copy:false in
				let fl = Listen.getLength fr
				in
				match mPhase.src with
					| Word phs -> (
						let p = phs.value *. pi
						in
						for i = 0 to fl - 1 do
							let a = mLastAngle +. SF.coefOfFrequence mLastFreq in
							Voice.set mOutput i (sin(a +. p));
							mLastFreq <- Listen.(fr@+i);
							mLastAngle <- a;
						done;
						mLastTick <- tick + fl;
						fl)
					| Talk phs -> (
						let pr = Listen.talk phs tick fl in
						let l = Listen.getLength pr in

						for i = 0 to l - 1 do
							let p = Listen.(pr@+i) *. pi in
							let a = mLastAngle +. SF.coefOfFrequence mLastFreq in
							Voice.set mOutput i (sin(a +. p));
							mLastFreq <- Listen.(fr@+i);
							mLastAngle <- a;
						done;
						mLastTick <- tick + l;
						l))
		in
		Voice.setTick mOutput tick;
		Voice.setLength mOutput (compute mFreq.src);
end

let make() = (new c)#base
end


module AbsSine = struct
	let kind = "absSine"

class c = object(self) inherit Tkr.c as super

	val mFreq = Tkr.mkBin ~tag:"frequence" ()
	val mPhase = Tkr.mkBin ~tag:"phase" ()
	val mOutput = Tkr.mkVoice ()

	method getEars = [|EBin mFreq; EBin mPhase|]
	method getKind = kind
	method getVoice name = mOutput
	method getVoices = [|mOutput|]

	method talk port tick len =
		Voice.checkLength mOutput len;
		
		let compute = function
			| Word freq -> (
				let freqCoef = SF.coefOfFrequence freq.value in
				let phsCoef = SF.fRate /. (pi *. freq.value) in
				match mPhase.src with
					| Word phs -> (
						let p = foi tick +. (phs.value *. phsCoef)
						in
						for i = 0 to len - 1 do
							let fti = foi i +. p in
							Voice.set mOutput i (sin(fti *. freqCoef));
						done; len)
					| Talk phs -> (
						let pr = Listen.talk phs tick len ~copy:false in
						let l = Listen.getLength pr
						in
						for i = 0 to l - 1 do
							let t = foi(tick + i) *. freqCoef in
							let p = Listen.(pr@+i) *. pi in
							Voice.set mOutput i (sin(t +. p));
						done; l))
			| Talk fTalk -> (
				let fr = Listen.talk fTalk tick len ~copy:false in
				let fl = Listen.getLength fr
				in
				match mPhase.src with
					| Word phs -> (
						let p = phs.value *. pi
						in
						for i = 0 to fl - 1 do
							let t = foi(tick + i) *. SF.coefOfFrequence Listen.(fr@+i) in
							let v = sin(t +. p) in
							Voice.set mOutput i v;
						done; fl)
					| Talk phs -> (
						let pr = Listen.talk phs tick fl in
						let l = Listen.getLength pr in

						for i = 0 to l - 1 do
							let t = foi(tick + i) *. SF.coefOfFrequence Listen.(fr@+i) in
							let p = Listen.(pr@+i) *. pi in
							let v = sin(t +. p) in
							Voice.set mOutput i v;
						done; l))
		in
		Voice.setTick mOutput tick;
		Voice.setLength mOutput (compute mFreq.src);
end

let make() = (new c)#base
end


module BSine = struct
	let kind = "bsine"

class c = object(self) inherit Tkr.c as super

	val mFreq = Tkr.mkTalk ~tag:"frequence"  ~value:440. ()
	val mPhase = Tkr.mkTalk ~tag:"phase" ~value:0. ()
	val mRoof = Tkr.mkTalk ~tag:"roof" ~value:1. ()
	val mFloor = Tkr.mkTalk ~tag:"floor" ~value:0. ()
	val mOutput = Tkr.mkVoice ()
	
	val mutable mLastTick = 0
	val mutable mLastFreq = 0.
	val mutable mLastAngle = 0.

	method getTalks = [mFreq; mPhase; mRoof; mFloor]
	method getKind = kind
	method getVoice name = mOutput
	method getVoices = [|mOutput|]

	method talk port tick len =

		Voice.checkLength mOutput len;

		if mLastTick <> tick then (
			mLastFreq <- 0.;
			mLastAngle <- 0.;
		);
				
		let freq = Listen.talk mFreq tick len in
		let phase = Listen.talk mPhase tick (Listen.getLength freq) in
		let roof = Listen.talk mRoof tick (Listen.getLength phase) in
		let floor = Listen.talk mFloor tick (Listen.getLength roof) ~copy:false in
		let l = Listen.getLength floor in

		for i = 0 to l - 1 do
			let p = Listen.(phase@+i) *. pi in
			let a = mLastAngle +. SF.coefOfFrequence mLastFreq in
			let v = sin(a +. p) in
			let rv = Listen.(roof@+i) in
			let fv = Listen.(floor@+i) in

			Voice.set mOutput i ((((v *. 0.5) +. 0.5) *. (rv -. fv)) +. fv);

			mLastFreq <- Listen.(freq@+i);
			mLastAngle <- a;
		done;
		mLastTick <- tick + l;
		Voice.setTick mOutput tick;
		Voice.setLength mOutput l;
end

let make() = (new c)#base
end


module AbsBSine = struct
	let kind = "absBsine"

class c = object(self) inherit Tkr.c as super

	val mFreq = Tkr.mkTalk ~tag:"frequence"  ~value:440. ()
	val mPhase = Tkr.mkTalk ~tag:"phase" ~value:0. ()
	val mRoof = Tkr.mkTalk ~tag:"roof" ~value:1. ()
	val mFloor = Tkr.mkTalk ~tag:"floor" ~value:0. ()
	val mOutput = Tkr.mkVoice ()

	method getTalks = [mFreq; mPhase; mRoof; mFloor]
	method getKind = kind
	method getVoice name = mOutput
	method getVoices = [|mOutput|]

	method talk port tick len =

		Voice.checkLength mOutput len;

		let freq = Listen.talk mFreq tick len in
		let phase = Listen.talk mPhase tick (Listen.getLength freq) in
		let roof = Listen.talk mRoof tick (Listen.getLength phase) in
		let floor = Listen.talk mFloor tick (Listen.getLength roof) ~copy:false in
		let l = Listen.getLength floor in
		
		for i = 0 to l - 1 do
			let fti = foi(tick + i) +. Listen.(phase@+i) *. pi2 in
			let v = sin(fti *. SF.coefOfFrequence Listen.(freq@+i)) in
			let rv = Listen.(roof@+i) in
			let fv = Listen.(floor@+i) in

			Voice.set mOutput i ((((v *. 0.5) +. 0.5) *. (rv -. fv)) +. fv);
		done;
		Voice.setTick mOutput tick;
		Voice.setLength mOutput l;
end

let make() = (new c)#base
end


module Square = struct
	let kind = "square"

class c = object(self) inherit Tkr.c as super

	val mAmpli = Tkr.mkTalk ~tag:"amplitude" ()
	val mFreq = Tkr.mkTalk ~tag:"frequence" ()
	val mRatio = Tkr.mkTalk ~tag:"ratio" ()
	val mOutput = Tkr.mkVoice ()
	val mutable mRemainder = Cornet.make 256
	val mutable mRemainderTick = -1
	val mutable mRemainderLen = 0

	method getTalks = [mAmpli; mFreq; mRatio]
	method getKind = kind
	method getVoice name = mOutput
	method getVoices = [|mOutput|]

	method talk port tick len =
		let ar = Listen.talk mAmpli tick len in
		let fr = Listen.talk mFreq tick (Listen.getLength ar) in
		let rr = Listen.talk mRatio tick (Listen.getLength fr) ~copy:false in
		let l = Listen.getLength rr in

		Voice.checkLength mOutput l;

		let ampli = ref 0. and invPos = ref 0 in
		(*trace("tick : "^soi tick^", len : "^soi l);*)
		let nextPeriodBegin = ref 0 in

		if tick = mRemainderTick then (
			if mRemainderLen < l then (
  			Cornet.blit mRemainder 0 (Voice.getCornet mOutput) 0 mRemainderLen;
  			nextPeriodBegin := mRemainderLen;
			)
			else (
  			Cornet.blit mRemainder 0 (Voice.getCornet mOutput) 0 l;
  			nextPeriodBegin := mRemainderLen;
			)
		);

		for i = !nextPeriodBegin to l - 1 do

			if i = !nextPeriodBegin then (
				let f = Listen.(fr@+i) and r = Listen.(rr@+i) in
				let p = SF.fRate /. f in
				invPos := i + iof(p *. (r *. 0.5 +. 0.5));
				ampli := Listen.(ar@+i);
				nextPeriodBegin := i + iof p;
			);
			Voice.set mOutput i (if i < !invPos then !ampli else -. !ampli)
		done;

		mRemainderLen <- !nextPeriodBegin - l;

		if mRemainderLen > 0 then (

			if Cornet.dim mRemainder < mRemainderLen then
				mRemainder <- Cornet.make mRemainderLen;

			for i = 0 to mRemainderLen - 1 do
				Cornet.set mRemainder i (if i + l < !invPos then !ampli else -. !ampli)
			done;
			mRemainderTick <- tick + l;
		);

		Voice.setTick mOutput tick;
		Voice.setLength mOutput l;
end

let make() = (new c)#base
end


module BSquare = struct
	let kind = "bsquare"

class c = object(self) inherit Tkr.c as super

	val mFreq = Tkr.mkTalk ~tag:"frequence"  ~value:440. ()
	val mRatio = Tkr.mkTalk ~tag:"ratio" ~value:0. ()
	val mRoof = Tkr.mkTalk ~tag:"roof" ~value:1. ()
	val mFloor = Tkr.mkTalk ~tag:"floor" ~value:0. ()
	val mOutput = Tkr.mkVoice ()
	val mutable mRoofTick = 0
	val mutable mFloorTick = -1

	method getTalks = [mFreq; mRatio; mRoof; mFloor]
	method getKind = kind
	method getVoice name = mOutput
	method getVoices = [|mOutput|]

	method talk port tick len =

		Voice.checkLength mOutput len;

		let rs = ref(mRoofTick - tick) in
		let fs = ref(mFloorTick - tick) in

		let i = ref 0 in

		while !i < len do

			let t = tick + !i in

			if !i = !rs then (
    		let fr = Listen.talk mFreq t 1 in
    		let rr = Listen.talk mRatio t 1 ~copy:false in

				let f = Listen.(fr @+ 0) and r = Listen.(rr @+ 0) in
				let p = SF.fRate /. f in

				rs := !i + iof p;
				fs := !i + iof(p *. (r *. 0.5 +. 0.5));
			);

			let e = mini len !fs in
			
			if !i < e then (
  			let roof = Listen.talk mRoof t (e - !i) in
				i := !i + Listen.blit roof mOutput !i;
			);

			let e = mini len !rs in

			if !i < e then (
  			let floor = Listen.talk mFloor (tick + !fs) (e - !i) in
				i := !i + Listen.blit floor mOutput !i;
			);
		done;

		mRoofTick <- !rs + tick;
		mFloorTick <- !fs + tick;

		Voice.setTick mOutput tick;
		Voice.setLength mOutput len;
end

let make() = (new c)#base
end


module Triangle = struct
	let kind = "triangle"

class c = object(self) inherit Square.c as super

	val mutable mPrevVal = 0.

	method getKind = kind

	method talk port tick len =
		let ar = Listen.talk mAmpli tick len in
		let fr = Listen.talk mFreq tick (Listen.getLength ar) in
		let rr = Listen.talk mRatio tick (Listen.getLength fr) ~copy:false in
		let l = Listen.getLength rr in

		Voice.checkLength mOutput l;

		let invPos = ref 0 in

		let nextPeriodBegin = ref(if tick = mRemainderTick then (
			Cornet.blit mRemainder 0 (Voice.getCornet mOutput) 0 mRemainderLen;
			mRemainderLen;
		) else 0) in

		let upStep = ref 0. in let downStep = ref 0. in

		for i = !nextPeriodBegin to l - 1 do

			if i = !nextPeriodBegin then (
				let ampli = Listen.(ar @+ i) in
				let freq = Listen.(fr @+ i) in
				let ratio = Listen.(rr @+ i) *. 0.5 +. 0.5 in
				let period = SF.fRate /. freq in
				let upDuration = period *. ratio in
				upStep := (ampli -. mPrevVal) /. upDuration;
				invPos := i + iof upDuration;
(*				
				downStep := (ampli +. Listen.(ar @+ !invPos)) /. (upDuration -. period);
*)
				downStep := (ampli +. Listen.(ar @+ (if !invPos < l then !invPos else l - 1))) /. (upDuration -. period);
				nextPeriodBegin := i + iof period;
			);
			mPrevVal <- mPrevVal +. (if i < !invPos then !upStep else !downStep);
			Voice.set mOutput i mPrevVal;
		done;

		mRemainderLen <- !nextPeriodBegin - l;

		if mRemainderLen > 0 then (

			if Cornet.dim mRemainder < mRemainderLen then
				mRemainder <- Cornet.make mRemainderLen;

			for i = 0 to mRemainderLen - 1 do
				mPrevVal <- mPrevVal +. (if i + l < !invPos then !upStep else !downStep);
				Cornet.set mRemainder i mPrevVal;
			done;
			mRemainderTick <- tick + l;
		);

		Voice.setTick mOutput tick;
		Voice.setLength mOutput l;
end

let make() = (new c)#base
end


module Amplifier = struct
	let kind = "amplifier"

class c = object(self) inherit Tkr.c as super

	val mutable mGain = 1.
	val mInput = Tkr.mkTalk ()
	val mOutput = Tkr.mkVoice ()

	method getGain = mGain
	method setGain g = mGain <- g

	method getValue = Tkr.f2v mGain
	method setValue v = self#setGain (Tkr.v2f v)

	method getTalks = [mInput]
	method getKind = kind
	
	method getVoice name = mOutput
	method getVoices = [|mOutput|]

	method talk port tick len =
		let ir = Listen.talk mInput tick len ~copy:false in
		
		Voice.checkLength mOutput (Listen.getLength ir);

		for i = 0 to (Listen.getLength ir) - 1 do
			Voice.set mOutput i (mGain *. Listen.(ir @+ i))
		done;

		Voice.setTick mOutput tick;
		Voice.setLength mOutput (Listen.getLength ir);
end

let make() = (new c)#base
end


module AmplitudeModulator = struct
	let kind = "amplitudeModulator"

class c = object(self) inherit Tkr.c as super

	val mInput = Tkr.mkTalk ()
	val mGain = Tkr.mkTalk ~tag:"gain" ()
	val mOutput = Tkr.mkVoice ()

	method getTalks = [mInput; mGain]
	method getKind = kind
	method getVoice name = mOutput
	method getVoices = [|mOutput|]

	method talk port tick len =
		let ir = Listen.talk mInput tick len in
		let gr = Listen.talk mGain tick (Listen.getLength ir) ~copy:false in
		let grl = Listen.getLength gr in
		
		Voice.checkLength mOutput grl;

		for i = 0 to grl - 1 do
			Voice.set mOutput i (Listen.(ir @+ i) *. Listen.(gr @+ i))
		done;
		Voice.setTick mOutput tick;
		Voice.setLength mOutput grl;
end

let make() = (new c)#base

end


module FrequencyModulator = struct
	let kind = "frequencyModulator"

class c = object(self) inherit Tkr.c as super

	val mInput = Tkr.mkTalk ()
	val mFreq = Tkr.mkTalk ~tag:"frequence" ()
	val mutable mTickMap = [(0, 0.)]
	val mOutput = Tkr.mkVoice ()

	method getTalks = [mInput; mFreq]
	method getKind = kind
	method getVoice name = mOutput
	method getVoices = [|mOutput|]

	method talk port tick len =

		let (fT, iT) = L.find ~f:(fun (fT, iT) -> tick >= fT) mTickMap in
		let dt = tick - fT in
		let it = ref iT in
		let fr = Listen.talk mFreq fT (dt + len) ~copy:false in
		let frl = Listen.getLength fr in let fro = Listen.getOffset fr in

		if frl <= dt then raise Voice.End;

		let fo = fro + dt in

		for i = fro to fo - 1 do it := !it +. Listen.(fr @. i) +. 1. done;

		let iTick = iof !it and il = ref (!it -. floor !it) in
		
		(* copy of frequence buffer because it can be modified by the input Listen.talk call *)
		let fl = frl - dt in
		let fb = Cornet.sub (Listen.getCornet fr) fo fl in

		for i = 0 to fl - 2 do il := !il +. Cornet.get fb i +. 1. done;

		let iLen = 1 + iof (ceil !il) in

		let ir = Listen.talk mInput iTick iLen ~copy:false in
		let irl = Listen.getLength ir in let iro = Listen.getOffset ir in

		Voice.checkLength mOutput fl;

		let ii = ref (!it -. floor !it) in
		let ol = ref 0 in

		if irl > 1 then (
			let olOfil =
				let nil = foi irl -. 1. in
	
				if irl < iLen / 2 then (
					let ft = ref 0 in
					let ii = ref (!it -. floor !it) in
	
					while !ii <= nil && !ft < fl do
						ii := !ii +. Cornet.get fb !ft +. 1.;
						ft := !ft + 1;
					done;
					!ft
				)
				else (
					let ft = ref (fl - 1) in
	
					while !il > nil && !ft > 0 do
						ft := !ft - 1;
						il := !il -. Cornet.get fb !ft +. 1.;
					done;
					!ft + 1
				);
			in
			ol := if irl < iLen then olOfil else fl;
	
			for oi = 0 to !ol - 1 do
				let iiDec = !ii -. floor !ii in
				let x1 = iro + iof !ii in
				let y1 = Listen.(ir @. x1) in

				if iiDec = 0. then Voice.set mOutput oi y1
				else (
					let y2 = Listen.(ir @. (x1 + 1)) in
					Voice.set mOutput oi (y1 +. (y2 -. y1) *. iiDec)
				);
				ii := !ii +. Cornet.get fb oi +. 1.;
			done;
		)
		else (
			while !ii < 1. && !ol < fl do
				Voice.set mOutput !ol Listen.(ir @. 0);
				ii := !ii +. Cornet.get fb !ol +. 1.;
				ol := !ol + 1;
			done;
		);

		mTickMap <- (tick + !ol, foi iTick +. !ii)::mTickMap;
		Voice.setTick mOutput tick;
		Voice.setLength mOutput !ol;

end

let make() = (new c)#base
end


module DynamicModulator = struct
	let kind = "dynamicModulator"

class c = object(self) inherit Tkr.c as super

	val mInput = Tkr.mkTalk ()
	val mGain = Tkr.mkTalk ~tag:"gain" ()
	val mOutput = Tkr.mkVoice ()
	val mutable mPrevVal = 0.

	method getTalks = [mInput; mGain]
	method getKind = kind
	method getVoice name = mOutput
	method getVoices = [|mOutput|]

	method talk port tick len =
		let ir = Listen.talk mInput tick len in
		let gr = Listen.talk mGain tick (Listen.getLength ir) ~copy:false in
		let grl = Listen.getLength gr in

		Voice.checkLength mOutput grl;

		for i = 0 to grl - 1 do
			let v = Listen.(ir @+ i) and g = Listen.(gr @+ i) in

			mPrevVal <- mPrevVal +. ((v -. mPrevVal) *. (g +. 1.));

			Voice.set mOutput i mPrevVal
		done;

		Voice.setTick mOutput tick;
		Voice.setLength mOutput grl;
end

let make() = (new c)#base
end


module Damper = struct
	let kind = "damper"

class c = object(self) inherit Tkr.c as super

	val mInput = Tkr.mkTalk ()
	val mCeiling = Tkr.mkTalk ~tag:"ceiling" ()
	val mGain = Tkr.mkTalk ~tag:"gain" ()
	val mOutput = Tkr.mkVoice ()
	val mutable mPrevVal = 0.

	method getTalks = [mInput; mCeiling; mGain]
	method getKind = kind
	method getVoices = [|mOutput|]

	method talk port tick len =
		let ir = Listen.talk mInput tick len in
		let cr = Listen.talk mCeiling tick (Listen.getLength ir) in
		let gr = Listen.talk mGain tick (Listen.getLength cr) ~copy:false in
		let grl = Listen.getLength gr in

		Voice.checkLength mOutput grl;

		for i = 0 to grl - 1 do
			let v = Listen.(ir @+ i) and c = Listen.(cr @+ i) and g = Listen.(gr @+ i) in

			mPrevVal <- if v > 0. then
				mPrevVal +. minf(v -. mPrevVal)((c -. mPrevVal) *. g)
			else mPrevVal +. maxf(v -. mPrevVal)((-.c -. mPrevVal) *. g);
			
			Voice.set mOutput i mPrevVal
		done;

		Voice.setTick mOutput tick;
		Voice.setLength mOutput grl;
end

let make() = (new c)#base
end


module Accumulator = struct
	let kind = "accumulator"

class c = object(self) inherit Tkr.c as super

	val mInput = Tkr.mkTalk ()
	val mIntegral = Tkr.mkTalk ~tag:"integral" ()
	val mDamper = Tkr.mkTalk ~tag:"damper" ()
	val mOutput = Tkr.mkVoice ()

	val mutable mPrevError = 0.
	val mutable mMidError = 0.
	val mutable mPrevOutput = 0.
	val mutable mIntegVal = 0.

	method getTalks = [mInput; mIntegral; mDamper]
	method getKind = kind
	method getVoices = [|mOutput|]

	method talk port tick len =
		let inr = Listen.talk mInput tick len in
		let ir = Listen.talk mIntegral tick (Listen.getLength inr) in
		let dr = Listen.talk mDamper tick (Listen.getLength ir) ~copy:false in
		let drl = Listen.getLength dr in

		Voice.checkLength mOutput drl;

		for i = 0 to drl - 1 do
			let v = Listen.(inr@+i) and ik = Listen.(ir@+i) and dk = Listen.(dr@+i) in
			let e = v -. mPrevOutput in

			if (e > 0. && e > mPrevError) || (e < 0. && e < mPrevError) then (
				mMidError <- e *. 0.5;
				mPrevError <- e;
			) else if e = 0. then (
				mMidError <- 0.;
				mPrevError <- 0.;
			);

			mIntegVal <- mIntegVal +. (e -. (mMidError *. dk)) *. ik;

			mPrevOutput <- mPrevOutput +. mIntegVal;
			
			Voice.set mOutput i mPrevOutput;
		done;

		Voice.setTick mOutput tick;
		Voice.setLength mOutput drl;
end

let make() = (new c)#base
end



module Regulator = struct
	let kind = "regulator"

class c = object(self) inherit Tkr.c as super
	
	val mInput = Tkr.mkTalk ()
	val mProportional = Tkr.mkTalk ~tag:"proportional" ()
	val mIntegral = Tkr.mkTalk ~tag:"integral" ()
	val mDerivative = Tkr.mkTalk ~tag:"derivative" ()
	val mOutput = Tkr.mkVoice ()
	
	val mutable mPrevError = 0.
	val mutable mPrevOutput = 0.
	val mutable mIntegVal = 0.

	method getTalks = [mInput; mProportional; mIntegral; mDerivative]
	method getKind = kind
	method getVoices = [|mOutput|]

	method talk port tick len =
		let inr = Listen.talk mInput tick len in
		let pr = Listen.talk mProportional tick (Listen.getLength inr) in
		let ir = Listen.talk mIntegral tick (Listen.getLength pr) in
		let dr = Listen.talk mDerivative tick (Listen.getLength ir) ~copy:false in
		let drl = Listen.getLength dr in
		
		Voice.checkLength mOutput drl;

		for i = 0 to drl - 1 do
			let v = Listen.(inr @+ i) in
			let pk = Listen.(pr @+ i) in
			let ik = Listen.(ir @+ i) in
			let dk = Listen.(dr @+ i) in
			
			let e = v -. mPrevOutput in
			let pv = e *. pk in
			mIntegVal <- mIntegVal +. e *. ik;
			let dv = (e -. mPrevError) *. dk in

			mPrevOutput <- mPrevOutput +. pv +. mIntegVal +. dv;
			
			Voice.set mOutput i mPrevOutput;
			mPrevError <- e;
		done;

		Voice.setTick mOutput tick;
		Voice.setLength mOutput drl;
end

let make() = (new c)#base
let handler = Plugin.{kind; category = "Shaper"; make = fun() -> new c}
end


let register =
	Factory.addTalkerMaker Constant.kind "Mathematics" Constant.make;
	Factory.addTalkerMaker Sum.kind "Mathematics" Sum.make;
	Factory.addTalkerMaker Product.kind "Mathematics" Product.make;
	Factory.addTalkerMaker Average.kind "Mathematics" Average.make;
	Factory.addTalkerMaker StaticSine.kind "Oscillator" StaticSine.make;
	Factory.addTalkerMaker Sine.kind "Oscillator" Sine.make;
	Factory.addTalkerMaker AbsSine.kind "Oscillator" AbsSine.make;
	Factory.addTalkerMaker BSine.kind "Oscillator" BSine.make;
	Factory.addTalkerMaker AbsBSine.kind "Oscillator" AbsBSine.make;
	Factory.addTalkerMaker Square.kind "Oscillator" Square.make;
	Factory.addTalkerMaker BSquare.kind "Oscillator" BSquare.make;
	Factory.addTalkerMaker Triangle.kind "Oscillator" Triangle.make;
	Factory.addTalkerMaker Amplifier.kind "Modulator" Amplifier.make;
	Factory.addTalkerMaker AmplitudeModulator.kind  "Modulator"AmplitudeModulator.make;
	Factory.addTalkerMaker FrequencyModulator.kind "Modulator" FrequencyModulator.make;
	Factory.addTalkerMaker DynamicModulator.kind "Modulator" DynamicModulator.make;
	Factory.addTalkerMaker Damper.kind "Shaper" Damper.make;
	Factory.addTalkerMaker Accumulator.kind "Shaper" Accumulator.make;
	Factory.addTalkerMaker Regulator.kind "Shaper" Regulator.make;


 