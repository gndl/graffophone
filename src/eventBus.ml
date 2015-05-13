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

type notification =
	| State of State.t
	| Session
	| Tick of int
	| TimeRange of int * int
	| Pause
	| End
	| Volume of float
	| TalkersRange of (string * string list) list
	| NewTalker
	| TalkerChanged
	| TalkerSelected of int
	| TalkerUnselected of int
	| EarSelected of int * int
	| EarUnselected of int * int
	| VoiceSelected of int * int
	| VoiceUnselected of int * int
	| TalkSelected of int * int
	| CurveAdded
	| CurveRemoved
	| Info of string
	| Warning of string
	| Error of string


let observers : (notification -> unit) list ref = ref []

let addObserver o = observers := o :: !observers

let notify notification = List.iter(fun observe -> observe notification) !observers
(*
class c =
object (self)
	val mutable mObservers : (notification -> unit) list = []
	
	method addObserver o = mObservers <- o::mObservers
	
	method notify notification = List.iter(fun observe -> observe notification) mObservers
end


type operation =
	| Synchronize of (unit -> unit)
	| SetStartTick of int
	| SetEndTick of int
	| CurveZoom of float

let operators : (operation -> unit) list ref = ref []

let addOperator o = operators := o :: !operators

let request operation = List.iter(fun operate -> operate operation) !operators

*)