
use std::ffi::CString;
use std::os::raw::c_char;

extern crate suil_sys;

use crate::xwindow::XWindow;

pub fn init() {
    let args = std::env::args().map(|arg| CString::new(arg).unwrap() ).collect::<Vec<CString>>();
    let mut c_args = args.iter().map(|arg| arg.clone().into_raw()).collect::<Vec<*mut c_char>>();

    let mut argc = c_args.len() as i32;
    let mut argv = c_args.as_mut_ptr();

    unsafe {
        suil_sys::suil_init((&mut argc) as *mut i32, (&mut argv) as *mut *mut *mut i8, suil_sys::SuilArg_SUIL_ARG_NONE);
    }
}

use lv2_raw::LV2Feature;

use crate::features;
use crate::host;
use crate::plugin_ui;

pub struct Instance {
    instance: *mut suil_sys::SuilInstance,
    instance_handle: *mut std::ffi::c_void,
    show_interface_ext_data: *const std::ffi::c_void,
    idle_interface_ext_data: *const std::ffi::c_void,
}

impl Instance {
    pub fn new(
        host: &mut host::Host,
        features: &features::Features,
        plugin_ui: &plugin_ui::Parameters,
    ) -> Result<Instance, failure::Error> {
        let host_type_uri = std::ptr::null();

        let plugin_uri= plugin_ui.c_plugin_uri.as_ptr() as *const i8;
        let ui_uri = plugin_ui.c_ui_uri.as_ptr() as *const i8;
        let ui_type_uri = plugin_ui.c_ui_type_uri.as_ptr() as *const i8;
        let ui_bundle_path = plugin_ui.c_ui_bundle_path.as_ptr() as *const i8;
        let ui_binary_path = plugin_ui.c_ui_binary_path.as_ptr() as *const i8;


        let lv2_features: Vec<*const LV2Feature> = features.iter_features()
            .map(|f| f as *const LV2Feature)
            .chain(std::iter::once(std::ptr::null()))
            .collect();

        let instance = unsafe {suil_sys::suil_instance_new(host.suil_host(),
            host.plugin_controller_ptr(),
            host_type_uri,
            plugin_uri,
            ui_uri,
            ui_type_uri,
            ui_bundle_path,
            ui_binary_path,
            lv2_features.as_ptr())
        };

        if instance.is_null() {
            return Err(failure::err_msg(format!("Luil failed to instantiate UI {}.", plugin_ui.plugin_uri)));
        }

        // Acquir instance handle
        let instance_handle = unsafe {suil_sys::suil_instance_get_handle(instance)};

        if instance_handle.is_null() {
            return Err(failure::err_msg(format!("Luil failed to get instance handle.")));
        }

        // Options interface
        let options_interface_ext_data = unsafe {suil_sys::suil_instance_extension_data(instance, lv2_sys::LV2_OPTIONS__interface.as_ptr().cast())};

        if !options_interface_ext_data.is_null() {
            let _options_interface: &mut lv2_sys::LV2_Options_Interface = unsafe { &mut *(options_interface_ext_data as *mut lv2_sys::LV2_Options_Interface) };
        }
        // Show interface
        let show_interface_ext_data = unsafe {suil_sys::suil_instance_extension_data(instance, lv2_sys::LV2_UI__showInterface.as_ptr().cast())};

        if !show_interface_ext_data.is_null() {
            let show_interface: *const lv2_sys::LV2UI_Show_Interface = show_interface_ext_data.cast();
            let oshow = unsafe {(*show_interface).show};
            
            if let Some(show) = oshow {
                let cr = unsafe {show(instance_handle)};
                
                if cr != 0 {
                    return Err(failure::err_msg(format!("Luil failed to show interface : {}.", cr)));
                }
            }
        }

        // Idle instance
        let idle_interface_ext_data = unsafe {suil_sys::suil_instance_extension_data(instance, lv2_sys::LV2_UI__idleInterface.as_ptr().cast())};

        Ok(Self {
            instance,
            instance_handle,
            show_interface_ext_data,
            idle_interface_ext_data,
        })
    }

    pub fn run(&self, host: &mut host::Host, xwindow: &Option<XWindow>) -> Result<bool, failure::Error> {

        let oidle = if self.idle_interface_ext_data.is_null() {
            None
        }
        else {
            let idle_interface: *const lv2_sys::LV2UI_Idle_Interface = self.idle_interface_ext_data.cast();
            unsafe {(*idle_interface).idle}
        };

        host.receive_notifications(|port_index, buffer_size, protocol, buffer| {
                unsafe { suil_sys::suil_instance_port_event(self.instance, port_index, buffer_size, protocol, buffer.as_ptr().cast()) }
            })?;

        if let Some(idle) = oidle {
            if 0 != unsafe {idle(self.instance_handle)} {
                return Ok(false);
            }
        }

        if let Some(xwin) = &xwindow {
            if !xwin.idle()? {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

impl Drop for Instance {
    fn drop(&mut self) {

        if !self.show_interface_ext_data.is_null() {
            let show_interface: *const lv2_sys::LV2UI_Show_Interface = self.show_interface_ext_data.cast();
            let ohide = unsafe {(*show_interface).hide};
            
            if let Some(hide) = ohide {
                let _ = unsafe {hide(self.instance_handle)};
            }
        }

        unsafe{
            suil_sys::suil_instance_free(self.instance);
        }
    }
}
