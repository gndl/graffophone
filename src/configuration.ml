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
open Sexplib.Std
       
type t = {
  mutable playbackVolume : float [@default 50.];(* Playback volume. Its default value is 50% *)

  mutable playbackOutputDeviceName : string [@default "ALSA audio output"];(* Playback output device name. Its default value is empty *)

  mutable libraryFiles : string list;  (* Audio files and folders of the library *)
} [@@deriving sexp]

let configFileName = (Sys.getenv "HOME")^"/.graffophone.conf"


(* Loading of the configuration file *)

let log_file = open_out "graffophone.log"

let default = {
  playbackVolume = 50.;
  playbackOutputDeviceName =  "ALSA audio output";
  libraryFiles = []
}


let save config = sexp_of_t config |> Sexplib.Sexp.save_hum configFileName

let config = match Sexplib.Sexp.load_sexp_conv configFileName t_of_sexp with
  | `Result c -> c
  | `Error _ | exception _ -> save default; default

let save() = save config

let getVolume = config.playbackVolume
let setVolume v = config.playbackVolume <- v

let getOutputDeviceName = config.playbackOutputDeviceName
let setOutputDeviceName v = config.playbackOutputDeviceName <- v


let addFiles filenameList = config.libraryFiles <- config.libraryFiles @ filenameList
let removeFile filename = config.libraryFiles <- L.filter ~f:(fun fn -> fn <> filename) config.libraryFiles
let getFiles = config.libraryFiles
