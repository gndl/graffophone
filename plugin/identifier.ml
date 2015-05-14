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

let incCount count = count := !count + 1; !count


class virtual c ?name ?kind count =
	object
	val mId = incCount count
	val mutable mName = ""

	initializer
		match name with
		| Some n ->
			if n.[String.length n - 1] = '(' then (
				match kind with
				| Some k -> mName <- n ^ k ^ " " ^ string_of_int mId ^ ")"
				| None -> mName <- n ^ string_of_int mId ^ ")"
			)
			else if n.[String.length n - 1] = '[' then (
				match kind with
				| Some k -> mName <- n ^ k ^ " " ^ string_of_int mId ^ "]"
				| None -> mName <- n ^ string_of_int mId ^ "]"
			)
			else mName <- n
		| None -> match kind with
			| Some k -> mName <- k ^ " " ^ string_of_int mId
			| None -> () (*mName <- "[" ^ string_of_int mId ^ "]"*)

	method getId = mId
	method getName = mName
	method setName name = mName <- name
	method dependsOf id = mId = id
end	

let toId st = (st :> c)
