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

class c initMinValue initValue initMaxValue callback = object (self)
  inherit GraffophoneGui.windowBoundedFloatEntryDialog() as super

  val mutable mInitValue = initValue
  val mutable mMinValue = 0.
  val mutable mMaxValue = 1.

  initializer
    spinbuttonBoundedFloatEntry#set_adjustment vscaleBoundedFloatEntry#adjustment;

    ignore(spinbuttonBoundedFloatEntry#event#connect#key_release ~callback:self#keyPressedOnValue);
    ignore(spinbuttonBoundedFloatEntryMin#event#connect#key_release ~callback:self#keyPressedOnBoundValue);
    ignore(spinbuttonBoundedFloatEntryMax#event#connect#key_release ~callback:self#keyPressedOnBoundValue);

    ignore(vscaleBoundedFloatEntry#adjustment#connect#value_changed (self#vscaleValueChanged true));

    ignore(toplevel#event#connect#leave_notify ~callback:self#leave);

    self#setAdjustment initMinValue initValue initMaxValue;

    (* in order to remind the c float 32 init value *)
    (* mInitValue <- vscaleBoundedFloatEntry#adjustment#value; *)


  method vscaleValueChanged onTheFly () = trace "> DialogBoundedFloatEntry.c#vscaleValueChanged";
    trace(
      "min value = "^sof vscaleBoundedFloatEntry#adjustment#lower ^
      "\nvalue = "^sof vscaleBoundedFloatEntry#adjustment#value ^
      "\nmax value = "^sof vscaleBoundedFloatEntry#adjustment#upper ^
      if onTheFly then "\n on the fly" else ""
    );
    callback
      vscaleBoundedFloatEntry#adjustment#lower
      vscaleBoundedFloatEntry#adjustment#value
      vscaleBoundedFloatEntry#adjustment#upper
      onTheFly;
    trace "< DialogBoundedFloatEntry.c#vscaleValueChanged";


  method setAdjustment minValue value maxValue = trace("> DialogBoundedFloatEntry.c#setAdjustment "^sof value);
    let lower, upper =
      if minValue < maxValue then (minValue, maxValue)
      else if value = 0. then (0., 1.)
      else if value > 0. then (0., 2. *. value)
      else (2. *. value, 0.)
    in

    let range = upper -. lower in
    let step_incr = range /. 100. in
    let page_incr = range /. 10. in

    vscaleBoundedFloatEntry#adjustment#set_bounds
      ~lower ~upper ~step_incr ~page_incr ~page_size:0. ();

    vscaleBoundedFloatEntry#adjustment#set_value value;

    spinbuttonBoundedFloatEntryMin#adjustment#set_bounds
      ~lower:(-100000.) ~upper:100000. ~step_incr ~page_incr ~page_size:0. ();
    spinbuttonBoundedFloatEntryMin#set_value lower;

    spinbuttonBoundedFloatEntryMax#adjustment#set_bounds
      ~lower:(-100000.) ~upper:100000. ~step_incr ~page_incr ~page_size:0. ();
    spinbuttonBoundedFloatEntryMax#set_value upper;

    trace("< DialogBoundedFloatEntry.c#setAdjustment "^sof value);


  method keyPressedOnBoundValue(ev:GdkEvent.Key.t) =

    if GdkEvent.Key.keyval ev = GdkKeysyms._Return then (
      self#setAdjustment
        spinbuttonBoundedFloatEntryMin#value
        spinbuttonBoundedFloatEntry#value
        spinbuttonBoundedFloatEntryMax#value
      ;
      true
    )
    else (
      trace (sof spinbuttonBoundedFloatEntry#value);
      false
    )

  method keyPressedOnValue(ev:GdkEvent.Key.t) =

    if GdkEvent.Key.keyval ev = GdkKeysyms._Return then (
      self#finish true;
      true
    )
    else (
      trace (sof spinbuttonBoundedFloatEntry#value);
      false
    )


  method finish force = trace "> DialogBoundedFloatEntry.c#finish";

    if force
    || vscaleBoundedFloatEntry#adjustment#lower <> initMinValue
    || vscaleBoundedFloatEntry#adjustment#value <> initValue
    || vscaleBoundedFloatEntry#adjustment#upper <> initMaxValue then (
      self#vscaleValueChanged false ();
    );
    toplevel#destroy();
    trace "< DialogBoundedFloatEntry.c#finish";


  method leave (ev:GdkEvent.Crossing.t) =

    match GdkEvent.Crossing.detail ev with
    | `INFERIOR -> true
    | _ -> self#finish false; true

end
