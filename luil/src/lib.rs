use std::collections::HashMap;
use std::ffi::CString;

extern crate failure;

mod features;
mod host;
mod plugin;
mod plugin_controller;
mod plugin_handle;
mod plugin_ui;
mod suil;
mod xwindow;

pub type HostConfiguration = host::HostConfiguration;
pub type PluginHandle = plugin_handle::PluginHandle;

pub trait PluginHandleBuilderTrait {
    fn visit(&mut self, instance: Option<&livi::Instance>) -> Result<PluginHandle, failure::Error>;
}
pub type PluginHandleBuilder = Box<dyn PluginHandleBuilderTrait>;

pub trait PluginTrait {
    fn urid_map(&mut self, uri: CString) -> lv2_raw::LV2Urid;
    fn urid_unmap(&mut self, urid: lv2_raw::LV2Urid) -> Option<CString>;
    fn index(&mut self, port_symbol: String) -> u32;
    fn notify(&mut self, message: String);
    fn on_run(&mut self) {}
    fn read(&mut self) -> Option<Vec<(u32, u32, u32, Vec<u8>)>> { None }
    fn write(&mut self, port_index: u32, buffer_size: u32, protocol: u32, buffer: Vec<u8>);
    fn touch(&mut self, _port_index: u32, _grabbed: bool) {}
}
pub type Plugin = Box<dyn PluginTrait>;

pub struct PluginHandleBuilderImpl {
    host_configuration: HostConfiguration,
    plugin_uri: String,
    bundle_uri: String,
    instance_name: String,
    plugin: Option<Plugin>,
}
impl PluginHandleBuilderTrait for PluginHandleBuilderImpl {
    fn visit(&mut self, instance: Option<&livi::Instance>) -> Result<PluginHandle, failure::Error> {

        let plugin = self.plugin.take().unwrap();

        let plugin_handle = PluginHandle::new(
            self.host_configuration,
            &self.plugin_uri,
            &self.bundle_uri,
            &self.instance_name,
            plugin,
            instance,
        )?;
        Ok(plugin_handle)
    }
}

pub struct Luil {
    host_configuration: HostConfiguration,
    instances: HashMap<String, PluginHandle>,
}

impl Luil {
    pub fn new(host_configuration: HostConfiguration) -> Luil {
        suil::init();
        Self {
            host_configuration,
            instances: HashMap::new(),
        }
    }

    pub fn launch_plugin_ui<F>(
        &mut self,
        plugin_uri: &str,
        bundle_uri: &str,
        instance_id: &str,
        instance_name: &str,
        plugin: Plugin,
        mut visite_instance: F
    ) -> Result<usize, failure::Error>
    where
        F: FnMut(&mut PluginHandleBuilder) -> Result<PluginHandle, failure::Error>,
    {
        let plugin_handle_builder = PluginHandleBuilderImpl {
            host_configuration: self.host_configuration,
            plugin_uri: plugin_uri.to_string(),
            bundle_uri: bundle_uri.to_string(),
            instance_name: instance_name.to_string(),
            plugin: Some(plugin),
        };
        let mut builder: PluginHandleBuilder = Box::new(plugin_handle_builder);
        let mut plugin_handle = visite_instance(&mut builder)?;

        plugin_handle.instanciate()?;

        self.instances.insert(instance_id.to_string(), plugin_handle);

        Ok(self.instances.len())
    }

    pub fn run(&mut self) -> Result<usize, failure::Error> {
        let mut exited = Vec::new();

        for (id, inst) in &mut self.instances {
            if !inst.run()? {
                exited.push(id.to_string());
            }
        }

        for id in &exited {
            self.instances.remove(id);
        }

        Ok(self.instances.len())
    }

    pub fn running_instances_count(&self) -> usize {
        self.instances.len()
    }
}

impl Drop for Luil {
    fn drop(&mut self) {
        for _inst in self.instances.values() {
            println!("Todo : close plugins UIs.")
        }
    }
}


#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Write;
    use std::ffi::CString;
    use crate::{HostConfiguration, PluginTrait, Luil};
    struct PluginTest {
        logfile: File,
        uri_count: u32,
    }

    impl PluginTrait for PluginTest {
        fn urid_map(&mut self, uri: CString) -> lv2_raw::LV2Urid {
            self.uri_count += 1;
            let _ = writeln!(self.logfile, "urid_map : {:?} -> {}", uri, self.uri_count);
            self.uri_count
        }
        fn urid_unmap(&mut self, _urid: lv2_raw::LV2Urid) -> Option<CString> {
            None
        }
        fn index(&mut self, port_symbol: String) -> u32 {
            let _ = writeln!(self.logfile, "index : port_symbol: {}", port_symbol);
            42
        }
        fn notify(&mut self, message: String) {
            let _ = writeln!(self.logfile, "notify : message: {}", message);
        }
        fn write(&mut self, port_index: u32, buffer_size: u32, protocol: u32, buffer: Vec<u8>) {
            let _ = writeln!(self.logfile, "write : port_index: {}, buffer_size: {}, protocol: {}, buffer: {:?}", port_index, buffer_size, protocol, buffer);
        }
        fn touch(&mut self, port_index: u32, grabbed: bool) {
            let _ = writeln!(self.logfile, "Touch : port_index: {}, grabbed: {}", port_index, grabbed);
        }
        fn read(&mut self) -> Option<Vec<(u32, u32, u32, Vec<u8>)>> {
            // let _ = writeln!(self.logfile, "read");
            None
        }
    }

    fn run_luil() -> Result<(), failure::Error>{
        let mut luil = Luil::new(HostConfiguration {
                sample_rate: 44100.,
                support_touch: true,
                support_peak_protocol: true,
            });

        let host = Box::new(PluginTest {logfile: File::create("sfizz.log")?, uri_count: 0});

        // "http://guitarix.sourceforge.net/plugins/gx_bmp_#_bmp_",
        let _ = luil.launch_plugin_ui(
            "http://sfztools.github.io/sfizz",
            "file:///usr/lib/lv2/sfizz.lv2/",
            "sfizz",
            "sfizz_01",
            host,
        |v| v.visit(None)
        )?;

        while luil.run()? > 0 {
            std::thread::sleep(std::time::Duration::from_millis(20));
        }

        let host = Box::new(PluginTest {logfile: File::create("Fluidsynth.log")?, uri_count: 0});

        let _ = luil.launch_plugin_ui(
            "http://calf.sourceforge.net/plugins/Fluidsynth",
            None,
            "Fluidsynth",
            "Fluidsynth_01",
            host,
            |v| v.visit(None)
        )?;

        while luil.run()? > 0 {
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        Ok(())
    }

    #[test]
    fn test_luil() {

        match run_luil() {
            Ok(()) => (),
            Err(e) => {
                let mut logfile = File::create("server_error.log").unwrap();
                let _ = writeln!(logfile, "{}", e);
            }
        }
    }
}
