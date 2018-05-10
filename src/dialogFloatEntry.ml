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

class c initValue callback = object (self)
  inherit GraffophoneGui.floatEntryDialog() as super

  val mutable mInitValue = initValue

  initializer
    spinbuttonFloatEntry#set_adjustment vscaleFloatEntry#adjustment;

    ignore(spinbuttonFloatEntry#event#connect#key_release ~callback:self#onKeyPressed);
    ignore(vscaleFloatEntry#adjustment#connect#value_changed self#vscaleValueChanged);
    ignore(buttonAdjust#connect#clicked ~callback:self#adjustRange);

    ignore(toplevel#event#connect#leave_notify ~callback:self#leave);

    self#setAdjustment initValue;

    (* in order to remind the c float 32 init value *)
    mInitValue <- vscaleFloatEntry#adjustment#value;


  method vscaleValueChanged () = trace "> DialogFloatEntry.c#vscaleValueChanged";
    callback vscaleFloatEntry#adjustment#value true;
    trace "< DialogFloatEntry.c#vscaleValueChanged";


  method adjustRange () = trace "> DialogFloatEntry.c#adjustRange";
    self#setAdjustment spinbuttonFloatEntry#value;(**)
    trace "< DialogFloatEntry.c#adjustRange";


  method setAdjustment value = trace("> DialogFloatEntry.c#setAdjustment "^sof value);
    (**)
    if value = 0. then (
      vscaleFloatEntry#adjustment#set_bounds ~lower:(-1.) ~upper:1.
        ~step_incr:0.01 ~page_incr:0.1 ~page_size:0. ();
    )
    else (
      let r = if value > 0. then 10. else -10. in
      let valueStrongWeight = floatStrongWeight value in

      let lower = -.r *. valueStrongWeight in
      let upper = r *. valueStrongWeight in
      let range = upper -. lower in
      let step_incr = range /. 100. in
      let page_incr = range /. 10. in

      vscaleFloatEntry#adjustment#set_bounds
        ~lower ~upper ~step_incr ~page_incr ~page_size:0. ();
    );
    vscaleFloatEntry#adjustment#set_value value;

    labelUpper#set_text(sof vscaleFloatEntry#adjustment#upper);
    labelLower#set_text(sof vscaleFloatEntry#adjustment#lower);

    trace("< DialogFloatEntry.c#setAdjustment "^sof value);


  method onKeyPressed(ev:GdkEvent.Key.t) =

    let key = GdkEvent.Key.keyval ev in

    if key = GdkKeysyms._Return then (

      self#finish ~force:true spinbuttonFloatEntry#value;
      true
    )
    else (
      trace (sof spinbuttonFloatEntry#value);
      false
    )


  method finish ?(force = false) value = trace "> DialogFloatEntry.c#finish";
    toplevel#destroy();

    if force || value <> mInitValue then (
      trace("value("^sof value^") <> mInitValue("^sof mInitValue^") -> callback value false");
      callback value false
    );
    trace "< DialogFloatEntry.c#finish";


  method leave (ev:GdkEvent.Crossing.t) =

    match GdkEvent.Crossing.detail ev with
    | `INFERIOR -> true
    | _ -> self#finish vscaleFloatEntry#adjustment#value; true

end
