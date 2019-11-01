use std::rc::Rc;
use crate::lv2_talker::Lv2Talker;
use gpplugin::talker::{Talker, TalkerHandlerBase};

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
use lv2::core::{Feature, FeatureBuffer, FeatureSet};

struct GpFeatureSet {
    hard_rt_capable: ::lv2::core::features::HardRTCapable,
}

impl GpFeatureSet {
    pub fn new() -> Self {
        Self {
            hard_rt_capable: ::lv2::core::features::HardRTCapable,
        }
    }
}

impl<'a> FeatureSet<'a> for GpFeatureSet {
    fn to_list(&self) -> FeatureBuffer {
        FeatureBuffer::from_vec(vec![Feature::descriptor(&self.hard_rt_capable)])
    }
}

enum PluginType {
    Lv2 { uri: String },
}

pub struct PluginHandler {
    base: TalkerHandlerBase,
    plugin_type: PluginType,
}

pub struct PluginsManager {
    world: World,
     feature_set: GpFeatureSet,
     handlers: Vec< PluginHandler>,
//     handlers: Vec<Box<dyn TalkerHandler>>,
}

impl PluginsManager {
    pub fn new() -> Self {
        Self {
            world: World::new().unwrap(),
            feature_set: GpFeatureSet::new(),
            handlers: Vec::new(),
        }
    }

    pub fn load_plugins(&mut self/*, world: &World , features: &'a FeatureBuffer*/) {
        println!("load_plugins start");
        /*
        lv2_talker::load_plugins(&mut self.handlers);
        for plugin in self.world.plugins() {
            self.handlers.push(Rc::new(Lv2TalkerHandler::new(
                TalkerHandlerBase::new(plugin.name().to_str(), plugin.class().label().to_str()),
                String::from(plugin.uri().to_string()),
            )));
        }
         */
        for plugin in self.world.plugins() {
            self.handlers.push(PluginHandler {
                base: TalkerHandlerBase::new(plugin.name().to_str(), plugin.class().label().to_str()),
                plugin_type: PluginType::Lv2{uri: String::from(plugin.uri().to_string())},
            });
        }
        println!("load_plugins end");
    }

    pub fn run(&self) {
let mut talkers = Vec::new();

        for ph in &self.handlers {
            println!("Plugin {} ({})", ph.base.kind(), ph.base.category());

            match &ph.plugin_type {
PluginType::Lv2 {uri } =>

match Lv2Talker::new(
            &self.world, &self.feature_set.to_list(), &uri,
        ) {
                Ok(tkr) => {
//                    println!("Plugin {} {}", tkr.id(), tkr.name());
                   talkers.push(Rc::new(tkr));
                }
                Err(e) => {
                    eprintln!("Make talker failed: {:?}", e);
                }
            }
        }
    }
for tkr in &talkers {
                    println!("Plugin {} {}", tkr.id(), tkr.name());

}
}
}
