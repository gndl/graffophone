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


module L = ListLabels

module A = struct
  include ArrayLabels

  let add a e = append a [|e|]

  let sup a i =
    let len = length a - 1 in

    if i = 0 then sub a ~pos:1 ~len
    else if i = len then sub a ~pos:0 ~len
    else append(sub a ~pos:0 ~len:i) (sub a ~pos:(i + 1) ~len:(len - i))

end


let pi = 4.0 *. atan 1.0
let pi2 = 2. *. pi

let mini (x:int) (y:int) = if x < y then x else y
let maxi (x:int) (y:int) = if x > y then x else y
let minf (x:float) (y:float) = if x < y then x else y
let maxf (x:float) (y:float) = if x > y then x else y
