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

module Bus = EventBus

class c (pSsnCtrl : SessionControler.c) (graphView : GraphView.c) =
  object (self) inherit GraffophoneGui.applicationView() as super

	val mCurveView = CurveView.make pSsnCtrl
	

	initializer
		Bus.addObserver self#observe;

		toplevel#maximize();

		(* File menu event connection *)
	  super#bind ~name:"on_newSessionMenuItem_activate" ~callback:pSsnCtrl#newSession;
	  super#bind ~name:"on_openSessionMenuItem_activate" ~callback:self#openSession;
	  super#bind ~name:"on_saveSessionMenuItem_activate" ~callback:pSsnCtrl#saveSession;
	  super#bind ~name:"on_saveSessionAsMenuItem_activate" ~callback:self#saveSessionAs;

		(* Help menu event connection *)
	  super#bind ~name:"on_about_activate" ~callback:GuiUtility.aboutDialog;

		(* Application toolbar event connection *)
	  super#bind ~name:"on_newSessionToolbarButton_clicked" ~callback:pSsnCtrl#newSession;
	  super#bind ~name:"on_openSessionToolbarButton_clicked" ~callback:self#openSession;
	  super#bind ~name:"on_saveSessionToolbarButton_clicked" ~callback:pSsnCtrl#saveSession;
	  super#bind ~name:"on_saveSessionAsToolbarButton_clicked" ~callback:self#saveSessionAs;
	  super#bind ~name:"on_preferencesToolbarButton_clicked" ~callback:self#showPreferences;

	  super#bind ~name:"on_playButton_clicked" ~callback:self#play;
(*	  super#bind ~name:"on_pauseButton_clicked" ~callback:self#pause;*)
	  super#bind ~name:"on_stopButton_clicked" ~callback:self#stop;
	  super#bind ~name:"on_beginButton_clicked" ~callback:self#toBegin;
	  super#bind ~name:"on_playToButton_clicked" ~callback:self#playTo;
	  super#bind ~name:"on_tickSpinButton_input" ~callback:self#tickEntered;

		loopButton#misc#hide();
		applicationToolbar#set_icon_size`MENU;

				(* integrete curveView
		graphCurveVpaned#pack1 ~resize:true ~shrink:true mCurveView#getWidget;
		*)
		curveZoneEventbox#add mCurveView#getWidget;

		(* integrete graphView canvas *)
		let graphCanvas = graphView#getGraphCanvas in

		scrolledwindowGraph#add graphCanvas#coerce;
		scrolledwindowGraph#set_hadjustment graphCanvas#hadjustment;
		scrolledwindowGraph#set_vadjustment graphCanvas#vadjustment;

		(* Graph toolbar event connection *)
		ignore(toolbuttonShowCurve#connect#toggled(
			fun() -> mCurveView#activate toolbuttonShowCurve#get_active));

		loopTalkerButton#misc#hide();
		graphToolbar#set_icon_size`MENU;

		(* Key event connection *)
		ignore(applicationView#event#connect#key_press ~callback:self#keyPressed);
		ignore(applicationView#event#connect#key_release ~callback:self#keyReleased);

		(* connect quit method to destroy event *)
		ignore(applicationView#connect#destroy ~callback:self#quit);



	method init () =
		statusbar#misc#hide();
		graphCurveVpaned#set_position 100;


	method openSession () =
		match GuiUtility.openFile ~filter:GuiUtility.sessionFilter (*~parent:toplevel*) () with
			| Some f -> super#applicationView#set_title(f^" : "^appName); pSsnCtrl#openSession f
			| None -> ()

	method saveSessionAs () =
		match GuiUtility.saveFile ~parent:toplevel GuiUtility.sessionFilter with
			| Some f -> super#applicationView#set_title(f^" : "^appName); pSsnCtrl#saveSessionAs f
			| None -> ()

	method showPreferences () =
		let _ = new ControlPanel.c in ()

	method quit () =
		pSsnCtrl#stop 0;
		GMain.quit()


	method play() = pSsnCtrl#play 0;
	method pause() = pSsnCtrl#pause 0;
	method stop() = pSsnCtrl#stop 0;
	method toBegin() = pSsnCtrl#setStartTick 0;
	method playTo() = pSsnCtrl#setStartTick(tickSpinButton#value_as_int);
	method tickEntered() = pSsnCtrl#setStartTick tickSpinButton#value_as_int; trace("tickEntered = "^soi tickSpinButton#value_as_int)


	method keyPressed ev =
		let key = GdkEvent.Key.keyval ev in
		
		if key = GdkKeysyms._Control_L || key = GdkKeysyms._Control_R then
			pSsnCtrl#setControlKeyPressed true
		else
		if key = GdkKeysyms._Shift_L || key = GdkKeysyms._Shift_R then
			pSsnCtrl#setShiftKeyPressed true
		else
		if key = GdkKeysyms._Alt_L || key = GdkKeysyms._Alt_R then
			pSsnCtrl#setAltKeyPressed true;
		
		false

	method keyReleased ev =
		let key = GdkEvent.Key.keyval ev in
		
		if key = GdkKeysyms._Control_L || key = GdkKeysyms._Control_R then
			pSsnCtrl#setControlKeyPressed false
		else
		if key = GdkKeysyms._Shift_L || key = GdkKeysyms._Shift_R then
			pSsnCtrl#setShiftKeyPressed false
		else
		if key = GdkKeysyms._Alt_L || key = GdkKeysyms._Alt_R then
			pSsnCtrl#setAltKeyPressed false;
		
		false


	(* menubarGraph generation *)
	method buildGraphMenu talkersRange = trace "buildGraphMenu";

		let addNewTalker kind () = pSsnCtrl#addNewTalker kind
		in
		L.iter talkersRange ~f:(fun (category, catKinds) ->

			let item = GMenu.menu_item ~label:category ~packing:menubarGraph#append ()
			in
			let menu = GMenu.menu ~packing:item#set_submenu ()
			in
			let entries = L.map catKinds ~f:(fun knd -> `I (knd, addNewTalker knd))
			in
			GToolbox.build_menu menu ~entries;
		);


	method showMessage message messageType =
		ignore(GWindow.message_dialog ~message ~message_type:messageType
			~buttons:GWindow.Buttons.ok ());
		traceRed message

	method onTimeRangeChange startTick endTick =
		tickSpinButton#set_value(foi startTick /. SampleFormat.fRate);

		if startTick = endTick then (
			loopButton#misc#hide();
			loopTalkerButton#misc#hide();
		)
		else (
			loopButton#misc#show();
			loopTalkerButton#misc#show();
		);


	(* observer method *)
	method observe =	function
		| Bus.Tick t -> tickSpinButton#set_value(foi t /. SampleFormat.fRate)
		| Bus.TimeRange (st, et) -> self#onTimeRangeChange st et
		| Bus.State s -> (
			match s with
  		| State.Playing -> playButton#set_stock_id `MEDIA_PAUSE; traceGreen "play"
  		| State.Paused -> playButton#set_stock_id `MEDIA_PLAY; traceGreen "pause"
  		| State.Stopped -> playButton#set_stock_id `MEDIA_PLAY; traceGreen "stop"
		)
		| Bus.CurveAdded
		| Bus.CurveRemoved -> graphCurveVpaned#set_position 720(*min mCurveView#getHeight 480*);
		| Bus.Info msg -> self#showMessage msg `INFO
		| Bus.Warning msg -> self#showMessage msg `WARNING
		| Bus.Error msg -> self#showMessage msg `ERROR
		| Bus.TalkersRange trs -> self#buildGraphMenu trs
		| _ -> ()

end

