use std::ffi::c_void;

extern crate suil_sys;

use crate::plugin_controller::PluginController;
use crate::Plugin;

unsafe extern "C" fn write_func(
    controller: suil_sys::SuilController,
    port_index: u32,
    buffer_size: u32,
    protocol: u32,
    buffer: *const ::std::os::raw::c_void,
) {
    let pc: &mut PluginController = unsafe { &mut *(controller as *mut PluginController) };

    match pc.write(port_index, buffer_size, protocol, buffer) {
        Ok(()) => (),
        Err(e) => println!("{}", e),
    }
}

unsafe extern "C" fn index_func(
    controller: suil_sys::SuilController,
    port_symbol: *const ::std::os::raw::c_char,
) -> u32 {
    let pc: &mut PluginController = unsafe { &mut *(controller as *mut PluginController) };

    match pc.index(port_symbol) {
        Ok(v) => v,
        Err(e) => {
            println!("{}", e);
            0
        }
    }
}

unsafe extern "C" fn subscribe_func(
    controller: suil_sys::SuilController,
    port_index: u32,
    protocol: u32,
    features: *const *const lv2_raw::LV2Feature,
) -> u32 {
    let pc: &mut PluginController = unsafe { &mut *(controller as *mut PluginController) };

    match pc.subscribe(port_index, protocol, features) {
        Ok(v) => v,
        Err(e) => {
            println!("{}", e);
            0
        }
    }
}

unsafe extern "C" fn unsubscribe_func(
    controller: suil_sys::SuilController,
    port_index: u32,
    protocol: u32,
    features: *const *const lv2_raw::LV2Feature,
) -> u32 {
    let pc: &mut PluginController = unsafe { &mut *(controller as *mut PluginController) };

    match pc.unsubscribe(port_index, protocol, features) {
        Ok(v) => v,
        Err(e) => {
            println!("{}", e);
            0
        }
    }
}


#[derive(PartialEq, Clone, Copy)]
pub struct HostConfiguration {
    pub sample_rate: f64,
    pub support_touch: bool,
    pub support_peak_protocol: bool,
}

pub struct Host {
    pub configuration: HostConfiguration,
    plugin_controller: PluginController,
    suil_host: *mut suil_sys::SuilHost,
}

impl Host {
    pub fn new(configuration: HostConfiguration, plugin: Plugin) -> Result<Host, failure::Error> {

        let plugin_controller = PluginController::new(plugin)?;

        let suil_host = unsafe {suil_sys::suil_host_new(Some(write_func),
            Some(index_func),
            Some(subscribe_func),
            Some(unsubscribe_func))
        };

        Ok(Self {configuration, plugin_controller, suil_host})
    }

    pub fn suil_host(&mut self) -> *mut suil_sys::SuilHost {
        self.suil_host
    }

    pub fn plugin_controller_ptr(&mut self) -> *mut c_void {
        &mut self.plugin_controller as *mut _ as *mut c_void
    }

    pub fn sample_rate(&self) -> f64 {
        self.configuration.sample_rate
    }

    pub fn support_touch(&self) -> bool {
        self.configuration.support_touch
    }

    pub fn support_peak_protocol(&self) -> bool {
        self.configuration.support_peak_protocol
    }

    pub fn receive_notifications<F>(&mut self, mut port_event: F) -> Result<(), failure::Error>
    where
        F: FnMut(u32, u32, u32, Vec<u8>),
    {
        if let Some(port_events) = self.plugin_controller.notify_port_events()? {

            for (port_index, buffer_size, protocol, buffer) in port_events {
                println!("receive_notifications port_index {}, buffer_size {}, protocol {}, buffer :\n{:?}", port_index, buffer_size, protocol, buffer);
                port_event(port_index, buffer_size, protocol, buffer);
            }
        }
        Ok(())
    }
}
impl Drop for Host {
    fn drop(&mut self) {
        unsafe{
            suil_sys::suil_host_free(self.suil_host);
        }
    }
}
