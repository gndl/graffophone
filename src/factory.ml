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
open SampleFormat

module Tkr = Talker

let talkerHandlers : (string * Plugin.talkerHandler) list ref = ref []

let addTalkerHandler th = Plugin.(
    talkerHandlers := (th.kind, th) :: !talkerHandlers;
    traceGreen("Talker "^th.kind^" ("^th.category^") registered")
  )

let getTalkerHandler kind =
  try L.assoc kind !talkerHandlers
  with Not_found -> (trace(kind^"'s factory not found!"); raise Not_found)


let makeTalker ?name kind =
  let th = getTalkerHandler kind in

  let talker = Tkr.mkTkr(Plugin.(th.make ())) in

  match name with None -> talker
                | Some nm -> talker#setName nm; talker


let getTalkersInfos() =
  L.map !talkerHandlers ~f:(fun (kind, th) -> Plugin.(kind, th.category))


let outputMakers:(string * Output.handler)list ref = ref []

let addOutputMaker handler =
  outputMakers := Output.(handler.feature, handler) :: !outputMakers

let getOutputMaker feature = Output.((L.assoc feature !outputMakers).make)

let makeOutput name feature attributs =
  (getOutputMaker feature) name attributs

