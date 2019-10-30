use crate::lv2_talker;
use gpplugin::talker::TalkerHandler;

use lilv::world::World;
/*
use gpplugin::talker::Talker;
use crate::lv2_talker;
use lv2::core::FeatureBuffer;
use lilv::plugin::Plugin;
use lilv::port::{UnknownInputPort, UnknownOutputPort};
use gpplugin::talker::{TalkerHandler, TalkerHandlerBase};
use crate::lv2_talker::Lv2TalkerHandler;
*/
enum PluginHandler {
    Lv2 { uri: String },
}

pub struct PluginsManager {
    // world: World,
    // feature_set: GpFeatureSet,
    //    pub handlers: Vec<Box<dyn TalkerHandler>>,
    pub handlers: Vec<Box<dyn TalkerHandler>>,
}

impl PluginsManager {
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
        }
    }

    pub fn load_plugins(&mut self, world: &World /*, features: &'a FeatureBuffer*/) {
        println!("load_plugins start");
        /*
        lv2_talker::load_plugins(&mut self.handlers);
        for plugin in global.world.plugins() {
            talker_handlers.push(Box::new(Lv2TalkerHandler::new(
                TalkerHandlerBase::new(plugin.name().to_str(), plugin.class().label().to_str()),
                String::from(plugin.uri().to_string()),
            )));
        }
         */
        println!("load_plugins end");
    }

    pub fn run(&self) {
        for ph in self.handlers {
            println!("Plugin {} ({})", ph.kind(), ph.category());

            match ph.make() {
                Ok(tkr) => {
                    println!("Plugin {} {}", tkr.id(), tkr.name());
                    //           talkers.push(tkr);
                }
                Err(e) => {
                    eprintln!("Make talker failed: {:?}", e);
                }
            }
        }
    }
}
