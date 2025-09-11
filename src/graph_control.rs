use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use gtk::gdk;
use gtk::glib;
use crate::gtk::prelude::{PopoverExt, WidgetExt};

use talker::identifier::{Id, Index};

use session::event_bus::{Notification, REventBus};

use crate::graph_presenter::RGraphPresenter;
use crate::session_presenter::RSessionPresenter;
use crate::talker_control::RTalkerControl;
use crate::ui;


pub struct GraphControl {
    session_presenter: RSessionPresenter,
    graph_presenter: RGraphPresenter,
    talker_controls: Vec<RTalkerControl>,
    event_bus: REventBus,
}
pub type RGraphControl = Rc<RefCell<GraphControl>>;

impl GraphControl {
    pub fn new_ref(
        window: &gtk::ApplicationWindow,
        session_presenter: &RSessionPresenter,
        graph_presenter: &RGraphPresenter,
        event_bus: &REventBus,
    ) -> RGraphControl {
        let instance = Rc::new(RefCell::new(Self {
            session_presenter: session_presenter.clone(),
            graph_presenter: graph_presenter.clone(),
            talker_controls: Vec::new(),
            event_bus: event_bus.clone(),
        }));

        // Key pressed event
        let key_pressed_ctrl = instance.clone();
        let key_pressed_event_controller = gtk::EventControllerKey::builder().build();
        key_pressed_event_controller.connect_key_pressed(move |_, key, _, _| key_pressed_ctrl.borrow().on_key_pressed(key));
        window.add_controller(key_pressed_event_controller);

        // Key released event
        let key_released_ctrl = instance.clone();
        let key_released_event_controller = gtk::EventControllerKey::builder().build();
        key_released_event_controller.connect_key_released(move |_, key, _, _| key_released_ctrl.borrow().on_key_released(key));
        window.add_controller(key_released_event_controller);

        instance
    }

    pub fn set_talker_controls(&mut self, talker_controls: &HashMap<Id, RTalkerControl>) {
        self.talker_controls.clear();

        for tkrc in talker_controls.values() {
            self.talker_controls.push(tkrc.clone());
        }
    }

    pub fn on_key_pressed(&self, key: gdk::Key) -> glib::signal::Propagation{

        if key == gdk::Key::Control_L || key == gdk::Key::Control_R {
            self.graph_presenter.borrow_mut().set_multi_selection(true);
        }
        glib::signal::Propagation::Proceed
    }

    pub fn on_key_released(&self, key: gdk::Key) {

        if key == gdk::Key::Control_L || key == gdk::Key::Control_R {
            self.graph_presenter.borrow_mut().set_multi_selection(false);
        }
    }

    fn on_ear_value_selected(
        &self,
        x: f64,
        y: f64,
        talker_id: Id,
        ear_idx: Index,
        set_idx: Index,
        hum_idx: Index,
        popover: &gtk::Popover,
    ) {
        let session_presenter = self.session_presenter.borrow();
        let tkr = session_presenter.find_talker(talker_id).unwrap();

        let (min, max, def) = tkr.ear(ear_idx).hum_range(hum_idx);
        let cur = tkr.ear(ear_idx).talk_value_or_default(set_idx, hum_idx);

        let hum = self.graph_presenter.borrow().backup_hum(talker_id, ear_idx, set_idx, hum_idx);

        let gp_on_scale = self.graph_presenter.clone();
        let gp_on_ok = self.graph_presenter.clone();
        let gp_on_cancel = self.graph_presenter.clone();
        let gp_on_default = self.graph_presenter.clone();

        let ok_popover = popover.clone();
        let cancel_popover = popover.clone();
        let default_popover = popover.clone();

        let dialog = ui::bounded_float_entry::create(
            min,
            max,
            def,
            cur,
            move |v| {
                gp_on_scale.borrow_mut().set_talker_ear_talk_value_volatly(
                    talker_id, ear_idx, set_idx, hum_idx, 0, v)
            },
            move |v| {
                gp_on_ok.borrow_mut().set_talker_ear_hum_value(
                    talker_id, ear_idx, set_idx, hum_idx, v);
                ok_popover.popdown()
            },
            move |_| {
                gp_on_cancel.borrow_mut().set_talker_ear_hum(hum.clone());
                cancel_popover.popdown()
            },
            move |_| {
                gp_on_default.borrow_mut().set_talker_ear_hum_value(
                    talker_id, ear_idx, set_idx, hum_idx, def);
                default_popover.popdown()
            },
        );

        popover.set_child(Some(&dialog));
        popover.set_pointing_to(Some(&gdk::Rectangle::new(x as i32, y as i32, 1, 1)));
        popover.popup();
    }

    pub fn on_button_release(&self, area_x: f64, area_y: f64, popover: &gtk::Popover) {
        let x = area_x - 5.;
        let y = area_y - 5.;

        for tkrc in &self.talker_controls {
            match tkrc.borrow().on_button_release(x, y, &self.graph_presenter) {
                Ok(None) => (),
                Ok(Some(notifications)) => {
                    for notification in notifications {
                        match notification {
                            Notification::EarValueSelected(
                                talker_id,
                                ear_idx,
                                set_idx,
                                hum_idx,
                            ) => self.on_ear_value_selected(
                                x, y, talker_id, ear_idx, set_idx, hum_idx, popover,
                            ),
                            _ => self.event_bus.borrow().notify(notification),
                        }
                    }
                    return;
                }
                Err(e) => self.event_bus.borrow().notify_error(e),
            }
        }
    }
}
