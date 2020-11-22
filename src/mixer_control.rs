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
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use cairo::Context;

use talker::identifier::Id;
use talker::talker::{RTalker, Talker};

//use session::event_bus::{Notification, REventBus};
use session::mixer::{Mixer, RMixer};

use crate::track_control::{RTrackControl, TrackControl};
//use crate::session_controler::RSessionPresenter;
use crate::talker_control::{
    ControlSupply, RTalkerControl, RTalkerControlBase, TalkerControl, TalkerControlBase,
};

pub struct MixerControl {
    base: RTalkerControlBase,
    track_controls: Vec<RTalkerControl>,
    track_controls_b_y: f64,
}

impl MixerControl {
    pub fn new(
        rmixer: &RMixer,
        control_supply: &ControlSupply,
    ) -> Result<MixerControl, failure::Error> {
        let rtalker: RTalker = rmixer.clone();
        let base = TalkerControlBase::new_ref(&rtalker, control_supply, true, false, false)?;

        let mut track_controls = Vec::new();

        let mut width = base.borrow_mut().width();
        let mut height = base.borrow_mut().height();
        let track_controls_b_y = height;

        for track in rmixer.borrow().tracks() {
            let track_control = TrackControl::new_ref(track, control_supply)?;

            width = f64::max(width, track_control.borrow().width());
            height += track_control.borrow().height();

            track_controls.push(track_control);
        }

        for tc in &track_controls {
            tc.borrow_mut().set_width(width);
        }

        base.borrow_mut().set_width(width);
        base.borrow_mut().set_height(height);
        /*
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
        */
        Ok(Self {
            base,
            track_controls,
            track_controls_b_y,
        })
    }
    pub fn new_ref(
        mixer: &RMixer,
        control_supply: &ControlSupply,
    ) -> Result<RTalkerControl, failure::Error> {
        Ok(Rc::new(RefCell::new(MixerControl::new(
            mixer,
            control_supply,
        )?)))
    }

    pub fn track_controls<'a>(&'a self) -> &'a Vec<RTalkerControl> {
        &self.track_controls
    }
}
impl TalkerControl for MixerControl {
    fn base<'a>(&'a self) -> &'a RTalkerControlBase {
        &self.base
    }
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
    fn draw(&self, cc: &Context, talker: &RTalker, talker_controls: &HashMap<Id, RTalkerControl>) {
        let base = self.base.borrow();
        base.draw_connections(cc, talker, talker_controls);
        base.draw_box(cc, talker, 0., 0.);
        base.draw_header(cc, talker, 0.);

        base.draw_ears_and_voices(cc, talker, 0.);

        for trkc in &self.track_controls {
            trkc.borrow().draw(cc, talker, talker_controls);
        }
        /*
                        self#drawHeader pY false true false;


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

    fn move_to(&mut self, x: f64, y: f64) {
        let mut trkc_b_y = y + self.track_controls_b_y;
        for trkc in &self.track_controls {
            trkc.borrow().base().borrow_mut().move_to(x, trkc_b_y);
            trkc_b_y += trkc.borrow().height();
        }

        self.base().borrow_mut().move_to(x, y);
    }
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
