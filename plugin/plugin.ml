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

type talkerHandler = {
	kind : string;
	category : string; 
	make : (unit -> Talker.c)
}

type handler = {
	name : string;
	talkerHandlers : talkerHandler list
}

let null = {name = "nullnull"; talkerHandlers = []}

let handler = ref null

let isRegistered() = !handler != null

let getHandler() = !handler

let reset() = handler := null


let defaultProvideHandler() =
	print_string ("Plugin did not register\n");
	flush stdout

let provideHandler = ref defaultProvideHandler


