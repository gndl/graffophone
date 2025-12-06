extern crate suil_sys;

use crate::host::Host;
use crate::suil;
use crate::Plugin;
use crate::features;
use crate::plugin;
use crate::plugin_ui;
use crate::xwindow;
use crate::HostConfiguration;

const MIN_BLOCK_SIZE: usize = 1;
const MAX_BLOCK_SIZE: usize = 4096;

pub struct PluginHandle {
    host: Host,
    features: features::Features,
    plugin_ui_parameters: plugin_ui::Parameters,
    xwindow: Option<xwindow::XWindow>,
    suil_instance: Option<suil::Instance>,
}

impl PluginHandle {
    pub fn new(
        host_configuration: HostConfiguration,
        plugin_uri: &str,
        bundle_uri: &str,
        plugin_instance_name: &str,
        plugin: Plugin,
        instance: Option<&livi::Instance>,
    ) -> Result<PluginHandle, failure::Error> {

        let mut host = Host::new(host_configuration, plugin)?;

        let world = if bundle_uri.is_empty() { livi::World::new() } else { livi::World::with_load_bundle(bundle_uri)};
    
        let plugin = world.plugin_by_uri(&plugin_uri).ok_or(failure::err_msg(format!("LV2 plugin {} not found.", plugin_uri)))?;
    
        let plugin_ui_parameters = plugin::select_plugin_ui(&world, &plugin)?;
    
        let mut features = features::Features::new(
            MIN_BLOCK_SIZE,
            MAX_BLOCK_SIZE,
            plugin_instance_name,
            &mut host,
            // None,
            instance,
        );
    
        if plugin_ui_parameters.is_x11 {
            let mut xwin = xwindow::XWindow::new(plugin_instance_name)?;
    
            features.add_xwindow(&mut xwin);
    
            Ok(Self {host, features, plugin_ui_parameters, xwindow: Some(xwin), suil_instance: None})
        }
        else {
            Ok(Self {host, features, plugin_ui_parameters, xwindow: None, suil_instance: None})
        }
    }

    pub fn instanciate(&mut self) -> Result<(), failure::Error> {
        self.suil_instance = Some(suil::Instance::new(
            &mut self.host,
            &self.features,
            &self.plugin_ui_parameters,
        )?);

        if let Some(xwin) = &mut self.xwindow {
            xwin.show()?;
        }
    
        Ok(())
    }

    pub fn run(&mut self) -> Result<bool, failure::Error> {
        if let Some(suil_instance) = &self.suil_instance {
            return suil_instance.run(&mut self.host, &self.xwindow);
        }
        Ok(false)
    }
}
