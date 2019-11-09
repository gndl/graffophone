use std::collections::HashMap;
//use std::error::Error;
use crate::lv2_talker::Lv2Talker;
use lv2::urid::features::{URIDMap, URIDUnmap};
//use lv2::urid::{SimpleMapper, URIDOf, URID};
use std::ffi::CString;
use std::rc::Rc;

use gpplugin::talker::{Talker, TalkerHandlerBase};
//use lv2::units::units::Frame;

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
    urid_map: ::lv2::urid::features::URIDMap,
    urid_unmap: ::lv2::urid::features::URIDUnmap,
//    mapper: HashMap<CString, u32>,
}

impl GpFeatureSet {
    pub fn new() -> Self {
/*
let m = HashMap::new();
        let mut fs = 
*/
Self {
            hard_rt_capable: ::lv2::core::features::HardRTCapable,
            urid_map: URIDMap::new(),
            urid_unmap: URIDUnmap::new(),
//            mapper: HashMap::new(),
        }
/*
;
fs.urid_map = URIDMap::new_with_hashmap(&fs.mapper);
            fs.urid_unmap = URIDUnmap::new_with_hashmap(&fs.mapper);
fs
*/
    }
}

impl<'a> FeatureSet<'a> for GpFeatureSet {
    fn to_list(&self) -> FeatureBuffer {
        FeatureBuffer::from_vec(vec![
            Feature::descriptor(&self.hard_rt_capable),
            Feature::descriptor(&self.urid_map),
            Feature::descriptor(&self.urid_unmap),
        ])
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
    handlers: Vec<PluginHandler>,
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

    pub fn load_plugins(&mut self /*, world: &World , features: &'a FeatureBuffer*/) {
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
                base: TalkerHandlerBase::new(
                    plugin.name().to_str(),
                    plugin.class().label().to_str(),
                ),
                plugin_type: PluginType::Lv2 {
                    uri: String::from(plugin.uri().to_string()),
                },
            });
        }
        println!("load_plugins end");
    }

    pub fn run(&self) {
        let mut talkers = Vec::new();

        for ph in &self.handlers {
            println!("Plugin {} ({})", ph.base.kind(), ph.base.category());

            match &ph.plugin_type {
                PluginType::Lv2 { uri } => {
                    match Lv2Talker::new(&self.world, &self.feature_set.to_list(), &uri) {
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
        }

        for tkr in &talkers {
            println!("Plugin {} {}", tkr.id(), tkr.name());
        }
    }
}
