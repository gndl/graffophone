/*
 * Copyright (C) 2015 Gaetan Dubreil
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

use session::track::{RTrack, Track};

use crate::talker_control::{
    ControlSupply, RTalkerControl, RTalkerControlBase, TalkerControl, TalkerControlBase,
};

pub struct TrackControl {
    base: RTalkerControlBase,
}
pub type RTrackControl = Rc<RefCell<TrackControl>>;

impl TrackControl {
    pub fn new(
        rtrack: &RTrack,
        control_supply: &ControlSupply,
    ) -> Result<TrackControl, failure::Error> {
        let rtalker: RTalker = rtrack.clone();
        let base = TalkerControlBase::new_ref(&rtalker, control_supply, false, false, false)?;

        Ok(Self { base })
    }
    pub fn new_ref(
        rtrack: &RTrack,
        control_supply: &ControlSupply,
    ) -> Result<RTalkerControl, failure::Error> {
        Ok(Rc::new(RefCell::new(TrackControl::new(
            rtrack,
            control_supply,
        )?)))
    }
}

impl TalkerControl for TrackControl {
    fn base<'a>(&'a self) -> &'a RTalkerControlBase {
        &self.base
    }
    /*
     _____________________
    |        NAME         |
    |in #                 |
    |gain #               |
    |channelGain 1 #      |
    |channelGain 2 #      |
    |_____________________|
         */

    fn draw(&self, cc: &Context, talker: &RTalker, talker_controls: &HashMap<Id, RTalkerControl>) {
        let base = self.base.borrow();
        base.draw_connections(cc, talker, talker_controls);
        base.draw_box(cc, talker, 0., 0.);
        //        base.draw_header(cc, talker, 0.);

        base.draw_ears_and_voices(cc, talker, 0.);
    }
    /*    method! draw _ pY =

          self#drawHeader pY false true false;

          self#drawEarsVoices (pY +. GTkr.marge);

          self#setWidth(self#getWidth +. GTkr.space);
          self#setHeight(self#getHeight +. GTkr.space);

          //self#drawBox ~pX ~pY ();
          self#positionTags();


        method! drawBox pX pY =

          let x2 = pX +. mWidth in
          let y2 = pY +. mHeight in

          let box = GnoCanvas.rect ~x1:pX ~y1:pY ~x2 ~y2 ~props:boxProperties mGroup
          in
          box#lower_to_bottom();
    */

    fn move_to(&mut self, _x: f64, _y: f64) { // the move is managed by the mixer
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
