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

open Graffophone_plugin
open Usual
open Bigarray
module SF = SampleFormat
open Output

module Bus = EventBus


class c ?(name = "Playback Output(") () =
  object(self) inherit Output.c name

    val mutable mOutputDevice = 0
    val mutable mNewOutputDevice = 0
    val mutable mStream = None
    val mutable mChannelsCount = 2
    val mutable mOutputBuffer = Array1.create float32 c_layout (SF.chunkSize * 2)

    initializer
      mOutputDevice <- Device.getOutput();
      mNewOutputDevice <- mOutputDevice;


    method openOutput nbChannels =


      let rec makeOutputStream device =
        try
          let rate = foi(SF.rate) in
          let bufframes = SF.chunkSize in
          let channels = mini nbChannels (Device.getMaxOutputChannels device) in

          if channels <> mChannelsCount then (
            mChannelsCount <- channels;
            mOutputBuffer <- Array1.create float32 c_layout (SF.chunkSize * channels)
          );

          let outparam = Portaudio.{channels; device;
                                    sample_format = format_float32; latency = 1.
                                   }
          in
          trace("makeOutputStream : channels = "^soi channels^", rate = "^sof rate^", bufframes = "^soi bufframes);
          let stream = Portaudio.open_stream None (Some outparam) ~interleaved:true rate bufframes []
          in
          Portaudio.start_stream stream;

          if device <> mOutputDevice || device <> mNewOutputDevice then (
            mOutputDevice <- device;
            mNewOutputDevice <- device;
          );

          mStream <- Some stream

        with Portaudio.Error code -> (
            Bus.asyncNotify(Bus.Error(Portaudio.string_of_error code));

            (* If the new device raise an error, we fallback to the previous device *)
            if device <> mOutputDevice then
              makeOutputStream mOutputDevice
            else
              raise (Portaudio.Error code)
          )
      in
      makeOutputStream mNewOutputDevice;


    method write lg channels =
      let wrt stream =

        for numChan = 0 to mChannelsCount - 1 do

          let chan = channels.(numChan) in

          for i = 0 to lg - 1 do
            let outIdx = (i * mChannelsCount) + numChan in
            mOutputBuffer.{outIdx} <- chan.(i);
          done;
        done;

        let genOutBuf = genarray_of_array1 mOutputBuffer
        in
        try
          Portaudio.write_stream_ba stream genOutBuf 0 lg;

        with Portaudio.Error code -> (
            let msg = "Portaudio.write_stream_ba Error code "^soi code^" : "^Portaudio.string_of_error code
            in
            if code = Device.errorCodeOutputUnderflowed then (
              traceYellow msg;
            )
            else (
              Bus.asyncNotify(Bus.Error msg);
            )
          )
      in
      match mStream with
      | Some stream -> wrt stream
      | None -> (self#openOutput (Array.length channels); match mStream with
        | Some stream -> wrt stream
        | None -> ())


    method closeOutput =
      match mStream with
      | None -> trace "No stream to close"
      | Some stream -> Portaudio.close_stream stream


    method backup = (Output.kind, "playback", [])


  end

let make name _ = toO(new c ~name ())

let handler = {feature = "playback"; make}

