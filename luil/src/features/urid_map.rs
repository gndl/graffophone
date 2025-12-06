use lv2_raw::LV2Feature;
use std::ffi::{CStr, CString};
use std::pin::Pin;
use std::ptr::NonNull;
use std::ffi::c_void;

use crate::host::Host;
use crate::plugin_controller::PluginController;

pub struct MapHandle {
    uris: Vec<CString>,
    controller: *mut c_void,
}

/// # Safety
/// Dereference to `uri_ptr` may be unsafe.
extern "C" fn do_map(handle: lv2_raw::LV2UridMapHandle, uri_ptr: *const i8) -> lv2_raw::LV2Urid {
    let handle = handle as *mut MapHandle;
    let map_handle = unsafe { &mut *handle };

    let pc: &mut PluginController = unsafe { &mut *(map_handle.controller as *mut PluginController) };

    let id = match pc.urid_map(uri_ptr) {
        Ok(v) => v,
        Err(e) => {
            println!("{}", e);
            0
        }
    };
    id
}

extern "C" fn do_unmap(handle: lv2_sys::LV2_URID_Map_Handle, urid: lv2_raw::LV2Urid) -> *const i8 {
    let handle = handle as *mut MapHandle;
    let map_handle = unsafe { &mut *handle };

    let pc: &mut PluginController = unsafe { &mut *(map_handle.controller as *mut PluginController) };

    match pc.urid_unmap(urid) {
        Ok(ouri) => {
            match ouri {
                Some(uri) => {
                    let ptr = uri.as_ptr();
                    map_handle.uris.push(uri);
                    ptr
                }
                None => std::ptr::null(),
            }
        },
        Err(_) => std::ptr::null(),
    }
}

pub struct UridMap {
    map_handle: MapHandle,
    map_data: lv2_raw::LV2UridMap,
    unmap_data: lv2_sys::LV2_URID_Unmap,
    _pin: std::marker::PhantomPinned,
}

unsafe impl Send for UridMap {}

impl UridMap {
    pub fn new(host: &mut Host) -> Pin<Box<UridMap>> {
        let map_handle = MapHandle {
            uris: Vec::new(),
            controller: host.plugin_controller_ptr(),
        };
        let mut urid_map = Box::pin(UridMap {
            map_handle,
            map_data: lv2_raw::LV2UridMap {
                handle: std::ptr::null_mut(),
                map: do_map,
            },
            unmap_data: lv2_sys::LV2_URID_Unmap {
                handle: std::ptr::null_mut(),
                unmap: Some(do_unmap),
            },
            _pin: std::marker::PhantomPinned,
        });
        let map_impl_ptr = NonNull::from(&urid_map.map_handle);
        unsafe {
            let mut_ref_pin: Pin<&mut UridMap> = Pin::as_mut(&mut urid_map);
            let mut_ref = Pin::get_unchecked_mut(mut_ref_pin);
            mut_ref.map_data.handle = map_impl_ptr.as_ptr().cast();
            mut_ref.unmap_data.handle = map_impl_ptr.as_ptr().cast();
        }
        urid_map
    }

    pub fn map(&self, uri: &CStr) -> lv2_raw::LV2Urid {
        do_map(self.map_data.handle, uri.as_ptr())
    }

    pub fn urid_map_feature(&self) -> LV2Feature {
        let map_data_ptr = NonNull::from(&self.map_data);
        LV2Feature {
            uri: lv2_sys::LV2_URID__map.as_ptr().cast(),
            data: map_data_ptr.as_ptr().cast(),
        }
    }

    pub fn urid_unmap_feature(&self) -> LV2Feature {
        let unmap_data_ptr = NonNull::from(&self.unmap_data);
        LV2Feature {
            uri: lv2_sys::LV2_URID__unmap.as_ptr().cast(),
            data: unmap_data_ptr.as_ptr().cast(),
        }
    }
}

impl std::fmt::Debug for UridMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UridMap").field("uris", &self.map_handle.uris).finish()
    }
}
