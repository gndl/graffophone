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


let pi = 4.0 *. atan 1.0
let pi2 = 2. *. pi

module L = ListLabels

module A = struct
	include ArrayLabels
	
	let add a e = append a [|e|]
	
	let sup a i =
		let newLen = length a - 1 in
		
		if i = 0 then sub a 1 newLen
		else if i = newLen then sub a 0 newLen
		else append(sub a 0 i) (sub a (i + 1) (newLen - i))
		
end
		
