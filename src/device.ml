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

open Usual

module Bus = EventBus

let errorCodeOutputUnderflowed = -9980

let output = ref 0


let getOutput() = !output


let getName device = Portaudio.((get_device_info device).d_name)
let getMaxOutputChannels device = Portaudio.((get_device_info device).d_max_output_channels)


let getOutputsNames() =
		let open Portaudio in
    let dcount = Portaudio.get_device_count () in

		let rec search id lst =
			if id < dcount then (
				
        let dinfo = Portaudio.get_device_info id in
				
				if dinfo.d_max_output_channels > 0 then
					search(id + 1) (dinfo.d_name::lst) 
				else
					search(id + 1) lst
			)
			else lst
		in
		L.rev(search 0 [])


let changeOutput newOutputName =
		let open Portaudio in
    let dcount = Portaudio.get_device_count () in

		let rec search id =
			if id < dcount then (
		    let dinfo = Portaudio.get_device_info id in
				
				if dinfo.d_name = newOutputName then id else search(id + 1)
			)
			else -1
		in
		
		let newOutput = search 0 in
		
		if newOutput >= 0 && newOutput <> !output then (
			output := newOutput;
		)


let initialize() =
	Portaudio.init ();
	try
		let outputName = Configuration.getOutputDeviceName
		in
		trace("outputDeviceName : "^outputName);
		
		if L.mem outputName ~set:(getOutputsNames()) then (
			changeOutput outputName
		)
		else (
			output := Portaudio.get_default_output_device();
			Configuration.setOutputDeviceName(getName !output)
		)

	with Portaudio.Error code -> (
		Bus.notify(Bus.Error(Portaudio.string_of_error code));
	)


let terminate() = Portaudio.terminate ()

