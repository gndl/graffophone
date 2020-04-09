use std::cell::RefCell;
use std::collections::HashMap;
use std::iter::Extend;
use std::rc::Rc;

use lv2::core::{FeatureBuffer, SharedFeatureBuffer};
use lv2::urid::features::{URIDMap, URIDUnmap};

use lilv::world::World;

use granode::talker::RTalker;
use granode::talker_handler::TalkerHandlerBase;

use crate::talkers::abs_sine;
use crate::talkers::abs_sine::AbsSine;
use crate::talkers::lv2::Lv2;
use crate::talkers::second_degree_frequency_progression;
use crate::talkers::second_degree_frequency_progression::SecondDegreeFrequencyProgression;
use crate::talkers::sinusoidal;
use crate::talkers::sinusoidal::Sinusoidal;

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
        self.buffer = Rc::new(FeatureBuffer::from_vec(vec![
            /*
            Feature::descriptor(&self.hard_rt_capable),
            Feature::descriptor(&self.urid_unmap),
            Feature::descriptor(&self.urid_map),
                            */
        ]));

        self
    }
    pub fn buffer(&self) -> SharedFeatureBuffer {
        Rc::clone(&self.buffer)
    }
}
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
    fn tkr_hr_kv(base: TalkerHandlerBase) -> (String, PluginHandler) {
        (
            base.model().to_string(),
            PluginHandler {
                base,
                plugin_type: PluginType::Internal,
            },
        )
    }

    pub fn make_plugins_handlers(world: &World) -> HashMap<String, PluginHandler> {
        println!("make_plugins_handlers start");
        let mut handlers = HashMap::new();

        for plugin in world.plugins() {
            handlers.insert(
                plugin.uri().to_string(),
                PluginHandler {
                    base: TalkerHandlerBase::new(
                        plugin.class().label().to_str(),
                        plugin.uri().to_string().as_str(),
                        plugin.name().to_str(),
                    ),
                    plugin_type: PluginType::Lv2,
                },
            );
        }

        handlers.extend(vec![
            PluginsManager::tkr_hr_kv(AbsSine::descriptor()),
            PluginsManager::tkr_hr_kv(Sinusoidal::descriptor()),
            PluginsManager::tkr_hr_kv(SecondDegreeFrequencyProgression::descriptor()),
        ]);

        println!("make_plugins_handlers end");
        handlers
    }

    pub fn new() -> Self {
        let world = World::new().unwrap();
        let handlers = PluginsManager::make_plugins_handlers(&world);

        Self {
            world,
            features: GpFeatureSet::new(),
            handlers,
        }
    }

    pub fn features_buffer(&self) -> SharedFeatureBuffer {
        self.features.buffer()
    }
    /*
    fn add_handler(&mut self, base: TalkerHandlerBase) {
        self.handlers.insert(
            base.model().to_string(),
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
                            plugin.class().label().to_str(),
                            plugin.uri().to_string().as_str(),
                            plugin.name().to_str(),
                        ),
                        plugin_type: PluginType::Lv2,
                    },
                );
            }

            self.add_handler(AbsSine::descriptor());
            self.add_handler(Sinusoidal::descriptor());
            self.add_handler(SecondDegreeFrequencyProgression::descriptor());

            println!("load_plugins end");
        }
    */
    pub fn make_internal_talker(&self, model: &String) -> Result<RTalker, failure::Error> {
        if model == sinusoidal::MODEL {
            Ok(Rc::new(RefCell::new(Sinusoidal::new())))
        } else if model == abs_sine::MODEL {
            Ok(Rc::new(RefCell::new(AbsSine::new())))
        } else if model == second_degree_frequency_progression::MODEL {
            Ok(Rc::new(RefCell::new(
                SecondDegreeFrequencyProgression::new(110., 0., 1., 1.),
            )))
        } else {
            Err(failure::err_msg("Unknown talker MODEL"))
        }
    }

    pub fn mk_tkr(&self, ph: &PluginHandler) -> Result<RTalker, failure::Error> {
        match &ph.plugin_type {
            PluginType::Lv2 => Lv2::new(&self.world, self.features.buffer(), ph.base.model()),
            PluginType::Internal => self.make_internal_talker(ph.base.model()),
        }
    }

    pub fn make_talker(&self, model: &str) -> Result<RTalker, failure::Error> {
        match self.handlers.get(model.to_string().as_str()) {
            Some(ph) => self.mk_tkr(ph),
            None => Err(failure::err_msg("Unknown talker URI")),
        }
    }

    pub fn get_categorized_talkers_label_model(&self) -> Vec<(String, Vec<(String, String)>)> {
        let mut categories_map: HashMap<&String, Vec<(&String, &String)>> = HashMap::new();
        let mut categories_vec: Vec<(String, Vec<(String, String)>)> = Vec::new();

        for (model, ph) in self.handlers.iter() {
            //            println!("Plugin {} ({})", ph.base.model(), ph.base.category());

            match categories_map.get_mut(ph.base.category()) {
                Some(category_talkers) => {
                    category_talkers.push((ph.base.label(), model));
                }
                None => {
                    let mut category_talkers = Vec::new();
                    category_talkers.push((ph.base.label(), model));
                    categories_map.insert(ph.base.category(), category_talkers);
                }
            }
        }

        for (category, talkers) in categories_map {
            let mut tkrs = Vec::new();

            for (label, model) in talkers {
                tkrs.push((label.to_string(), model.to_string()));
            }
            tkrs.sort();
            categories_vec.push((category.to_string(), tkrs));
        }
        categories_vec.sort();
        categories_vec
    }

    pub fn run(&self) {
        let mut talkers = Vec::new();

        for (_model, ph) in self.handlers.iter() {
            println!("Plugin {} ({})", ph.base.model(), ph.base.category());

            match self.mk_tkr(ph) {
                Ok(tkr) => {
                    talkers.push(tkr);
                }
                Err(e) => {
                    eprintln!("Make talker failed: {:?}", e);
                }
            }
        }

        for tkr in &talkers {
            println!("Plugin {} {}", tkr.borrow().model(), tkr.borrow().name());
        }
    }
}
