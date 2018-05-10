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

open FFmpeg
open Avutil

open Graffophone_plugin
open Util
open Usual
open Factory
open SampleFormat

module Bus = EventBus
module Tkr = Talker
module Converter = Swresample.Make (Swresample.Frame) (Swresample.DblPlanarBigArray)


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
      let file = Av.open_input fileName in
      let _, stream, codec = Av.find_best_audio_stream file in
      let nbChs = Avcodec.Audio.get_nb_channels codec in
      let cl = Avcodec.Audio.get_channel_layout codec in
      let dur = Av.get_duration ~format:`Microsecond stream in
      let len = Int64.(to_int(div (mul dur (of_int sf.rate)) (of_int 1000000))) in

      let conv_ctx = Converter.from_codec ~options:[`Engine_soxr] codec cl sf.rate in

      let mkCh p = Tkr.mkVoice ~tag:("Channel_" ^ soi(p + 1))
          ~port:p ~len ~talker:self#base ()
      in
      mChannels <- A.init nbChs mkCh;
      let pos = ref 0 in

      stream |> Av.iter_frame (fun frame ->
          Converter.convert conv_ctx frame
          |> A.iteri ~f:(fun nc plane ->
              let len = Cornet.dim plane in
              Cornet.blit plane 0 (Voice.getCornet(mChannels.(nc))) !pos len;
              pos := !pos + len;
            )
        );

      Av.close file;
      mFilename <- fileName;
    with Avutil.Failure msg -> Bus.notify(Bus.Error(fileName ^ " : " ^ msg));


  method getKind = kind
  method getVoices = mChannels

  method talk port tick len = ()

end

let handler = Plugin.{kind; category = "Input"; make = fun() -> (new c)#base}
