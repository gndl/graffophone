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

open Graffophone_plugin
open Util
open Usual
open Identifier
open SampleFormat
open Output

module Converter = Swresample.Make (Swresample.PlanarFloatArray) (Swresample.Frame)

class c ?(filename="") ?(name = "Sound File Output (") () =
  object(self) inherit Output.c name
    val mutable mFilename = filename
    val mutable mCtx = None
    val mutable mNbChannels = sf.channels
    val mutable mBuf = A.make (chunkSize * sf.channels) 0.0
    val mutable mCodecId = `Flac
    val mutable mChannelLayout = `Stereo
    val mutable mSampleRate = sf.rate


    method openOutput nbChannels =
      mNbChannels <- nbChannels;
      try
        let cl = Avutil.Channel_layout.get_default nbChannels in
        let sample_format = Avcodec.Audio.find_best_sample_format mCodecId `Dbl in

        let stream = Av.open_output mFilename
                     |> Av.new_audio_stream ~codec_id:mCodecId
                       ~channel_layout:mChannelLayout ~sample_format
                       ~sample_rate:mSampleRate in

        let conv = Converter.to_codec cl sf.rate (Av.get_codec stream) in

        mCtx <- Some(conv, stream);
      with Failure msg -> trace ("Failed to open file "^mFilename^" : "^msg)


    method write lg channels =

      if mCtx = None then self#openOutput (A.length channels);

      match mCtx with
      | None -> ()
      | Some(conv, stream) ->
        let planes = if lg = A.length channels.(0) then channels
          else A.map ~f:(fun plane -> A.sub plane 0 lg) channels in

        Converter.convert conv planes |> Av.write_frame stream;


    method closeOutput =
      match mCtx with
      | None -> trace "No file to close"
      | Some(_, stream) -> Av.get_output stream |> Av.close;


    method backup = (Output.kind, "file", [("filename", mFilename)])
  end

let make name attributs =
  let filename = try
      let (t, d, _) = List.find (fun (t, d, _) -> t = "filename") attributs in d
    with Not_found -> "" in
  toO(new c ~filename ~name ())

let handler = {feature = "file"; make}
