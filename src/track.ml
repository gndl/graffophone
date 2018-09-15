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
open Usual

module Tkr = Talker 

let sTrackCount = ref 0

let kind = "track"

class c = object(self) inherit Tkr.c

  val mInput = Tkr.mkTalk ()
  val mGain = Tkr.mkBin ~tag:"gain" ~value:1. ()
  val mChannelsGains = Tkr.mkBins ~tag:"channelGain" ()

  val mutable mBuf = A.make 256 1.

  method getKind = kind

  method put add tick buf len channels =

    let computeGain = function
      | Ear.Word gainWord -> (
          let ir = Listen.talk mInput tick len ~copy:false in
          let irl = Listen.getLength ir in
          let gain = gainWord.Ear.value in

          for i = 0 to irl - 1 do
            buf.(i) <- Listen.(ir @+ i) *. gain
          done;
          irl)
      | Ear.Talk gainTalk -> (
          let ir = Listen.talk mInput tick len in
          let gr = Listen.talk gainTalk tick (Listen.getLength ir) ~copy:false in
          let grl = Listen.getLength gr in

          for i = 0 to grl - 1 do
            buf.(i) <- Listen.(ir @+ i) *. Listen.(gr @+ i)
          done;
          grl)
    in
    let ol = ref(computeGain mGain.Ear.src) in
    let n = mini (A.length channels) (A.length mChannelsGains.Ear.bins) in

    if add then (
      for i = 0 to n - 1 do
        let ch = channels.(i) in
        match mChannelsGains.Ear.bins.(i).Ear.src with
        | Ear.Word word -> let cg = word.Ear.value in
          if cg <> 0. then (
            for j = 0 to !ol - 1 do
              ch.(j) <- ch.(j) +. cg *. buf.(j) done;)
        | Ear.Talk cge -> (
            let cgr = Listen.talk cge tick !ol in
            let cgrl = Listen.getLength cgr in

            for j = 0 to cgrl - 1 do
              ch.(j) <- ch.(j) +. Listen.(cgr @+ j) *. buf.(j) done;
            ol := cgrl;)
      done;

      for i = n to A.length channels - 1 do
        let ch = channels.(i) in
        for j = 0 to !ol - 1 do
          ch.(j) <- ch.(j) +. buf.(j) done;
      done;
    )
    else (
      for i = 0 to n - 1 do
        let ch = channels.(i) in
        match mChannelsGains.Ear.bins.(i).Ear.src with
        | Ear.Word word -> let cg = word.Ear.value in
          if cg <> 0. then (
            for j = 0 to !ol - 1 do
              ch.(j) <- cg *. buf.(j) done;
          ) else A.fill ch ~pos:0 ~len:!ol 0.;
        | Ear.Talk cge -> (
            let cgr = Listen.talk cge tick !ol in
            let cgrl = Listen.getLength cgr in

            for j = 0 to cgrl - 1 do
              ch.(j) <- Listen.(cgr @+ j) *. buf.(j) done;
            ol := cgrl;)
      done;

      for i = n to A.length channels - 1 do
        let ch = channels.(i) in
        A.blit ~src:buf ~src_pos:0 ~dst:ch ~dst_pos:0 ~len:!ol;
      done;
    );
    !ol


  method! getEars = Ear.([|ETalk mInput; EBin mGain; EBins mChannelsGains|])

  method! backup = (kind, "", self#getEars)
end

let make() = new c
