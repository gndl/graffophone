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

open Graffophone_plugin
open Usual

module Bus = EventBus


class c (pSsnCtrl : SessionControler.c) =
  object (self)
    val mutable mLeftTick = 0
    val mutable mLastTickX = 0
    val mutable mTicksCount = 0
    val mutable mCurvesWidth = 0
    val mutable mPixelsPerTick = 0.0250

    val mutable mGCurves = []
    val pTable = GPack.table ~columns:2 ~rows:2 ~homogeneous:false ~row_spacings:0 ~col_spacings:0 ()

    val mTimeRuler = GRange.ruler `HORIZONTAL ~metric:`PIXELS
        ~lower:0. ~upper:100. ~position:0. ()
    val mCurvesBox = GPack.vbox ~spacing:1 ~border_width:0 ()
    val mCtrlsBox = GPack.vbox ~spacing:1 ~border_width:0 ()
    val mCurvesSW = GBin.scrolled_window ~placement:`BOTTOM_LEFT
        ~hpolicy:`NEVER ~vpolicy:`AUTOMATIC ~border_width:0 ()
    val mutable mActive = false


    initializer
      Bus.addObserver self#observe;

      mCurvesSW#add_with_viewport mCurvesBox#coerce;

      ignore(mCurvesBox#misc#connect#size_allocate ~callback:(fun area ->
          self#onCurvesAreaResizing Gtk.(area.width);));

      pTable#attach ~left:1 ~top:1 ~expand:`BOTH mCurvesSW#coerce;

      let timeAdjustment = mCurvesSW#hadjustment in

      ignore(timeAdjustment#connect#changed ~callback:(fun() -> self#setTimeRuler()));
      ignore(timeAdjustment#connect#value_changed ~callback:(fun() -> self#setTimeRuler()));

      pTable#attach ~left:1 ~top:0 ~expand:`NONE mTimeRuler#coerce;

      let ctrlsSW = GBin.scrolled_window ~vadjustment:mCurvesSW#vadjustment
          ~placement:`BOTTOM_RIGHT ~hpolicy:`NEVER ~vpolicy:`NEVER ~border_width:0 ()
      in
      ctrlsSW#add_with_viewport mCtrlsBox#coerce;

      pTable#attach ~left:0 ~top:1 ~expand:`Y ctrlsSW#coerce;

      let tb = GButton.toolbar ~orientation:`HORIZONTAL ~style:`ICONS ~tooltips:true ()
      in
      let bzi = GButton.tool_button ~stock:`ZOOM_IN ~packing:tb#insert ~expand:false ()
      in
      ignore(bzi#connect#clicked(fun() -> self#drawCurves ~zoom:2.));

      let bzo = GButton.tool_button ~stock:`ZOOM_OUT ~packing:tb#insert ~expand:false ()
      in
      ignore(bzo#connect#clicked(fun() -> self#drawCurves ~zoom:0.5));

      let b2b = GButton.tool_button ~stock:`MEDIA_PREVIOUS ~packing:tb#insert ~expand:false ()
      in
      let b2p = GButton.tool_button ~stock:`MEDIA_REWIND ~packing:tb#insert ~expand:false ()
      in
      let b2n = GButton.tool_button ~stock:`MEDIA_FORWARD ~packing:tb#insert ~expand:false ()
      in
      ignore(b2b#connect#clicked(fun() ->
          if mLeftTick > 0 then (
            mLeftTick <- 0;
            self#drawCurves ~zoom:1.;
          )));

      ignore(b2p#connect#clicked(fun() ->
          if mLeftTick > 0 then (
            mLeftTick <- max 0 (mLeftTick - ((mTicksCount * 9) / 10));
            self#drawCurves ~zoom:1.;
          )));

      ignore(b2n#connect#clicked(fun() ->
          mLeftTick <- mLeftTick + mTicksCount;
          self#drawCurves ~zoom:1.;
        ));

      tb#set_show_arrow false;
      tb#set_icon_size`MENU(*SMALL_TOOLBAR*);
      pTable#attach ~left:0 ~top:0 ~expand:`NONE tb#coerce;


    method getWidget = pTable#coerce

    method activate v = mActive <- v
    method isActive = mActive

    method getHeight =
      let n = L.length mGCurves in
      if n > 0 then n * GCurve.height + 65 else 0


    method setTimeRuler() =
      let timeAdjustment = mCurvesSW#hadjustment in
      let lower = (foi mLeftTick +. timeAdjustment#value /. mPixelsPerTick)
                  *. 1000. /. SampleFormat.fRate
      in
      let upper = (foi mLeftTick +. (timeAdjustment#value +. timeAdjustment#page_size) /. mPixelsPerTick)
                  *. 1000. /. SampleFormat.fRate
      in
      mTimeRuler#set_lower lower;
      mTimeRuler#set_upper upper;
      mTimeRuler#set_max_size (upper -. lower);


    method addTalkCurve tkrId port =
      try
        let voice = (Session.findTalker tkrId)#getVoices.(port) in

        let gCurve = GCurve.make voice pSsnCtrl in

        mCurvesBox#pack ~expand:false gCurve#getCurve;
        mCtrlsBox#pack ~expand:false gCurve#getControls;

        ignore(gCurve#getRemoveButton#connect#clicked(self#remove gCurve));

        mGCurves <- gCurve::mGCurves;
        self#setTimeRange pSsnCtrl#getStartTick pSsnCtrl#getEndTick;
        self#drawCurve gCurve;
        Bus.notify Bus.CurveAdded;

      with Not_found -> traceRed("CurveView#addTalkCurve : Talker "^soi tkrId^" not found")


    method drawCurve gCurve =
      gCurve#draw mLeftTick mTicksCount mCurvesWidth mPixelsPerTick;


    method drawCurves ~zoom =
      mPixelsPerTick <- mPixelsPerTick *. zoom;
      mTicksCount <- iof(foi mCurvesWidth /. mPixelsPerTick);

      self#setTimeRange pSsnCtrl#getStartTick pSsnCtrl#getEndTick;

      L.iter mGCurves ~f:(fun gc -> self#drawCurve gc);

      self#setTimeRuler();


    method onCurvesAreaResizing width =

      if width > mCurvesWidth then (
        mCurvesWidth <- width;

        self#drawCurves ~zoom:1.
      )
      else mCurvesWidth <- width;


    method drawTick tick =

      if tick >= mLeftTick && mLastTickX < mCurvesWidth then (
        let x = iof(foi(tick - mLeftTick) *. mPixelsPerTick) in

        L.iter mGCurves ~f:(fun gc -> gc#drawTick mLastTickX x);

        mLastTickX <- x;
      )


    method remove gCurve () =
      mCurvesBox#remove gCurve#getCurve;
      mCtrlsBox#remove gCurve#getControls;
      mGCurves <- L.filter((!=) gCurve) mGCurves;
      Bus.notify Bus.CurveRemoved


    method toggleTalkCurve tkrId port =
      try
        let talkCurves = L.filter(fun gc -> gc#voiceIsFrom tkrId port) mGCurves in

        if talkCurves = [] then self#addTalkCurve tkrId port
        else
          L.iter talkCurves ~f:(fun gc -> self#remove gc ())

      with Not_found -> traceRed("CurveView#toggleTalkCurve : Talker "^soi tkrId^" not found")


    method clear() =
      L.iter mGCurves ~f:(fun gc -> gc#clear());
      mLastTickX <- 0;


    method empty() =
      L.iter(fun gCurve ->
          mCurvesBox#remove gCurve#getCurve;
          mCtrlsBox#remove gCurve#getControls;
        ) mGCurves;

      mGCurves <- [];
      mLeftTick <- 0;
      mLastTickX <- 0;

      Bus.notify Bus.CurveRemoved;


    method setTimeRange startTick endTick =

      let startX = iof(foi(startTick - mLeftTick) *. mPixelsPerTick) in
      let endX = iof(foi(endTick - mLeftTick) *. mPixelsPerTick) in

      L.iter mGCurves ~f:(fun gc -> gc#setSelectedTimeRangeX startX endX)


    (* observer methods *)
    method observe = function
      | Bus.Tick t -> self#drawTick t
      | Bus.TimeRange (startTick, endTick) ->
        self#setTimeRange startTick endTick;
        self#clear()
      | Bus.State State.Stopped -> self#clear()
      | Bus.TalkSelected (tkrId, port) -> self#toggleTalkCurve tkrId port
      | Bus.Session -> self#empty()
      | _ -> ()

  end

let make appCtrl = new c appCtrl
