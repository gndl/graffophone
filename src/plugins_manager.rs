use lilv::plugin::Plugin;
use lilv::world::World;
use gpplugin::talker::{Talker, Handler};
use crate::lv2_talker;
use crate::lv2_talker::Lv2Talker;

pub struct PluginsManager {
    world: lilv::world::World,
    plugin_handlers: Vec<Handler>,
}

impl PluginsManager {
    pub fn new() -> Self {
        Self {
	    world: World::new().unwrap(),
	        plugin_handlers: Vec::new(),
        }
    }

    pub fn lilv_world<'a>(&'a self) -> &'a World {
	&self.world
    }
    
    pub fn load_plugins<'a>(&'a mut self) -> &'a Vec<Handler> {
    /*
    let tkr: Box<dyn Talker> = Box::new(Lv2Talker::new().unwrap());
    println!("tkr id {}, name {}", tkr.get_id(), tkr.get_name());
    let phs = Vec::new();
*/

    //    println!("lilv_plugins_size: {}", lilv_sys::lilv_plugins_size(plugins));

    println!("Print plugins start");

	for plugin in self.world.plugins() {
	    let plg_uri = String::from(plugin.uri().to_string());
	    
        let ph = Handler::new(
            plugin.name().to_str(),
            plugin.class().label().to_str(),
            Box::new(move || Box::new(Lv2Talker::new(&plg_uri))),
        );
            self.plugin_handlers.push(ph);

        println!("{}({}) {}", plugin.name(), plugin.class().label().to_str(), plugin.uri());
        /*
        for port in plugin.inputs() {
            println!("> {:?}", port);
        }
        for port in plugin.outputs() {
            println!("< {:?}", port);
        }
        */
    }
    println!("Print plugins end");
        &self.plugin_handlers
}
}

