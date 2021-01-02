use std::cell::RefCell;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::ffi::{CStr, CString};
use std::iter::Extend;
use std::rc::Rc;
use std::sync::atomic::AtomicU32;
use std::sync::Arc;
use std::sync::RwLock;

use talker::talker::RTalker;
use talker::talker_handler::TalkerHandlerBase;

use crate::lv2_resources::{Lv2Resources, UridMapFeature};
use crate::talkers::abs_sine;
use crate::talkers::abs_sine::AbsSine;
use crate::talkers::lv2::Lv2;
use crate::talkers::second_degree_frequency_progression;
use crate::talkers::second_degree_frequency_progression::SecondDegreeFrequencyProgression;
use crate::talkers::sinusoidal;
use crate::talkers::sinusoidal::Sinusoidal;
/*
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
        ]));

        self
    }
    pub fn buffer(&self) -> SharedFeatureBuffer {
        Rc::clone(&self.buffer)
    }
}
*/
enum PluginType {
    Internal,
    Lv2(lilv::Plugin),
}

pub struct PluginHandler {
    base: TalkerHandlerBase,
    plugin_type: PluginType,
}

pub struct PluginsManager {
    lv2_resources: Lv2Resources,
    //    features: GpFeatureSet,
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

    pub fn make_plugins_handlers(world: &lilv::World) -> HashMap<String, PluginHandler> {
        println!("make_plugins_handlers start");
        let mut handlers = HashMap::new();

        //    let features = [w.new_uri(UridMapFeature::URI)];
        let supported_features = [world.new_uri(UridMapFeature::URI)];

        for plugin in world.all_plugins().iter() {
            if plugin.uri().as_uri().is_none() {
                println!("Could not get uri from {:?}.", plugin.uri().turtle_token());
                continue;
            }
            let mut supported = true;
            if let Some(required_features) = plugin.required_features() {
                for feature in required_features.iter() {
                    if supported_features.iter().find(|f| *f == &feature).is_none() {
                        supported = false;
                        println!(
                            "LV2 plugin {:?} requires feature {:?}.",
                            plugin.uri().turtle_token(),
                            feature.turtle_token()
                        );
                    }
                }
            }
            if let Some(optional_features) = plugin.optional_features() {
                for feature in optional_features.iter() {
                    if supported_features.iter().find(|f| *f == &feature).is_none() {
                        println!(
                            "LV2 plugin {:?} has optional feature {:?}.",
                            plugin.uri().turtle_token(),
                            feature.turtle_token()
                        );
                    }
                }
            }
            if !supported {
                continue;
            }

            handlers.insert(
                plugin.uri().turtle_token(),
                PluginHandler {
                    base: TalkerHandlerBase::new(
                        &plugin.class().label().turtle_token(),
                        &plugin.uri().turtle_token(),
                        &plugin.name().turtle_token(),
                    ),
                    plugin_type: PluginType::Lv2(plugin),
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
        let lv2_resources = Lv2Resources::new();
        let handlers = PluginsManager::make_plugins_handlers(&lv2_resources.world);

        Self {
            lv2_resources,
            handlers,
        }
    }
    /*
        pub fn features_buffer(&self) -> SharedFeatureBuffer {
            self.features.buffer()
        }
    */
    //    pub fn make_lv2_talker(&self, model: &String) -> Result<RTalker, failure::Error> {
    pub fn make_lv2_talker(&self, plugin: &lilv::Plugin) -> Result<RTalker, failure::Error> {
        let features: Vec<*const lv2_raw::LV2Feature> = vec![
            self.lv2_resources.urid_map.as_lv2_feature(),
            std::ptr::null(),
        ];
        Lv2::new(
            &self.lv2_resources,
            features.as_ptr(),
            plugin, //            ph.base.model(),
        )
    }

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
            PluginType::Lv2(plugin) => self.make_lv2_talker(&plugin),
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
