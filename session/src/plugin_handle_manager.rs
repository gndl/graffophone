use std::cell::RefCell;
use std::rc::Rc;
use std::collections::HashMap;

use std::ffi::CString;

extern crate luil;

use luil::ui_connector::UiConnector;

use talker::lv2_handler;
use talker::talker::RTalker;
use talker::identifier::{Id, Identifiable};
use talker::audio_format::AudioFormat;

use crate::band::{Band, Operation};
use crate::util;

struct Report {
    operations: Vec<Operation>,
}
impl Report {
    pub fn new() -> Report {
        Self {operations: Vec::new()}
    }
}
type RReport = Rc<RefCell<Report>>;

struct HostPresenter {
    talker: RTalker,
    port_symbol_indexes: HashMap<String, u32>,
    ears_indexes: Vec<usize>,
    report: RReport,
}

impl HostPresenter {
    pub fn new(talker: RTalker, report: RReport) -> Result<HostPresenter, failure::Error> {
        let plugin_uri = talker.model();
        let port_symbol_indexes = lv2_handler::get_port_symbol_indexes(&plugin_uri)?;
        let ears_indexes = lv2_handler::get_ears_indexes(&plugin_uri)?;

        Ok(Self {
            talker,
            port_symbol_indexes,
            ears_indexes,
            report,
        })
    }
}

impl luil::HostTrait for HostPresenter {
    fn urid_map(&mut self, uri: CString) -> lv2_raw::LV2Urid {
        lv2_handler::urid_map(&uri)
    }
    fn urid_unmap(&mut self, urid: lv2_raw::LV2Urid) -> Option<CString> {
        lv2_handler::urid_unmap(urid)
    }
    fn index(&mut self, port_symbol: String) -> u32 {
        *self.port_symbol_indexes.get(&port_symbol).unwrap_or(&1)
    }
    fn notify(&mut self, _message: String) {
    }
    fn on_run(&mut self) {
    }
    fn write(&mut self, port_index: u32, protocol: u32, buffer: Vec<u8>) -> Option<Vec<(u32, u32, Vec<u8>)>> {
        let tkr_id = self.talker.id();

        if protocol == 0 {
            let val_ptr: *const f32 = buffer.as_ptr().cast();
            let value = unsafe {*val_ptr};
            let ear_idx = self.ears_indexes[port_index as usize];

            util::print_error(self.talker.set_ear_hum_value(ear_idx, 0, 0, value), ());

            self.report.borrow_mut().operations.push(Operation::SetEarHumValue(tkr_id, ear_idx, 0, 0, value));

            None
        }
        else {
            util::print_error(self.talker.set_indexed_data(port_index as usize, protocol, &buffer), ());

            self.report.borrow_mut().operations.push(Operation::SetIndexedData(tkr_id, port_index as usize, protocol, buffer));

            self.read()
        }
    }
    fn read(&mut self) -> Option<Vec<(u32, u32, Vec<u8>)>> {

        let evs = util::print_error(self.talker.read_ports_events(), Vec::default());

        if evs.is_empty() {
            None
        }
        else {
            Some(evs)
        }
    }
    fn subscribe(&mut self, port_index: u32, protocol: u32, features: Vec<CString>) -> u32 {
        println!("subscribe : port_index: {}, protocol: {}, features: {:?}", port_index, protocol, features);
        0
    }
    fn unsubscribe(&mut self, port_index: u32, protocol: u32, features: Vec<CString>) -> u32 {
        println!("unsubscribe : port_index: {}, protocol: {}, features: {:?}", port_index, protocol, features);
        0
    }
    fn state(&mut self) -> Option<String> {
        self.talker.state().unwrap_or(None)
    }
}

pub struct PluginHandleManager {
    luil: luil::Luil,
    handle_count: usize,
    report: RReport,
}

impl PluginHandleManager {
    pub fn new() -> PluginHandleManager {
        let luil = luil::Luil::new(luil::HostConfiguration {
            sample_rate: AudioFormat::sample_rate() as f64,
            support_touch: false,
            support_peak_protocol: false,
        });
        let report = Rc::new(RefCell::new(Report::new()));

        Self {luil, handle_count: 0, report}
    }

    pub fn add_plugin_handle(&mut self, talker_id: Id, ui_connector: UiConnector, band: &mut Band) -> Result<(), failure::Error> {

        let talker = band.fetch_talker(&talker_id)?;

        let host: Box<HostPresenter> = Box::new(HostPresenter::new(talker.clone(), self.report.clone())?);

        self.handle_count = self.luil.create_plugin_handle(
            host,
            &talker_id.to_string(),
            ui_connector
        )?;

       Ok(())
    }

    pub fn transmit(&mut self, live: bool) -> Result<(), failure::Error> {

        self.handle_count = util::print_error(self.luil.transmit(live), self.handle_count);
        Ok(())
    }

    pub fn has_handle(&self) -> bool {
        self.handle_count > 0
    }

    pub fn band_modifications_and_ui_count(&self) -> (Vec<Operation>, usize) {
        let mut operations: Vec<Operation> = Vec::new();

        operations.append(&mut self.report.borrow_mut().operations);

        (operations, self.luil.running_instances_count())
    }
}

