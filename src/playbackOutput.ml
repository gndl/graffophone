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

open Bigarray

open FFmpeg

open Graffophone_plugin
open Usual

module SF = SampleFormat
open Output

module Bus = EventBus
module Converter = Swresample.Make (Swresample.PlanarFloatArray) (Swresample.Frame)


class c ?(name = "Playback Output(") () =
  object(self) inherit Output.c name

    val mutable mOutputDevice = ""
    val mutable mNewOutputDevice = ""
    val mutable mCtx = None
    val mutable mChannelsCount = 2
    val mutable mOutputBuffer = Array1.create float32 c_layout (SF.chunkSize * 2)

    initializer
      mOutputDevice <- Device.getOutput();
      mNewOutputDevice <- mOutputDevice;


    method openOutput nbChannels =

      let rec makeOutputStream device =
        try
          let fmt = Device.getOutputIdFormat device in
          let codec_id = Av.Format.get_audio_codec_id fmt in

          let cl = Avutil.Channel_layout.get_default nbChannels in
          let out_sample_format = Avcodec.Audio.find_best_sample_format codec_id `Dbl in

          let output = Av.open_output_format fmt in

          let conv = Converter.create cl SF.rate cl ~out_sample_format SF.rate in

          if device <> mOutputDevice || device <> mNewOutputDevice then (
            mOutputDevice <- device;
            mNewOutputDevice <- device;
          );

          mCtx <- Some(conv, output);
        with Avutil.Error e -> (
            Bus.Error(Avutil.string_of_error e) |> Bus.asyncNotify;

            (* If the new device raise an error, we fallback to the previous device *)
            if device <> mOutputDevice then
              makeOutputStream mOutputDevice
            else
              raise (Avutil.Error e)
          )
      in
      makeOutputStream mNewOutputDevice;


    method write lg channels =
      if mCtx = None then self#openOutput (A.length channels);

      match mCtx with
      | None -> ()
      | Some(conv, output) ->
        let planes = if lg = A.length channels.(0) then channels
          else A.map ~f:(fun plane -> A.sub plane ~pos:0 ~len:lg) channels in
        try
          Converter.convert conv planes |> Av.write_audio_frame output;
        with Avutil.Error e -> Bus.Error(Avutil.string_of_error e) |> Bus.asyncNotify;


    method closeOutput =
      match mCtx with
      | None -> trace "No output to close"
      | Some(_, output) -> Av.close output


    method backup = (Output.kind, "playback", [])

  end

let make name _ = toO(new c ~name ())

let handler = {feature = "playback"; make}

