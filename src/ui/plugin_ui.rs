
extern crate luil;

use talker::talker::RTalker;
use talker::identifier::{Id, Identifiable};

use crate::session_presenter::RSessionPresenter;
use crate::session::event_bus::Notification;

const HMI_UPDATE_PERIOD: u64 = 333;

pub struct Manager {
    pending_ui: Option<Id>,
}

impl Manager {
    pub fn new() -> Manager {
        Self {pending_ui: None}
    }

    pub fn prepare_new_ui(&mut self, talker: &RTalker) {
        self.pending_ui = Some(talker.id());
    }

    pub fn show_pending_ui(&mut self, session_presenter: &RSessionPresenter) -> Result<(), failure::Error> {

        if let Some(talker_id) = self.pending_ui {
            session_presenter.borrow_mut().add_plugin_handle(talker_id);

            if session_presenter.borrow().ui_count() == 1 {
                let period = std::time::Duration::from_millis(HMI_UPDATE_PERIOD);
                let idle_session_presenter = session_presenter.clone();

                let _ = glib::source::timeout_add_local(period, move || {
                    let (modification_count, ui_count) = idle_session_presenter.borrow_mut().update_band_and_ui_count();

                    if modification_count > 0 {
                        idle_session_presenter.borrow().notify(Notification::TalkerChanged);
                    }

                    if ui_count > 0 {
                        glib::ControlFlow::Continue
                    }
                    else {
                        glib::ControlFlow::Break
                    }
                });
            }
            self.pending_ui = None;
        }
        Ok(())
    }
}
