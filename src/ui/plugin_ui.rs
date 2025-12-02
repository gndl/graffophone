
use std::collections::HashMap;

use std::ffi::CString;
use std::str::FromStr;

extern crate luil;

use talker::lv2_handler;
use talker::talker::RTalker;
use talker::identifier::{Id, Identifiable};
use talker::audio_format::AudioFormat;

use session::band::Operation;
use session::talkers::lv2;
use crate::session_presenter::RSessionPresenter;
use crate::session::event_bus::Notification;

const HMI_UPDATE_PERIOD: u64 = 333;


struct HostPresenter {
    talker_id: Id,
    session_presenter: RSessionPresenter,
    port_symbol_indexes: HashMap<String, u32>,
    ears_indexes: Vec<usize>,
}

impl HostPresenter {
    pub fn new(talker_id: Id, plugin_uri: &str, session_presenter: &RSessionPresenter) -> Result<HostPresenter, failure::Error> {
        let port_symbol_indexes = lv2::get_port_symbol_indexes(plugin_uri)?;
        let ears_indexes = lv2::get_ears_indexes(plugin_uri)?;
        
        Ok(Self {
            talker_id,
            session_presenter: session_presenter.clone(),
            port_symbol_indexes,
            ears_indexes,
        })
    }
}

impl luil::HostTrait for HostPresenter {
    fn configuration(&mut self) -> luil::HostConfiguration {
        luil::HostConfiguration {
            sample_rate: AudioFormat::sample_rate() as f64,
            support_touch: false,
            support_peak_protocol: false,
        }
    }
    fn urid_map(&mut self, uri: CString) -> lv2_raw::LV2Urid {
        lv2_handler::visit(|lv2_handler| {
            let urid = lv2_handler.features.urid(&uri);
            Ok(urid)
        }).unwrap()
    }
    fn urid_unmap(&mut self, urid: lv2_raw::LV2Urid) -> Option<CString> {
        lv2_handler::visit(|lv2_handler| {
            Ok(lv2_handler.features.uri(urid).map(|s| CString::from_str(s).unwrap()))
        }).unwrap()
    }
    fn index(&mut self, port_symbol: String) -> u32 {
        let idx = *self.port_symbol_indexes.get(&port_symbol).unwrap_or(&1);
        println!("Index : Port {} index = {}", port_symbol, idx);
        idx
    }
    fn notify(&mut self, message: String) {
        self.session_presenter.borrow().notify(Notification::Error(message));
    }
    fn write(&mut self, port_index: u32, buffer_size: u32, protocol: u32, buffer: Vec<u8>) {
        println!("Write port_index {}, buffer_size {}, protocol {}, buffer {:?}", port_index, buffer_size, protocol, buffer);

        if protocol == 0 {
            let val_ptr: *const f32 = buffer.as_ptr().cast();
            let value = unsafe {*val_ptr};

            self.session_presenter.borrow_mut().modify_band_volatly(
                &Operation::SetEarHumValue(
                self.talker_id, self.ears_indexes[port_index as usize], 0, 0, value));
            }
        else {
            self.session_presenter.borrow_mut().modify_band_volatly(
                &Operation::SetIndexedData(
                    self.talker_id, port_index as usize, protocol, buffer));
        }
    }
    fn read(&mut self) -> Option<Vec<(u32, u32, u32, Vec<u8>)>> {
        None
    }
}

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
        let luil = luil::Luil::new();

        Self {luil, pending_ui: None}
    }

    pub fn prepare_new_ui(&mut self, talker: &RTalker) {
        let bundle_uri = lv2::get_bundle_uri(&talker.model()).unwrap();

        self.pending_ui = Some(UiParameters {
            talker_id: talker.id(),
            plugin_uri: talker.model(),
            bundle_uri,
            instance_name: talker.name(),
        });
    }

    pub fn show_pending_ui(&mut self, session_presenter: &RSessionPresenter) -> Result<(), failure::Error> {

        if let Some(ui_param) = &self.pending_ui {
            let host: Box<HostPresenter> = Box::new(HostPresenter::new(ui_param.talker_id, &ui_param.plugin_uri, session_presenter)?);

            let mut plugin_handle = self.luil.create_plugin_handle(
                &ui_param.plugin_uri,
                Some(&ui_param.bundle_uri),
                &ui_param.instance_name,
                host
            )?;

            session_presenter.borrow_mut().add_plugin_handle(ui_param.talker_id, plugin_handle.take_ui_connector().unwrap());

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
