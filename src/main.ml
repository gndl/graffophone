(* 
 * Copyright (C) 2015 Ga�tan Dubreil
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

let () =
	
	Printexc.record_backtrace true;
	try
	Device.initialize();

	PluginsManager.loadPlugins ();

  let ssnCtrl = new SessionControler.c in

	let graphView = new GraphView.c ssnCtrl#graph in
	let appView = new ApplicationView.c ssnCtrl graphView in

	ssnCtrl#init();
	appView#init();

	let timeoutHookId = GMain.Timeout.add ~ms:100 ~callback:EventBus.asyncUpdate
	in

	GtkThread.main ();

	GMain.Timeout.remove timeoutHookId;

	Device.terminate();

	with exc -> (
		traceMagenta(Printexc.to_string exc);
		traceYellow(Printexc.get_backtrace())
	)
