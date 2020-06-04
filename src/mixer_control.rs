/*
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
 */
/*
let separatorProperties = [
  `FILL_COLOR_RGBA GTkr.boxBorderColor;
  `WIDTH_PIXELS 1]
*/

use std::cell::RefCell;
//use std::collections::BTreeMap;
use std::collections::HashMap;
use std::rc::Rc;

//use gdk::EventMask;
//use gio::prelude::*;
//use gtk::gtk_sys::GtkScrolledWindow;
//use gtk::prelude::*;
use gtk::DrawingArea;

//use cairo::enums::{FontSlant, FontWeight};
use cairo::Context;

use talker::identifier::Id;
use talker::talker::{RTalker, Talker};

//use session::event_bus::{Notification, REventBus};
use session::mixer::RMixer;

//use crate::graph_control::GraphControl;
//use crate::session_controler::RSessionPresenter;
use crate::talker_control::{RTalkerControl, RTalkerControlBase, TalkerControl, TalkerControlBase};

pub struct TrackControl {
    base: RTalkerControlBase,
}

pub struct MixerControl {
    base: RTalkerControlBase,
    track_controls: Vec<TrackControl>,
}

impl MixerControl {
    pub fn new(mixer: &RMixer, row: i32, column: i32) -> MixerControl {
        let base = TalkerControlBase::new_ref(mixer.borrow().base());

        base.borrow_mut().row = row;
        base.borrow_mut().column = column;

        Self {
            base,
            track_controls: Vec::new(),
        }
    }
    pub fn new_ref(mixer: &RMixer, row: i32, column: i32) -> RTalkerControl {
        Rc::new(RefCell::new(MixerControl::new(mixer, row, column)))
    }

    pub fn track_controls<'a>(&'a self) -> &'a Vec<TrackControl> {
        &self.track_controls
    }
}
impl TalkerControl for MixerControl {
    fn base<'a>(&'a self) -> &'a RTalkerControlBase {
        &self.base
    }
    // fn visit_base<F, P, R>(&mut self, mut f: F, p: P) -> R
    // where
    //     F: FnMut(&mut TalkerControlBase, P) -> R,
    // {
    //     f(self.base.borrow_mut(), p)
    // }
    /*
     _______________
    |     NAME      |
    |---------------|
    |[TRACK 1]      |
    |---------------|
    |[TRACK 2]      |
    |---------------|
    |volume #       |
    |_______________|
    */
    fn draw(
        &self,
        drawing_area: &DrawingArea,
        cr: &Context,
        talker: &RTalker,
        talker_controls: &HashMap<Id, RTalkerControl>,
    ) {
        /*
        self#drawHeader pY false true false;

        let topTrackY = self#getHeight +. GTkr.space in

        let (w, h, gTrks) = L.fold_left mixingConsole#getTracks
            ~init:(self#getWidth +. 20., topTrackY, [])
            ~f:(fun (w, h, gTrks) track ->
                let gTrk = new GTrack.c track ~group:self#getGroup canvas in

                gTrk#setWidth w;

                gTrk#draw (1. -. GTkr.boxRadius) h;

                (max w gTrk#getWidth, h +. gTrk#getHeight, gTrk::gTrks)
              ) in

        self#setWidth(w -. GTkr.boxRadius);
        self#setHeight(h +. GTkr.marge -. pY);
        mGTracks <- gTrks;

        self#drawEarsVoices pY;
        self#drawBox pX pY;

        let w = self#getWidth in
        let points = [|pX; topTrackY; pX +. w; topTrackY|] in

        ignore(GnoCanvas.line ~points ~props:separatorProperties mGroup);

        ignore(L.fold_left gTrks ~init:topTrackY
                 ~f:(fun y gTkr ->
                     let y = y +. gTkr#getHeight in
                     let points = [|pX; y; pX +. w; y|] in

                     ignore(GnoCanvas.line ~points ~props:separatorProperties mGroup);
                     y
                   ));
         */
    }

    //    fn move_to(&mut self, _x: f64, _y: f64) {}
    /*
        fn on_button_release(&mut self, x: f64, y: f64, controler: &RSessionPresenter) -> bool {
            if self.base().borrow_mut().on_button_release(x, y, controler) {
                true
            } else {
                false
            }
        }
    */
}
