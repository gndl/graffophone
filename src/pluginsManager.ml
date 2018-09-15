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

open Graffophone_plugin
open Usual

let loadPlugins () =
  try
    let open Plugin in

    (* Inner talkers registering *)
    L.iter ~f:Factory.addTalkerHandler InnerTalkers.handler.talkerHandlers;
    traceGreen(InnerTalkers.handler.name^" talkers registered");

    Factory.addTalkerHandler FileInput.handler;

    (* Plugins talkers registering *)
    let pluginsDir = "_build/default/plugins/" in
    let suffix = if Dynlink.is_native then ".cmxs" else ".cma"
    in
    let loadIfPlugin fileName =
      if Filename.check_suffix fileName suffix then
        (
          Dynlink.loadfile (pluginsDir^fileName);

          if Plugin.isRegistered() then (

            let ph = Plugin.getHandler() in

            L.iter ~f:Factory.addTalkerHandler ph.talkerHandlers;
            traceGreen("Plugin "^ph.name^" ("^fileName^") registered");
          )
          else (
            traceRed("Plugin "^fileName^" did not register\n");
          );
          Plugin.reset();
        )
    in
    Array.iter loadIfPlugin (Sys.readdir pluginsDir);

    (* Output registering *)
    Factory.addOutputMaker FileOutput.handler;
    Factory.addOutputMaker PlaybackOutput.handler;

  with
    Dynlink.Error e -> print_endline (Dynlink.error_message e)

