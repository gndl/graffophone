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

type t = (float, Bigarray.float32_elt, Bigarray.c_layout) Bigarray.Array1.t

let get (cor:t) i = Bigarray.Array1.unsafe_get cor i
let set (cor:t) i value = Bigarray.Array1.unsafe_set cor i value

let make len = Bigarray.Array1.create Bigarray.float32 Bigarray.c_layout len

let init ?(v = 0.) ?f len =

  let cor = make len in

  match f with
  | Some f -> for i = 0 to len - 1 do set cor i (f i) done; cor
  | None -> Bigarray.Array1.fill cor v; cor


let sub (cor:t) ofs len = Bigarray.Array1.sub cor ofs len
let fill (cor:t) ofs len value = Bigarray.Array1.fill(sub cor ofs len)value
let dim (cor:t) = Bigarray.Array1.dim cor

let blit (src:t) srcOfs (dest:t) destOfs len =
  let s = sub src srcOfs len in
  let d = sub dest destOfs len in
  Bigarray.Array1.blit s d


