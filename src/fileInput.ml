(* 
 * Copyright (C) 2015 Ga�tan Dubreil
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

open Graffophone_plugin
open Usual
open SampleFormat

module Bus = EventBus
module Tkr = Talker
module Converter = Swresample.Make (Swresample.Frame) (Swresample.FltPlanarBigArray)


let kind = "fileInput"

class c = object(self) inherit Tkr.c

  val mutable mFilename = "Click here to select a file"
  val mutable mChannels = [||]

  method! getValue = Tkr.fl2v mFilename
  method! setValue value = self#setFilename(Tkr.v2fl value)

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
      mChannels <- A.init nbChs ~f:mkCh;
      let pos = ref 0 in

      stream |> Av.iter_frame (fun frame ->
          let planes = Converter.convert conv_ctx frame in

          for nc = 0 to nbChs - 1 do
            let plane = planes.(nc) in
            let len = Cornet.dim plane in
            Cornet.blit plane 0 (Voice.getCornet(mChannels.(nc))) !pos len
          done;

          if A.length planes > 0 then pos := !pos + (Cornet.dim planes.(0));
        );

      Av.close file;
      mFilename <- fileName;
      self#setName(Filename.basename fileName);
      trace fileName;
    with Avutil.Error e -> Bus.notify(Bus.Error(fileName ^ " : " ^ Avutil.string_of_error e));


  method getKind = kind
  method! getVoices = mChannels

  method! talk _ _ _ = ()

end

let handler = Plugin.{kind; category = "Input"; make = fun() -> (new c)#base}
