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

let devicesColumns = new GTree.column_list
let devicesNameColumn = devicesColumns#add Gobject.Data.string

class c =
  object
    val gui = new GraffophoneGui.controlPanelDialog()

    initializer
      let txtRndrr = GTree.cell_renderer_text [] in

      (* build output devices tree view *)
      let selection = gui#devicesTreeview#selection in

      let devsCol = GTree.view_column ~title:"Name"
          ~renderer:(txtRndrr, ["text", devicesNameColumn]) ()
      in
      ignore(gui#devicesTreeview#append_column devsCol);

      selection#set_mode`SINGLE;

      (* fill output devices tree view *)
      let currDev = Configuration.getOutputDeviceName in
      let devicesModel = GTree.list_store devicesColumns in

      let currRow = L.fold_left ~f:(fun currRow name ->
          let row = devicesModel#append () in
          devicesModel#set ~row ~column:devicesNameColumn name;

          if name = currDev then Some row else currRow
        )
          ~init:None (Device.getOutputsNames())
      in

      gui#devicesTreeview#set_model(Some (devicesModel#coerce));

      ignore(match currRow with None -> ()
                              | Some row -> gui#devicesTreeview#selection#select_iter row
        );

      (* manage dialog control *)
      let closeDialog() = gui#toplevel#destroy() in

      let onValidation() =
        let model = gui#devicesTreeview#model in

        let _ = match gui#devicesTreeview#selection#get_selected_rows with
          | [] -> ()
          | path::_ ->
            let row = model#get_iter path in
            let name = model#get ~row ~column:devicesNameColumn in
            Configuration.setOutputDeviceName name;
        in
        Configuration.save();
        closeDialog();
      in

      ignore(gui#controlPanelOkButton#connect#clicked ~callback:onValidation);
      ignore(gui#controlPanelCancelButton#connect#clicked ~callback:closeDialog);
      ignore(gui#toplevel#connect#close ~callback:closeDialog);

      gui#toplevel#show()

  end
