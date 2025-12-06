use std::ffi::CString;

pub struct Parameters {
    pub plugin_uri: String,
    pub c_plugin_uri: CString,
    pub c_ui_uri: CString,
    pub c_ui_type_uri: CString,
    pub c_ui_bundle_path: CString,
    pub c_ui_binary_path: CString,
    pub is_x11: bool,
    pub _has_show_interface: bool,
}
impl Parameters {
    pub fn new(plugin_uri: String, ui_uri: &str, ui_type_uri: &str, ui_bundle_path: &str, ui_binary_path: &str, is_x11: bool, has_show_interface: bool) -> Parameters {
        let c_plugin_uri = CString::new(plugin_uri.as_str()).unwrap();
        Self {
            plugin_uri,
            c_plugin_uri,
            c_ui_uri: CString::new(ui_uri).unwrap(),
            c_ui_type_uri: CString::new(ui_type_uri).unwrap(),
            c_ui_bundle_path: CString::new(ui_bundle_path).unwrap(),
            c_ui_binary_path: CString::new(ui_binary_path).unwrap(),
            is_x11,
            _has_show_interface: has_show_interface,
        }
    }
}
