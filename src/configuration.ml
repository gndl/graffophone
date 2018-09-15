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
open Config_file

let group = new group
let configFileName = (Sys.getenv "HOME")^"/.graffophone.conf"

(* Playback volume. Its default value is 50% *)
let volume = new float_cp ~group ["playback"; "volume"] 50. "Playback volume (%)"

(* Playback output device name. Its default value is empty *)
let outputDeviceName = new string_cp ~group ["playback"; "outputDeviceName"] "" "Playback output device"

let files = new list_cp string_wrappers ~group ["library"; "files"] [] "Audio files and folders of the library"


(* Loading of the configuration file *)

let log_file = open_out "graffophone.log";;
group#read
  ~on_type_error:
    (fun groupable_cp _ output filename _ ->
       Printf.fprintf log_file
         "Type error while loading configuration parameter %s from file %s.\n%!"
         (S.concat "." groupable_cp#get_name) filename;
       output log_file; (* get more information into log_file *)
    )
  configFileName


let getVolume = volume#get
let setVolume v = volume#set v

let getOutputDeviceName = outputDeviceName#get
let setOutputDeviceName v = outputDeviceName#set v


let addFiles filenameList = files#set(files#get @ filenameList)
let removeFile filename = files#set(L.filter ~f:(fun fn -> fn <> filename) files#get)
let getFiles = files#get

let save() = group#write configFileName
