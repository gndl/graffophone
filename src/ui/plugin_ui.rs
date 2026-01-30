

extern crate luil;

use talker::lv2_handler;
use talker::talker::RTalker;
use talker::identifier::{Id, Identifiable};
use talker::audio_format::AudioFormat;

use crate::session_presenter::RSessionPresenter;
use crate::session::event_bus::Notification;

const HMI_UPDATE_PERIOD: u64 = 333;

pub struct UiParameters {
    talker_id: Id,
    plugin_uri: String,
    bundle_uri: String,
    instance_name: String,
}
pub struct Manager {
    luil: luil::Luil,
    pending_ui: Option<UiParameters>,
}

impl Manager {
    pub fn new() -> Manager {
        let luil = luil::Luil::new(luil::HostConfiguration {
            sample_rate: AudioFormat::sample_rate() as f64,
            support_touch: false,
            support_peak_protocol: false,
        });

        Self {luil, pending_ui: None}
    }

    pub fn prepare_new_ui(&mut self, talker: &RTalker) {
        let bundle_uri = lv2_handler::get_bundle_uri(&talker.model()).unwrap();

        self.pending_ui = Some(UiParameters {
            talker_id: talker.id(),
            plugin_uri: talker.model(),
            bundle_uri,
            instance_name: talker.name(),
        });
    }

    pub fn show_pending_ui(&mut self, session_presenter: &RSessionPresenter) -> Result<(), failure::Error> {

        if let Some(ui_param) = &self.pending_ui {
            let ui_connector = self.luil.launch_plugin_ui(
                &ui_param.plugin_uri,
                Some(&ui_param.bundle_uri),
                &ui_param.instance_name,
                |uri| lv2_handler::urid_map(&uri)
            )?;
    
            session_presenter.borrow_mut().add_plugin_handle(ui_param.talker_id, ui_connector);

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
