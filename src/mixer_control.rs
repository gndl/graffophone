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
//use talker::talker::RTalker;

use session::event_bus::Notification;
use session::mixer::RMixer;

use crate::graph_presenter::{GraphPresenter, RGraphPresenter};
use crate::talker_control::{
    ControlSupply, RTalkerControl, RTalkerControlBase, TalkerControl, TalkerControlBase,
};

pub struct MixerControl {
    base: RTalkerControlBase,
}

impl MixerControl {
    pub fn new(
        rmixer: &RMixer,
        control_supply: &ControlSupply,
    ) -> Result<MixerControl, failure::Error> {
        let mixer = rmixer.borrow();
        let base =
            TalkerControlBase::new_ref(mixer.talker(), control_supply, true, false, false, false)?;

        Ok(Self { base })
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
}

impl TalkerControl for MixerControl {
    fn base<'a>(&'a self) -> &'a RTalkerControlBase {
        &self.base
    }
    /*
     _______________
    |     NAME      |
    |volume #       |
    |---------------|
    |[TRACK 1]      |
    |---------------|
    |[TRACK 2]      |
    |_______________|
    */
    fn draw_connections(&self, cc: &Context, talker_controls: &HashMap<Id, RTalkerControl>) {
        self.base.borrow().draw_connections(cc, talker_controls);
    }

    fn draw(&self, cc: &Context, graph_presenter: &GraphPresenter) {
        let base = self.base.borrow();
        base.draw_box(cc, graph_presenter);
        base.draw_header(cc, false);

        base.draw_ears_and_voices(cc, graph_presenter);
    }

    fn move_to(&mut self, x: f64, y: f64) {
        self.base().borrow_mut().move_to(x, y);
    }

    fn on_button_release(
        &self,
        x: f64,
        y: f64,
        graph_presenter: &RGraphPresenter,
    ) -> Result<Option<Vec<Notification>>, failure::Error> {
        self.base()
            .borrow()
            .on_button_release(x, y, graph_presenter)
    }
}
