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
  inherit GraffophoneGui.windowBoundedFloatEntryDialog()

  initializer
    spinbuttonBoundedFloatEntry#set_adjustment vscaleBoundedFloatEntry#adjustment;

    ignore(spinbuttonBoundedFloatEntry#event#connect#key_release ~callback:self#keyPressedOnValue);
    ignore(vscaleBoundedFloatEntry#adjustment#connect#value_changed ~callback:(self#valueChanged true));

    ignore(toplevel#event#connect#leave_notify ~callback:self#leave);

    let minValue, maxValue = if initMinValue = initMaxValue then (-20000., 20000.)
      else (initMinValue, initMaxValue) in

    ignore(spinbuttonBoundedFloatEntryMin#adjustment#connect#value_changed ~callback:self#setAdjustment);

    spinbuttonBoundedFloatEntryMin#adjustment#set_bounds
      ~lower:(-100000.) ~upper:100000. ~step_incr:100. ~page_incr:1000. ~page_size:0. ();

    spinbuttonBoundedFloatEntryMin#set_value minValue;


    ignore(spinbuttonBoundedFloatEntryMax#adjustment#connect#value_changed ~callback:self#setAdjustment);

    spinbuttonBoundedFloatEntryMax#adjustment#set_bounds
      ~lower:(-100000.) ~upper:100000. ~step_incr:100. ~page_incr:1000. ~page_size:0. ();

    spinbuttonBoundedFloatEntryMax#set_value maxValue;


    self#setAdjustment ();

    spinbuttonBoundedFloatEntry#adjustment#set_value initValue;


  method valueChanged onTheFly () = trace "> DialogBoundedFloatEntry.c#valueChanged";
    trace(
      "min value = "^sof vscaleBoundedFloatEntry#adjustment#lower ^
      "\nvalue = "^sof vscaleBoundedFloatEntry#adjustment#value ^
      "\nmax value = "^sof vscaleBoundedFloatEntry#adjustment#upper ^
      if onTheFly then "\n on the fly" else ""
    );
    callback
      spinbuttonBoundedFloatEntryMin#adjustment#value
      spinbuttonBoundedFloatEntry#adjustment#value
      spinbuttonBoundedFloatEntryMax#adjustment#value
      onTheFly;
    trace "< DialogBoundedFloatEntry.c#valueChanged"


  method setAdjustment() =

    let minValue = spinbuttonBoundedFloatEntryMin#value in
    let maxValue = spinbuttonBoundedFloatEntryMax#value in

    trace("> DialogBoundedFloatEntry.c#setAdjustment "^sof minValue^" "^sof maxValue);

    if minValue <> maxValue then (
      let lower, upper =
        if minValue < maxValue then (minValue, maxValue)
        else (
          spinbuttonBoundedFloatEntryMin#set_value maxValue;
          spinbuttonBoundedFloatEntryMax#set_value minValue;

          (maxValue, minValue)
        ) in

      if lower > vscaleBoundedFloatEntry#adjustment#value then (
        vscaleBoundedFloatEntry#adjustment#set_value lower;
      )
      else if upper < vscaleBoundedFloatEntry#adjustment#value then (
        vscaleBoundedFloatEntry#adjustment#set_value upper;
      );
      
      let range = upper -. lower in

      vscaleBoundedFloatEntry#adjustment#set_bounds ~lower ~upper
        ~step_incr:(range /. 1000.) ~page_incr:(range /. 100.) ~page_size:0. ();
    );
    trace("< DialogBoundedFloatEntry.c#setAdjustment");


  method keyPressedOnBoundValue(ev:GdkEvent.Key.t) =

    if GdkEvent.Key.keyval ev = GdkKeysyms._Return then (
      self#setAdjustment();
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
      self#valueChanged false ();
    );
    toplevel#destroy();
    trace "< DialogBoundedFloatEntry.c#finish";


  method leave (ev:GdkEvent.Crossing.t) =

    match GdkEvent.Crossing.detail ev with
    | `INFERIOR -> true
    | _ -> self#finish false; true

end
