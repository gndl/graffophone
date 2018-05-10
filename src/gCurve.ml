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
open Util
open Usual

module SF = SampleFormat

(*let width = 2048*)
let curveHeight = 101
let marginRate = 5. /. 100.
let margin = iof(foi curveHeight *. marginRate)
let height = curveHeight + 2 * margin


class c pVoice (pSsnCtrl : SessionControler.c) = object (self)

  val mDrawingArea = GMisc.drawing_area (*~width*) ~height ()
  (*val mDrawingArea = GBin.event_box ~height ()*)
  val mutable mPixmap = Gdk.Pixmap.create ~width:1 ~height ~depth:24 ()

  val mControls = GPack.hbox ()

  val mVoiceLabel = GMisc.label ~text:(Voice.getIdentity pVoice) ()
  val mVoiceColor = Style.makeVoiceGdkColor pVoice

  val mRemoveButton = GButton.tool_button ~stock:`CLOSE ~expand:false ()
  val mValueRuler = GRange.ruler `VERTICAL ~metric:`PIXELS (*INCHES CENTIMETERS*)
      ~lower:(-1.) ~upper:1. (*~max_size:float*) ~position:0. ()

  val mutable mLeftTick = 0
  val mutable mPixelsPerTick = 0.
  val mutable mSelectedTimeRangeStartX = 0
  val mutable mSelectedTimeRangeEndX = 0
  val mutable mButtonReleaseTime = Int32.zero


  initializer

    let ctrlBox = GPack.vbox ~packing:mControls#add () in

    ctrlBox#add mVoiceLabel#coerce;
    let tb = GButton.toolbar ~orientation:`VERTICAL ~style:`ICONS ~tooltips:true ()
    in
    tb#insert mRemoveButton;
    tb#set_icon_size`MENU;

    ctrlBox#add tb#coerce;
    mValueRuler#misc#set_size_request ~width:30 ~height ();
    mControls#pack ~expand:false mValueRuler#coerce;

  (*
mDrawingArea#misc#modify_bg Style.background;
*)
    ignore(mDrawingArea#event#connect#expose
             ~callback:(fun ev -> self#expose(GdkEvent.Expose.area ev)));


  method init() = 
    Gdk.Window.set_back_pixmap mDrawingArea#misc#window (`PIXMAP mPixmap);

    mDrawingArea#event#add [`BUTTON_PRESS; `BUTTON_RELEASE];

    ignore(mDrawingArea#event#connect#button_release ~callback:(fun ev ->
        let button = GdkEvent.Button.button ev in
        let time = GdkEvent.Button.time ev in
        let x = GdkEvent.Button.x ev in

        let clickTick = mLeftTick + iof(x /. mPixelsPerTick) in
        let doubleClick = Int32.(to_int(sub time mButtonReleaseTime)) < 250 in
        trace("button released at "^sof x^" at "^ Int32.to_string time^" on "^Voice.getIdentity pVoice^(if doubleClick then " double click" else ""));

        if button = 1 then (
          if doubleClick then pSsnCtrl#curve#zoomIn()
          else pSsnCtrl#setStartTick clickTick
        )
        else if button = 3 then (
          if doubleClick then pSsnCtrl#curve#zoomOut()
          else pSsnCtrl#setEndTick clickTick
        );
        mButtonReleaseTime <- time;
        true)
      );


  method getCurve = mDrawingArea#coerce
  method getControls = mControls#coerce

  method getRemoveButton = mRemoveButton


  method expose area =
(*
Gdk.Window.set_back_pixmap mDrawingArea#misc#window (`PIXMAP mPixmap);
false
*)
    let win = mDrawingArea#misc#window in
    let gc = Gdk.GC.create win in

    let x = Gdk.Rectangle.x area in
    let y = Gdk.Rectangle.y area in
    let width = Gdk.Rectangle.width area in
    let height = Gdk.Rectangle.height area in

    Gdk.Draw.pixmap win gc ~xsrc:x ~ysrc:y ~xdest:x ~ydest:y ~width ~height mPixmap;
    (*Gdk.Window.clear win;*)

    self#drawSelectedTimeRange x width;

    true


  method draw leftTick len width pixelsPerTick = trace("leftTick "^soi leftTick^", len "^soi len^", width "^soi width^", pixelsPerTick "^sof pixelsPerTick);

    mLeftTick <- leftTick;
    mPixelsPerTick <- pixelsPerTick;

    let chunkIndex = len / SF.chunkSize in
    let lastLen = len mod SF.chunkSize in

    let values = A.make_matrix (chunkIndex + 1) SF.chunkSize 0. in

    let minVal = ref max_float in
    let maxVal = ref min_float in

    let readChunk i l () =
      let t = leftTick + SF.chunkSize * i in
      let vr = Listen.voice pVoice t l in

      for j = 0 to Listen.getLength vr - 1 do
        let v = Listen.(vr @+ j) in

        if v < !minVal then minVal := v;
        if v > !maxVal then maxVal := v;

        values.(i).(j) <- v;
      done;
    in

    for i = 0 to chunkIndex - 1 do
      pSsnCtrl#synchronize(readChunk i SF.chunkSize);
    done;

    pSsnCtrl#synchronize(readChunk chunkIndex lastLen);

    let valMargin = (!maxVal -. !minVal) *. marginRate in

    let bottom = !minVal -. valMargin in
    let top = !maxVal +. valMargin in

    let coef = foi height /. (bottom -. top) in

    let rec mkPoints i j ft x minY maxY points =

      let newX = iof (ft *. pixelsPerTick) in
      let newY = iof((values.(i).(j) -. bottom) *. coef) + height - 1 in

      let (minY, maxY, points) =
        if newX = x then (
          if newY < minY then (newY, maxY, points)
          else if newY > maxY then (minY, newY, points)
          else (minY, maxY, points)
        )
        else (
          if minY = maxY then (newY, newY, (x, maxY)::points)
          else (newY, newY, (x, minY)::(x, maxY)::points)
        )
      in

      if j > 0 then
        mkPoints i (j - 1) (ft -. 1.) newX minY maxY points
      else if i > 0 then
        mkPoints (i - 1) (SF.chunkSize - 1) (ft -. 1.) newX minY maxY points
      else points
    in

    let fLen = foi len in
    let mid = height / 2 in

    let points = mkPoints chunkIndex (lastLen - 1) (fLen -. 1.)
        (width - 1) mid mid []
    in
    let window = mDrawingArea#misc#window in

    let (pixmapWidth, _) = Gdk.Drawable.get_size mPixmap in

    if width > pixmapWidth then (

      mPixmap <- Gdk.Pixmap.create ~window ~width ~height ();
      Gdk.Window.set_back_pixmap window (`PIXMAP mPixmap);
(*
  mDrawingArea#misc#set_size_request ~width ();
*)
    );
    let gc = Gdk.GC.create window in

    (*Gdk.GC.set_background gc Style.gdkBackgroundColor;*)
    Gdk.GC.set_foreground gc Style.gdkBackgroundColor;
    Gdk.Draw.rectangle mPixmap gc ~x:0 ~y:0 ~width ~height ~filled:true ();

    Gdk.GC.set_foreground gc mVoiceColor;
    Gdk.Draw.lines mPixmap gc points;

    Gdk.GC.set_foreground gc Style.gdkDelimitationColor;
    Gdk.Draw.line mPixmap gc ~x:0 ~y:mid ~x:(width - 1) ~y:mid;

    mValueRuler#set_lower top;
    mValueRuler#set_upper bottom;

    self#clear();


  method drawSelectedTimeRange x width =

    if mSelectedTimeRangeStartX <> mSelectedTimeRangeEndX then (

      let (x, width, color) = if mSelectedTimeRangeStartX <= mSelectedTimeRangeEndX then (
          (mSelectedTimeRangeStartX,
           (mSelectedTimeRangeEndX - mSelectedTimeRangeStartX),
           Style.gdkSelectionColor)
        ) else (
          (mSelectedTimeRangeEndX,
           (mSelectedTimeRangeStartX - mSelectedTimeRangeEndX),
           Style.gdkReverseSelectionColor)
        )
      in
      let win = mDrawingArea#misc#window in
      let gc = Gdk.GC.create win in
      Gdk.GC.set_foreground gc color; 
      Gdk.GC.set_line_attributes gc ~width:margin ~style:`SOLID ~cap:`BUTT ~join:`MITER;

      let alfMargin = margin / 2 in
      let x = x - alfMargin - 1 in
      let width = width + margin + 1 in
      let height = height - margin in

      Gdk.Draw.rectangle win gc ~x ~y:alfMargin ~width ~height ~filled:false ();
    );
(*
let leftX = mini mSelectedTimeRangeStartX mSelectedTimeRangeEndX in
let rightX = maxi mSelectedTimeRangeStartX mSelectedTimeRangeEndX in

if leftX <= x + width && rightX >= x then (

Gdk.GC.set_foreground gc Style.gdkSelectionColor;

if mSelectedTimeRangeStartX >= x
&& mSelectedTimeRangeStartX <= x + width then
(
  Gdk.Draw.line win gc ~x:(mSelectedTimeRangeStartX - 1) ~y:0
 ~x:(mSelectedTimeRangeStartX - 1) ~y:height;
  Gdk.Draw.line win gc ~x:(mSelectedTimeRangeStartX + 1) ~y:0
 ~x:(mSelectedTimeRangeStartX + 1) ~y:height;
);

if leftX <> rightX then
(
if mSelectedTimeRangeEndX >= x
&& mSelectedTimeRangeEndX <= x + width then
(
  Gdk.Draw.line win gc ~x:mSelectedTimeRangeEndX ~y:0
 ~x:mSelectedTimeRangeEndX ~y:height;
);
let selectedTimeRangeWidth = rightX - leftX in

Gdk.Draw.rectangle win gc ~x:leftX ~y:0
~width:selectedTimeRangeWidth ~height:margin ~filled:true ();

Gdk.Draw.rectangle win gc ~x:leftX ~y:(height - margin)
~width:selectedTimeRangeWidth ~height:margin ~filled:true ();
);
);
*)

  method drawTick lastX x =

    let window = mDrawingArea#misc#window in
    let gc = Gdk.GC.create window in

    Gdk.Draw.pixmap window gc
      ~xsrc:lastX ~ysrc:0 ~xdest:lastX ~ydest:0 ~width:1 ~height mPixmap;

    Gdk.GC.set_foreground gc Style.gdkMarkerColor;
    Gdk.Draw.line window gc ~x ~y:0 ~x ~y:height;


  method clear() =
    let window = mDrawingArea#misc#window in
    Gdk.Window.clear window;

    self#drawSelectedTimeRange mSelectedTimeRangeStartX
      (mSelectedTimeRangeEndX - mSelectedTimeRangeStartX);


  method setSelectedTimeRangeX startX endX =
    mSelectedTimeRangeStartX <- startX;
    mSelectedTimeRangeEndX <- endX;


end

let make voice pSsnCtrl = new c voice pSsnCtrl

