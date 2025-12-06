use lv2_raw::LV2Feature;
use lv2_sys::{LV2_Extension_Data_Feature, LV2UI_Feature_Handle, LV2UI_Touch, LV2UI_Resize};
use std::pin::Pin;
use std::ptr::NonNull;

pub mod options;
pub mod urid_map;

use crate::host::Host;
use crate::plugin_controller::PluginController;
use crate::xwindow::XWindow;

/// `Features` are used to provide functionality to plugins.

unsafe extern "C" fn do_ui_resize(
    handle: LV2UI_Feature_Handle,
    width: ::std::os::raw::c_int,
    height: ::std::os::raw::c_int,
) -> ::std::os::raw::c_int {
    let xwin: &mut XWindow = unsafe { &mut *(handle as *mut XWindow) };
    
    match xwin.resize(width as u32, height as u32) {
        Ok(()) => 0,
        Err(e) => {
            println!("{}", e);
            1
        }
    }
}

unsafe extern "C" fn do_touch(
    handle: LV2UI_Feature_Handle,
    port_index: u32,
    grabbed: bool,
) {
    let pc: &mut PluginController = unsafe { &mut *(handle as *mut PluginController) };

    match pc.touch(port_index, grabbed) {
        Ok(()) => (),
        Err(e) => println!("{}", e),
    }
}

//#[derive(Clone, Debug)]
pub struct Features {
    urid_map: Pin<Box<urid_map::UridMap>>,
    options: options::Options,
    min_block_length: usize,
    max_block_length: usize,
    _extention_data: LV2_Extension_Data_Feature,
    ui_resize: LV2UI_Resize,
    ui_touch: LV2UI_Touch,
    collection: Vec<LV2Feature>,
}

impl Features {
    pub fn new(
        min_block_length: usize,
        max_block_length: usize,
        window_title: &str,
        host: &mut Host,
        plugin_instance: Option<&livi::Instance>
    ) -> Features {
        let urid_map = urid_map::UridMap::new(host);
        let options = options::Options::new(&urid_map);

        let mut features = Features {
            urid_map,
            options,
            min_block_length,
            max_block_length,
            _extention_data: LV2_Extension_Data_Feature {
                data_access: None,
            },
            ui_resize: LV2UI_Resize {
                handle: std::ptr::null_mut(),
                ui_resize: Some(do_ui_resize)                
            },
            ui_touch: LV2UI_Touch {
                handle: host.plugin_controller_ptr(),
                touch: Some(do_touch)                
            },
            collection: Vec::new(),
        };

        features.collection.push(features.urid_map.urid_map_feature());
        features.collection.push(features.urid_map.urid_unmap_feature());

        features.collection.push(LV2Feature {
            uri: lv2_sys::LV2_BUF_SIZE__boundedBlockLength.as_ptr().cast(),
            data: std::ptr::null_mut(),
        });
        features.collection.push(LV2Feature {
            uri: lv2_sys::LV2_UI__idleInterface.as_ptr().cast(),
            data: std::ptr::null_mut(),
        });
        features.collection.push(LV2Feature {
            uri: lv2_sys::LV2_UI__portSubscribe.as_ptr().cast(),
            data: std::ptr::null_mut(),
        });

        if let Some(plugin_instance) = plugin_instance {
            let lilv_instance = plugin_instance.raw().instance();
/*
        if let Some(descriptor) = lilv_instance.descriptor() {
            // Will work when livi will use lv2_raw 0.3
//            features.extention_data.data_access = Some(descriptor.extension_data);
        }
        let extention_data = NonNull::from(&features.extention_data);

        features.collection.push(LV2Feature {
            uri: lv2_sys::LV2_DATA_ACCESS_URI.as_ptr().cast(),
            data: extention_data.as_ptr().cast(),
        });
*/
            features.collection.push(LV2Feature {
                uri: lv2_sys::LV2_INSTANCE_ACCESS_URI.as_ptr().cast(),
                data: lilv_instance.handle(),
            });
        }

        // Options
        features.options.set_int_option(
            &features.urid_map,
            lv2_sys::LV2_BUF_SIZE__minBlockLength,
            min_block_length as i32,
        );
        features.options.set_int_option(
            &features.urid_map,
            lv2_sys::LV2_BUF_SIZE__maxBlockLength,
            max_block_length as i32,
        );
        features.options.set_string_option(
            &features.urid_map,
            lv2_sys::LV2_UI__windowTitle,
            window_title,
        );
        features.options.set_float_option(
            &features.urid_map,
            lv2_sys::LV2_PARAMETERS__sampleRate,
            host.sample_rate() as f32,
        );
        features.options.set_float_option(
            &features.urid_map,
            lv2_sys::LV2_UI__scaleFactor,
            1.,
        );

        features.collection.push(features.options.feature());

        // Touch
        if host.support_touch() {
            let ui_touch_ptr = NonNull::from(&features.ui_touch);

            features.collection.push(LV2Feature {
                uri: lv2_sys::LV2_UI__touch.as_ptr().cast(),
                data: ui_touch_ptr.as_ptr().cast(),
            });
        }

        // Peak protocol
        if host.support_peak_protocol() {
            features.collection.push(LV2Feature {
                uri: lv2_sys::LV2_UI__peakProtocol.as_ptr().cast(),
                data: std::ptr::null_mut(),
            });
        }

        features
    }

    pub fn add_xwindow(&mut self, xwindow: &XWindow) {
        self.collection.push(LV2Feature {
            uri: lv2_sys::LV2_UI__parent.as_ptr().cast(),
            data: xwindow.id() as *mut libc::c_void,
        });

        let xwindow_ptr = NonNull::from(xwindow);
        self.ui_resize.handle = xwindow_ptr.as_ptr().cast();

        let ui_resize_ptr = NonNull::from(&self.ui_resize);

        self.collection.push(LV2Feature {
            uri: lv2_sys::LV2_UI__resize.as_ptr().cast(),
            data: ui_resize_ptr.as_ptr().cast(),
        });
    }

    /// Iterate over all the LV2 features.
    pub fn iter_features<'a>(&'a self) -> impl Iterator<Item = &'a LV2Feature> {
        self.collection.iter()
    }
}

impl std::fmt::Debug for Features {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Features")
            .field("urid_map", &self.urid_map)
            .field("options", &self.options)
            .field("bounded_block_length", &"__uri__")
            .field("min_block_length", &self.min_block_length)
            .field("max_block_length", &self.max_block_length)
            .finish()
    }
}
