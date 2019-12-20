use std::collections::HashMap;
//use std::error::Error;
use crate::lv2_talker::Lv2Talker;
use lv2::urid::features::{URIDMap, URIDUnmap};
//use lv2::urid::{SimpleMapper, URIDOf, URID};
use std::rc::Rc;

use gpplugin::talker;
use gpplugin::talker::MTalker;
use gpplugin::talker_handler::TalkerHandlerBase;
//use lv2::units::units::Frame;

use lilv::world::World;
/*
use std::ffi::CString;
use gpplugin::talker::Talker;
use crate::lv2_talker;
use lv2::core::FeatureBuffer;
use lilv::plugin::Plugin;
use lilv::port::{UnknownInputPort, UnknownOutputPort};
use gpplugin::talker::{TalkerHandler, TalkerHandlerBase};
use crate::lv2_talker::Lv2TalkerHandler;
*/
use lv2::core::{Feature, FeatureBuffer, FeatureSet, SharedFeatureBuffer};

struct GpFeatureSet {
    hard_rt_capable: ::lv2::core::features::HardRTCapable,
    urid_map: ::lv2::urid::features::URIDMap,
    urid_unmap: ::lv2::urid::features::URIDUnmap,
    buffer: SharedFeatureBuffer,
}

impl GpFeatureSet {
    pub fn new() -> Self {
        GpFeatureSet::init(Self {
            hard_rt_capable: ::lv2::core::features::HardRTCapable,
            urid_map: URIDMap::new(),
            urid_unmap: URIDUnmap::new(),
            buffer: Rc::new(FeatureBuffer::new()),
        })
    }
    fn init(mut self) -> Self {
        self.buffer = Rc::new(
            FeatureBuffer::from_vec(vec![
                Feature::descriptor(&self.hard_rt_capable),
                Feature::descriptor(&self.urid_map),
                Feature::descriptor(&self.urid_unmap),
            ]), //                     self.to_list()
        );
        self
    }
    pub fn buffer(&self) -> SharedFeatureBuffer {
        self.buffer.clone()
        //Rc::clone(&
    }
}
/*
impl FeatureSet for GpFeatureSet {
    fn to_list(&self) -> FeatureBuffer {
        FeatureBuffer::from_vec(vec![
            Feature::descriptor(&self.hard_rt_capable),
            Feature::descriptor(&self.urid_map),
            Feature::descriptor(&self.urid_unmap),
        ])
    }
}
*/
enum PluginType {
    Lv2 { uri: String },
}

pub struct PluginHandler {
    base: TalkerHandlerBase,
    plugin_type: PluginType,
}

pub struct PluginsManager {
    world: World,
    features: GpFeatureSet,
    handlers: HashMap<String, PluginHandler>,
}

impl PluginsManager {
    pub fn new() -> Self {
        Self {
            world: World::new().unwrap(),
            features: GpFeatureSet::new(),
            handlers: HashMap::new(),
        }
    }

    pub fn features_buffer(&self) -> SharedFeatureBuffer {
        self.features.buffer()
    }
    pub fn load_plugins(&mut self) {
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
            self.handlers.insert(
                plugin.uri().to_string(),
                PluginHandler {
                    base: TalkerHandlerBase::new(
                        plugin.uri().to_string().as_str(),
                        plugin.name().to_str(),
                        plugin.class().label().to_str(),
                    ),
                    plugin_type: PluginType::Lv2 {
                        uri: String::from(plugin.uri().to_string()),
                    },
                },
            );
        }
        println!("load_plugins end");
    }

    pub fn run(&self) {
        let mut talkers = Vec::new();

        for (_id, ph) in self.handlers.iter() {
            println!("Plugin {} ({})", ph.base.model(), ph.base.category());

            match &ph.plugin_type {
                PluginType::Lv2 { uri } => {
                    match Lv2Talker::new(&self.world, self.features.buffer(), &uri) {
                        Ok(tkr) => {
                            //                    println!("Plugin {} {}", tkr.id(), tkr.name());
                            talkers.push(tkr);
                        }
                        Err(e) => {
                            eprintln!("Make talker failed: {:?}", e);
                        }
                    }
                }
            }
        }

        for tkr in &talkers {
            println!("Plugin {} {}", tkr.borrow().id(), tkr.borrow().name());
        }
    }

    pub fn make_talker(
        &self,
        uri: &String,
        name: Option<&String>,
    ) -> Result<MTalker, failure::Error> {
        match self.handlers.get(uri) {
            Some(ph) => match &ph.plugin_type {
                PluginType::Lv2 { uri } => {
                    let talker = Lv2Talker::new(&self.world, self.features.buffer(), &uri);
                    match talker {
                        Ok(tkr) => {
                            match name {
                                Some(nm) => tkr.borrow().set_name(nm),
                                None => (),
                            };

                            return Ok(tkr);
                        }
                        Err(e) => {
                            eprintln!("Make talker failed: {:?}", e);
                            return Err(e);
                        }
                    }
                }
            },
            None => Err(failure::err_msg("Unknown talker URI")),
        }
    }
}
