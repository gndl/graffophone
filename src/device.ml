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

open Usual

module Bus = EventBus

let output = ref ""


let getOutput() = !output

(*
let getName device = Portaudio.((get_device_info device).d_name)
let getMaxOutputChannels device = Portaudio.((get_device_info device).d_max_output_channels)
*)

let getOutputsNames() =
  Avdevice.get_audio_output_formats() |> List.map Av.Format.get_output_long_name


let getOutputIdFormat id =
  Avdevice.get_audio_output_formats()
  |> List.find(fun fmt -> Av.Format.get_output_name fmt = id)


let getOutputNameFormat name =
  Avdevice.get_audio_output_formats()
  |> List.find(fun fmt -> Av.Format.get_output_long_name fmt = name)


let changeOutput newOutputName =
  output := getOutputNameFormat newOutputName |> Av.Format.get_output_name


let initialize() =
  let outputName = Configuration.getOutputDeviceName in

  if L.mem outputName ~set:(getOutputsNames()) then (
    changeOutput outputName
  )
  else (
    let dev = Avdevice.get_default_audio_output_format() in
    output := Av.Format.get_output_name dev;
    Configuration.setOutputDeviceName(Av.Format.get_output_long_name dev);
  );
  trace("outputDeviceName : " ^ !output)


let terminate() = ()

