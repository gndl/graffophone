use lv2_raw::{LV2Feature, LV2Urid};
use lv2_sys::LV2_Options_Option;
use std::convert::TryFrom;
use std::{collections::HashMap, ffi::{CString, CStr}};

const EMPTY_OPTION: LV2_Options_Option = LV2_Options_Option {
    context: 0,
    subject: 0,
    key: 0,
    size: 0,
    type_: 0,
    value: std::ptr::null(),
};

pub struct Options {
    data: Vec<lv2_sys::LV2_Options_Option>,
    atom_float_urid: LV2Urid,
    float_values: HashMap<LV2Urid, Box<f32>>,
    atom_int_urid: LV2Urid,
    int_values: HashMap<LV2Urid, Box<i32>>,
    atom_string_urid: LV2Urid,
    string_values: HashMap<LV2Urid, CString>,
}

unsafe impl Send for Options {}

impl Options {
    pub fn new(urid_map: &crate::features::urid_map::UridMap) -> Options {
        Self {
            data: vec![EMPTY_OPTION],
            atom_float_urid: urid_map.map(CStr::from_bytes_with_nul(lv2_sys::LV2_ATOM__Float).unwrap()),
            float_values: HashMap::new(),
            atom_int_urid: urid_map.map(CStr::from_bytes_with_nul(lv2_sys::LV2_ATOM__Int).unwrap()),
            int_values: HashMap::new(),
            atom_string_urid: urid_map.map(CStr::from_bytes_with_nul(lv2_sys::LV2_ATOM__String).unwrap()),
            string_values: HashMap::new(),
        }
    }

    pub fn set_float_option(
        &mut self,
        urid_map: &crate::features::urid_map::UridMap,
        uri: &[u8],
        value: f32,
    ) {
        let key = urid_map.map(CStr::from_bytes_with_nul(uri).unwrap());
        let size = u32::try_from(std::mem::size_of::<f32>()).expect("Size exceeded capacity of u32.");
        let type_ = self.atom_float_urid;
        let value = Box::new(value);
        let value_ptr = value.as_ref() as *const f32;

        self.float_values.insert(key, value);

        self.push_option(LV2_Options_Option {
            context: 0,
            subject: 0,
            key,
            size,
            type_,
            value: value_ptr.cast(),
        });
    }

    pub fn set_int_option(
        &mut self,
        urid_map: &crate::features::urid_map::UridMap,
        uri: &[u8],
        value: i32,
    ) {
        let key = urid_map.map(CStr::from_bytes_with_nul(uri).unwrap());
        let size = u32::try_from(std::mem::size_of::<i32>()).expect("Size exceeded capacity of u32.");
        let type_ = self.atom_int_urid;
        let value = Box::new(value);
        let value_ptr = value.as_ref() as *const i32;

        self.int_values.insert(key, value);

        self.push_option(LV2_Options_Option {
            context: 0,
            subject: 0,
            key,
            size,
            type_,
            value: value_ptr.cast(),
        });
    }

    pub fn set_string_option(
        &mut self,
        urid_map: &crate::features::urid_map::UridMap,
        uri: &[u8],
        value: &str,
    ) {
        let c_value = CString::new(value).unwrap();

        let key = urid_map.map(CStr::from_bytes_with_nul(uri).unwrap());
        let size = value.len() as u32 + 1;
        let type_ = self.atom_string_urid;
        let value_ptr = c_value.as_ptr();

        self.string_values.insert(key, c_value);

        self.push_option(LV2_Options_Option {
            context: 0,
            subject: 0,
            key,
            size,
            type_,
            value: value_ptr.cast(),
        });
    }

    fn push_option(&mut self, option: LV2_Options_Option) {
        self.data.pop(); // Remove the last `EMPTY_OPTION`.
        self.data.push(option);
        self.data.push(EMPTY_OPTION);
    }

    pub fn feature(&mut self) -> LV2Feature {
        LV2Feature {
            uri: lv2_sys::LV2_OPTIONS__options.as_ptr().cast(),
            data: self.data.as_mut_ptr().cast(),
        }
    }
}

impl std::fmt::Debug for Options {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Options")
            .field("data", &self.data)
            .field("values", &self.int_values)
            .field("feature", &"__feature__")
            .finish()
    }
}
