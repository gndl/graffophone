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

let sOutputCount = ref 0

let kind = "output"

class virtual c name = object inherit
  Identifier.c ~name ~kind sOutputCount

  method virtual openOutput : int -> unit(* -> unit*)
  (*method virtual voice : time   buffer        length  outlength *)  
  (*method virtual write : int -> float array -> int -> float array array -> int*)
  method virtual write : int -> float array array -> unit
  method virtual closeOutput : unit
  (*                           kind     value     tag      dep *)
  method virtual backup : (string * string * (string * string) list)
end

let toO st = (st :> c)

type handler = {
  feature : string;
  make : (string -> (string * string * string) list -> c)
}

