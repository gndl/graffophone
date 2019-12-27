use std::collections::HashMap;
//use std::error::Error;
use crate::talkers::abs_sine;
use crate::talkers::abs_sine::AbsSine;
use crate::talkers::lv2::Lv2;
use lv2::urid::features::{URIDMap, URIDUnmap};
//use lv2::urid::{SimpleMapper, URIDOf, URID};
use std::cell::RefCell;
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
    Internal,
    Lv2,
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

    fn add_handler(&mut self, base: TalkerHandlerBase) {
        self.handlers.insert(
            base.id().to_string(),
            PluginHandler {
                base,
                plugin_type: PluginType::Internal,
            },
        );
    }

    pub fn load_plugins(&mut self) {
        println!("load_plugins start");
        for plugin in self.world.plugins() {
            self.handlers.insert(
                plugin.uri().to_string(),
                PluginHandler {
                    base: TalkerHandlerBase::new(
                        plugin.uri().to_string().as_str(),
                        plugin.name().to_str(),
                        plugin.class().label().to_str(),
                    ),
                    plugin_type: PluginType::Lv2,
                },
            );
        }

        self.add_handler(abs_sine::descriptor());

        println!("load_plugins end");
    }

    pub fn make_internal_talker(&self, id: &String) -> Result<MTalker, failure::Error> {
        if id == abs_sine::id() {
            Ok(Rc::new(RefCell::new(AbsSine::new())))
        } else {
            Err(failure::err_msg("Unknown talker ID"))
        }
    }

    pub fn mk_tkr(&self, ph: &PluginHandler) -> Result<MTalker, failure::Error> {
        match &ph.plugin_type {
            PluginType::Lv2 => Lv2::new(&self.world, self.features.buffer(), ph.base.id()),
            PluginType::Internal => self.make_internal_talker(ph.base.id()),
        }
    }

    pub fn make_talker(
        &self,
        id: &String,
        name: Option<&String>,
    ) -> Result<MTalker, failure::Error> {
        match self.handlers.get(id) {
            Some(ph) => {
                let talker = self.mk_tkr(ph);
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
            None => Err(failure::err_msg("Unknown talker URI")),
        }
    }

    pub fn run(&self) {
        let mut talkers = Vec::new();

        for (_id, ph) in self.handlers.iter() {
            println!("Plugin {} ({})", ph.base.model(), ph.base.category());

            match self.mk_tkr(ph) {
                Ok(tkr) => {
                    //                    println!("Plugin {} {}", tkr.id(), tkr.name());
                    talkers.push(tkr);
                }
                Err(e) => {
                    eprintln!("Make talker failed: {:?}", e);
                }
            }
        }

        for tkr in &talkers {
            println!("Plugin {} {}", tkr.borrow().id(), tkr.borrow().name());
        }
    }
}
