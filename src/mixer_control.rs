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

use session::event_bus::Notification;
use session::mixer::{self, RMixer};

use crate::graph_presenter::{GraphPresenter, RGraphPresenter};
use crate::talker_control::{RTalkerControl, RTalkerControlBase, TalkerControl, TalkerControlBase};
use crate::ui::{self, control::{Area, ControlSupply}};
use crate::util;


struct SoloMuteControl {
    solo_area: Area,
    mute_area: Area,
}

pub struct MixerControl {
    base: RTalkerControlBase,
    solo_mutes: Vec<SoloMuteControl>,
}

impl MixerControl {
    pub fn new(
        rmixer: &RMixer,
        control_supply: &ControlSupply,
    ) -> Result<MixerControl, failure::Error> {
        let mut solo_mutes = Vec::new();

        let mixer = rmixer.borrow();
        let base =
            TalkerControlBase::new_ref(
                mixer.talker(),
                control_supply,
                true,
                false,
                false,
                false,
                |ear_idx, _, b_x, b_y| {

                    if ear_idx == mixer::TRACKS_EAR_INDEX {
                        let mut solo_area = ui::control::dim_to_area(b_x, b_y, &control_supply.solo_dim);
                        let mut mute_area = ui::control::dim_to_area(b_x, solo_area.e_y + 1., &control_supply.mute_dim);

                        solo_area.farthest_right_adjustment(&mut mute_area);

                        let ctrl = SoloMuteControl {solo_area, mute_area};

                        solo_mutes.push(ctrl);
                        (solo_area.e_x.max(mute_area.e_x), mute_area.e_y)
                    }
                    else {
                        (b_x, b_y)
                    }
                }
            )?;

        Ok(Self { base, solo_mutes })
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

    fn draw_solo_mutes(&self, cc: &Context, graph_presenter: &GraphPresenter) -> Result<(), cairo::Error> {
        let base = self.base.borrow();
        let tkr_id = base.id();
        

        for (trk_idx, ctrls) in self.solo_mutes.iter().enumerate() {
            base.draw_control(
                cc,
                &ctrls.solo_area,
                ui::control::SOLO_TAG,
                ui::style::add,
                ui::style::background,
                graph_presenter.is_solo_track(tkr_id, trk_idx),
            )?;

            base.draw_control(
                cc,
                &ctrls.mute_area,
                ui::control::MUTE_TAG,
                ui::style::sup,
                ui::style::background,
                graph_presenter.is_mute_track(tkr_id, trk_idx),
            )?;
        }
        Ok(())
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
        util::print_cairo_result(self.base.borrow().draw_connections(cc, talker_controls));
    }

    fn draw(&self, cc: &Context, graph_presenter: &GraphPresenter) {
        let base = self.base.borrow();
        util::print_cairo_result(base.draw_box(cc, graph_presenter));
        util::print_cairo_result(base.draw_header(cc, false));

        util::print_cairo_result(base.draw_ears_and_voices(cc, graph_presenter));
        util::print_cairo_result(self.draw_solo_mutes(cc, graph_presenter));
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
        let base  = self.base.borrow();

        if base.is_under(x, y) {
            let (rx, ry) = base.relative_coordinates(x, y);

            for (trk_idx, ctrls) in self.solo_mutes.iter().enumerate() {

                if ctrls.solo_area.is_under(rx, ry) {
                    let notifications = graph_presenter.borrow_mut().set_solo_track(base.id(), trk_idx)?;
                    return Ok(Some(notifications));
                }

                if ctrls.mute_area.is_under(rx, ry) {
                    let notifications = graph_presenter.borrow_mut().set_mute_track(base.id(), trk_idx)?;
                    return Ok(Some(notifications));
                }

                if self.base.borrow().sup_set_is_under(mixer::TRACKS_EAR_INDEX, trk_idx, x, y) {
                    let notifications = graph_presenter
                        .borrow_mut()
                        .sup_mixer_track(base.id(), trk_idx)?;
                    return Ok(Some(notifications));
                }
            }

            base.on_button_release(x, y, graph_presenter)
        } else {
            Ok(None)
        }
    }
}
