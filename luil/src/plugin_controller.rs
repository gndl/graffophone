use std::io::Write;
use std::ffi::CStr;
use std::ffi::CString;

use crate::Plugin;

pub struct PluginController {
    plugin: Plugin,
}
impl PluginController {
    pub fn new(plugin: Plugin) -> Result<PluginController, failure::Error> {

        Ok(Self {plugin})
    }

    pub fn urid_map(&mut self, uri_ptr: *const core::ffi::c_char) -> Result<lv2_raw::LV2Urid, failure::Error> {
        let uri: &CStr = unsafe { CStr::from_ptr(uri_ptr) };

        let id = self.plugin.urid_map(uri.to_owned());
        Ok(id)
    }

    pub fn urid_unmap(&mut self, urid: lv2_raw::LV2Urid) -> Result<Option<CString>, failure::Error> {

        let uri = self.plugin.urid_unmap(urid);
        Ok(uri)
    }

    pub fn index(
        &mut self,
        port_symbol: *const core::ffi::c_char,
    ) -> Result<u32, failure::Error> {
        let ps = unsafe { CStr::from_ptr(port_symbol) };
        let idx = self.plugin.index(ps.to_string_lossy().to_string());
        Ok(idx)
    }

    pub fn notify(&mut self, message: String) -> Result<(), failure::Error> {
        self.plugin.notify(message);
        Ok(())
    }

    fn read(&mut self) -> Option<Vec<(u32, u32, u32, Vec<u8>)>> { None }

    pub fn write(
        &mut self,
        port_index: u32,
        buffer_size: u32,
        protocol: u32,
        buffer: *const ::std::os::raw::c_void,
    ) -> Result<(), failure::Error> {

        let slice = unsafe { std::slice::from_raw_parts(buffer as *const u8, buffer_size as usize) };
        let mut buf = Vec::with_capacity(buffer_size as usize);
        buf.write_all(slice)?;

        self.plugin.write(port_index, buffer_size, protocol, buf);

        Ok(())
    }

    pub fn subscribe(
        &mut self,
        port_index: u32,
        protocol: u32,
        _features: *const *const lv2_raw::LV2Feature,
    ) -> Result<u32, failure::Error> {
        println!("TODO : subscribe : port_index: {}, protocol: {}", port_index, protocol);
        Ok(0)
    }

    pub fn unsubscribe(
        &mut self,
        port_index: u32,
        protocol: u32,
        _features: *const *const lv2_raw::LV2Feature,
    ) -> Result<u32, failure::Error> {
        println!("TODO : unsubscribe : port_index: {}, protocol: {}", port_index, protocol);
        Ok(0)
    }

    pub fn touch(
        &mut self,
        port_index: u32,
        grabbed: bool,
    ) -> Result<(), failure::Error> {
        self.plugin.touch(port_index, grabbed);
        Ok(())
    }

    pub fn notify_port_events(&mut self) -> Result<Option<Vec<(u32, u32, u32, Vec<u8>)>>, failure::Error> {
        Ok(self.plugin.read())
    }
}
