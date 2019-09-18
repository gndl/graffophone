#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::ffi::c_void;
use std::os::raw::c_char;
use std::ptr;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[inline(always)] // From lilv.h
pub unsafe fn lilv_instance_activate(instance: *const LilvInstance) {
    let activate = (*(*instance).lv2_descriptor).activate;
    if let Some(activate) = activate {
        activate((*instance).lv2_handle)
    }
}

#[inline(always)] // From lilv.h
pub unsafe fn lilv_instance_deactivate(instance: *const LilvInstance) {
    let deactivate = (*(*instance).lv2_descriptor).deactivate;
    if let Some(deactivate) = deactivate {
        deactivate((*instance).lv2_handle)
    }
}

#[inline(always)]
pub unsafe fn lilv_instance_connect_port(instance: *const LilvInstance, port_index: u32, location: *mut c_void) {
    let connect_port = (*(*instance).lv2_descriptor).connect_port;

    if let Some(connect_port) = connect_port {
        connect_port((*instance).lv2_handle, port_index, location)
    }
}

#[inline(always)]
pub unsafe fn lilv_instance_run(instance: *const LilvInstance, sample_count: u32) {
    let run = (*(*instance).lv2_descriptor).run;

    if let Some(run) = run {
        run((*instance).lv2_handle, sample_count)
    }
}

#[inline(always)]
pub unsafe fn lilv_instance_get_extension_data(instance: *const LilvInstance, uri: *const c_char) -> *const c_void {
    let extension_data = (*(*instance).lv2_descriptor).extension_data;

    if let Some(extension_data) = extension_data {
        extension_data(uri)
    } else {
        ptr::null()
    }
}

#[inline(always)]
pub unsafe fn lilv_instance_get_uri(instance: *const LilvInstance) -> *const c_char {
    (*(*instance).lv2_descriptor).URI
}

#[inline(always)]
pub unsafe fn lilv_instance_get_descriptor(instance: *const LilvInstance) -> *const LV2_Descriptor {
    (*instance).lv2_descriptor
}

#[inline(always)]
pub unsafe fn lilv_instance_get_handle(instance: *const LilvInstance) -> LV2_Handle {
    (*instance).lv2_handle
}
