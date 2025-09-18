use std::collections::HashMap;
use std::ffi::{c_void, CString};
use std::os::raw::c_char;

use lv2_raw::LV2Feature;

extern crate suil_sys;

use talker::lv2_handler;
use talker::talker::RTalker;
use talker::identifier::{Id, Identifiable};

use crate::session_presenter::RSessionPresenter;

pub fn init() {
    let args = std::env::args().map(|arg| CString::new(arg).unwrap() ).collect::<Vec<CString>>();
    let mut c_args = args.iter().map(|arg| arg.clone().into_raw()).collect::<Vec<*mut c_char>>();
    
    let mut argc = c_args.len() as i32;
    let mut argv = c_args.as_mut_ptr();
    
    unsafe {
        suil_sys::suil_init((&mut argc) as *mut i32, (&mut argv) as *mut *mut *mut i8, suil_sys::SuilArg_SUIL_ARG_NONE);
    }
}
struct PluginPresenter {
    _talker: RTalker,
    session_presenter: RSessionPresenter,
}

unsafe extern "C" fn write_func(
    controller: suil_sys::SuilController,
    port_index: u32,
    buffer_size: u32,
    protocol: u32,
    _buffer: *const ::std::os::raw::c_void,
) {
    let plugin_presenter: &mut PluginPresenter = unsafe { &mut *(controller as *mut PluginPresenter) };
    
    let state = plugin_presenter.session_presenter.borrow_mut().check_state().to_string();
    println!("port_index: {}, buffer_size: {}, protocol: {}, state: {}", port_index, buffer_size, protocol, state);
}
unsafe extern "C" fn index_func(
    controller: suil_sys::SuilController,
    _port_symbol: *const ::std::os::raw::c_char,
) -> u32 {
   let _plugin_presenter: &mut PluginPresenter = unsafe { &mut *(controller as *mut PluginPresenter) };
0
}
unsafe extern "C" fn subscribe_func(
    _controller: suil_sys::SuilController,
    _port_index: u32,
    _protocol: u32,
    _features: *const *const lv2_raw::LV2Feature,
) -> u32 {0}
unsafe extern "C" fn unsubscribe_func(
    _controller: suil_sys::SuilController,
    _port_index: u32,
    _protocol: u32,
    _features: *const *const lv2_raw::LV2Feature,
) -> u32 {0}

struct InstanceHandler {
    instance: *mut suil_sys::SuilInstance,
    _plugin_presenter: PluginPresenter,
}

pub struct Manager {
    suil_host: *mut suil_sys::SuilHost,
    instances: HashMap<Id, InstanceHandler>,
    features: Vec<lv2_raw::LV2Feature>,
}

impl Manager {
    pub fn new() -> Manager {
        let suil_host = unsafe {suil_sys::suil_host_new(Some(write_func),
            Some(index_func),
            Some(subscribe_func),
            Some(unsubscribe_func))
        };

        let bounded_block_length_feature = lv2_raw::LV2Feature {
            uri: lv2_sys::LV2_BUF_SIZE__boundedBlockLength.as_ptr().cast(),
            data: std::ptr::null_mut(),
        };

        let features = vec![bounded_block_length_feature];

        Self {
            suil_host,
            instances: HashMap::new(),
            features,
        }
    }

    pub fn show(&mut self, talker: &RTalker, session_presenter: &RSessionPresenter) -> Result<(), failure::Error> {
        lv2_handler::visit(|lv2_handler| {
            let plugin_uri = &talker.model();
                
            match lv2_handler.world.plugin_by_uri(plugin_uri) {
                Some(plugin) => {
                    let plugin_uri_node = plugin.raw().uri();
                    let plugin_uri = plugin_uri_node.as_uri().expect("Missing plugin uri.");

                    let uis = plugin.raw().uis().expect("Missing plugin UIs.");
                    let ui = uis.iter().next().expect("Missing plugin UI.");
                    let ui_type_uri_node = ui.classes().iter().next().expect("Missing UI classe.");
                    let ui_type_uri = ui_type_uri_node.as_uri().unwrap().as_ptr() as *const i8;
                    let ui_uri = ui.uri();
                    let ui_binary_uri = ui.binary_uri().expect("Missing binary uri.");
                    let ui_binary_path = ui_binary_uri.as_str().expect("Missing binary uri str.").get(7..).unwrap();
                    // let (ui_binary_hostname, ui_binary_path) = binary_uri.path().expect("Missing binary path.");
                    let ui_bundle_uri = ui.bundle_uri().expect("Missing bundle uri.");
                    let ui_bundle_path = ui_bundle_uri.as_str().expect("Missing bundle uri str.").get(7..).unwrap();
                    // let (ui_bundle_hostname, ui_bundle_path) = ui.bundle_uri().map_or(None, |u| u.path()).expect("Missing bundle path.");

                    let mut plugin_presenter = PluginPresenter {
                        _talker: talker.clone(),
                        session_presenter: session_presenter.clone(),
                    };

                    let plugin_presenter_ptr: *mut c_void = &mut plugin_presenter as *mut _ as *mut c_void;

                    let host_type_uri = lv2_handler::LV2_UI_HOST_TYPE_URI.as_ptr() as *const i8;
                    let plugin_uri = plugin_uri.as_ptr() as *const i8;
                    let ui_uri = ui_uri.as_uri().unwrap().as_ptr() as *const i8;
                    let ui_bundle_path = ui_bundle_path.as_ptr() as *const i8;
                    let ui_binary_path = ui_binary_path.as_ptr() as *const i8;
            
                    let features: Vec<*const LV2Feature> = self.features
                        .iter()
                        .map(|f| f as *const LV2Feature)
                        .chain(std::iter::once(std::ptr::null()))
                        .collect();

                    let instance = unsafe {suil_sys::suil_instance_new(self.suil_host, 
                        plugin_presenter_ptr,
                        host_type_uri,
                        plugin_uri,
                        ui_uri,
                        ui_type_uri,
                        ui_bundle_path,
                        ui_binary_path,
                        features.as_ptr())
                    };

                    self.instances.insert(talker.id(), InstanceHandler{instance, _plugin_presenter: plugin_presenter});

                    Ok(())
                },
                None => Err(failure::err_msg(format!("LV2 plugin {} not found.", plugin_uri))),
            }
        })
    }
}

impl Drop for Manager {
    fn drop(&mut self) {
        for inst in self.instances.values() {
            unsafe{suil_sys::suil_instance_free(inst.instance);}
        }
        unsafe{suil_sys::suil_host_free(self.suil_host);}
    }
}
