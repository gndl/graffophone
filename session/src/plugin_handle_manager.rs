use std::cell::RefCell;
use std::rc::Rc;
use std::collections::HashMap;

use std::ffi::CString;
use std::str::FromStr;

extern crate luil;

use talker::lv2_handler;
use talker::talker::RTalker;
use talker::identifier::{Id, Identifiable, Index};
use talker::audio_format::AudioFormat;

use crate::band::Band;
use crate::talkers::lv2;

struct Report {
    modifications: Vec<(Id, Index, Index, Index, f32)>,
}
impl Report {
    pub fn new() -> Report {
        Self {modifications: Vec::new()}
    }
}
type RReport = Rc<RefCell<Report>>;

struct PluginPresenter {
    talker: RTalker,
    port_symbol_indexes: HashMap<String, u32>,
    ears_indexes: Vec<usize>,
    report: RReport,
}

impl PluginPresenter {
    pub fn new(talker: RTalker, report: RReport) -> Result<PluginPresenter, failure::Error> {
        let plugin_uri = talker.model();
        let port_symbol_indexes = lv2::get_port_symbol_indexes(&plugin_uri)?;
        let ears_indexes = lv2::get_ears_indexes(&plugin_uri)?;

        Ok(Self {
            talker,
            port_symbol_indexes,
            ears_indexes,
            report,
        })
    }
}

impl luil::PluginTrait for PluginPresenter {
    fn urid_map(&mut self, uri: CString) -> lv2_raw::LV2Urid {
        lv2_handler::visit(|lv2_handler| {
            let urid = lv2_handler.features.urid(&uri);
            println!("urid_map {:?} -> {}", uri, urid);
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
    fn notify(&mut self, _message: String) {
    }
    fn on_run(&mut self) {
    }
    fn write(&mut self, port_index: u32, buffer_size: u32, protocol: u32, buffer: Vec<u8>) {
        let _ = println!("Write port_index {}, buffer_size {}, protocol {}, buffer {:?}", port_index, buffer_size, protocol, buffer);
        self.talker.deactivate();

        if protocol == 0 {
            let val_ptr: *const f32 = buffer.as_ptr().cast();
            let value = unsafe {*val_ptr};
            let ear_idx = self.ears_indexes[port_index as usize];

            let _ = self.talker.set_ear_hum_value(ear_idx, 0, 0, value);

            let tkr_id = self.talker.id();

            let mut report = self.report.borrow_mut();
            let mut modif_idx = usize::MAX;

            for (i, (ti, ei, _, _, _)) in report.modifications.iter().enumerate() {

                if *ti == tkr_id && *ei == ear_idx {
                    modif_idx = i;
                    break;
                }
            }

            if modif_idx < report.modifications.len() {
                report.modifications[modif_idx] = (tkr_id, ear_idx, 0, 0, value);
            }
            else {
                report.modifications.push((tkr_id, ear_idx, 0, 0, value));
            }
        }
        else {
            let _ = self.talker.set_indexed_data(port_index as usize, protocol, &buffer);
        }
        self.talker.activate();
    }
    fn read(&mut self) -> Option<Vec<(u32, u32, u32, Vec<u8>)>> {
//        self.session_presenter.borrow().read_port_events(self.talker_id)
        None
    }
}

pub struct PluginHandleManager {
    luil: luil::Luil,
    handle_count: usize,
    report: RReport,
}

impl PluginHandleManager {
    pub fn new() -> PluginHandleManager {
        let host_configuration = luil::HostConfiguration {
            sample_rate: AudioFormat::sample_rate() as f64,
            support_touch: false,
            support_peak_protocol: false,
        };

        let luil = luil::Luil::new(host_configuration);
        let report = Rc::new(RefCell::new(Report::new()));

        Self {luil, handle_count: 0, report}
    }

    pub fn add_plugin_handle(&mut self, talker_id: Id, band: &mut Band) -> Result<(), failure::Error> {

        let talker = band.fetch_talker(&talker_id)?;

        let instance_id = talker.id().to_string();
        let plugin_uri = talker.model();
        let bundle_uri = lv2::get_bundle_uri(&talker.model()).unwrap();
        let instance_name = talker.name();

        let plugin: Box<PluginPresenter> = Box::new(PluginPresenter::new(talker.clone(), self.report.clone())?);

        self.handle_count = self.luil.launch_plugin_ui(
            &plugin_uri,
            &bundle_uri,
            &instance_id,
            &instance_name,
            plugin,
            |v| talker.lv2_instance(v)
        )?;
        Ok(())
    }

    pub fn run(&mut self) -> Result<(), failure::Error> {

        self.handle_count = self.luil.run()?;
        Ok(())
    }

    pub fn has_handle(&self) -> bool {
        self.handle_count > 0
    }

    pub fn band_modifications_and_ui_count(&self) -> (Vec<(Id, Index, Index, Index, f32)>, usize) {
        let mut modifications: Vec<(Id, Index, Index, Index, f32)> = Vec::new();

        modifications.append(&mut self.report.borrow_mut().modifications);

        (modifications, self.luil.running_instances_count())
    }
}

