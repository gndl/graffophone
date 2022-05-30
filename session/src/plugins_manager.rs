use std::collections::HashMap;
use std::iter::Extend;

use talker::identifier::Identifiable;
use talker::lv2_handler;
use talker::rtalker;
use talker::talker::RTalker;
use talker::talker::TalkerCab;
use talker::talker_handler::TalkerHandlerBase;
use talkers::abs_sine;
use talkers::abs_sine::AbsSine;
use talkers::lv2::Lv2;
use talkers::round;
use talkers::round::Round;
use talkers::second_degree_frequency_progression;
use talkers::second_degree_frequency_progression::SecondDegreeFrequencyProgression;
use talkers::sinusoidal;
use talkers::sinusoidal::Sinusoidal;
use talkers::tseq::tseq;
use talkers::tseq::tseq::Tseq;

enum PluginType {
    Internal,
    Lv2,
}

pub struct PluginHandler {
    base: TalkerHandlerBase,
    plugin_type: PluginType,
}

pub struct PluginsManager {
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

    fn make_plugins_handlers() -> HashMap<String, PluginHandler> {
        println!("make_plugins_handlers start");
        let mut handlers = HashMap::new();

        lv2_handler::visit(|lv2_handler| {
            for plugin in lv2_handler.world.iter_plugins() {
                handlers.insert(
                    plugin.uri(),
                    PluginHandler {
                        base: TalkerHandlerBase::new(
                            "lv2", //plugin.class().label().to_str(),
                            &plugin.uri(),
                            &plugin.name(),
                        ),
                        plugin_type: PluginType::Lv2,
                    },
                );
            }
            Ok(())
        })
        .unwrap_or_else(|e| eprintln!("PluginsManager::make_plugins_handlers failed : {:?}", e));

        handlers.extend(vec![
            PluginsManager::tkr_hr_kv(AbsSine::descriptor()),
            PluginsManager::tkr_hr_kv(Sinusoidal::descriptor()),
            PluginsManager::tkr_hr_kv(SecondDegreeFrequencyProgression::descriptor()),
            PluginsManager::tkr_hr_kv(Tseq::descriptor()),
            PluginsManager::tkr_hr_kv(Round::descriptor()),
        ]);

        println!("make_plugins_handlers end");
        handlers
    }

    pub fn new() -> Self {
        Self {
            handlers: PluginsManager::make_plugins_handlers(),
        }
    }

    pub fn make_internal_talker(&self, model: &String) -> Result<RTalker, failure::Error> {
        if model == sinusoidal::MODEL {
            Ok(rtalker!(Sinusoidal::new()?))
        } else if model == abs_sine::MODEL {
            Ok(rtalker!(AbsSine::new()?))
        } else if model == second_degree_frequency_progression::MODEL {
            Ok(rtalker!(SecondDegreeFrequencyProgression::new(
                110., 0., 1., 1.
            )?))
        } else if model == tseq::MODEL {
            Ok(rtalker!(Tseq::new()?))
        } else if model == round::MODEL {
            Ok(rtalker!(Round::new()?))
        } else {
            Err(failure::err_msg("Unknown talker MODEL"))
        }
    }

    pub fn mk_tkr(&self, ph: &PluginHandler) -> Result<RTalker, failure::Error> {
        match &ph.plugin_type {
            PluginType::Lv2 => lv2_handler::visit(|lv2_handler| {
                Ok(rtalker!(Lv2::new(lv2_handler, ph.base.model())?))
            }),
            PluginType::Internal => self.make_internal_talker(ph.base.model()),
        }
    }

    pub fn make_talker(&self, model: &str) -> Result<RTalker, failure::Error> {
        match self.handlers.get(model) {
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
            println!("Plugin {} {}", tkr.model(), tkr.name());
        }
    }
}
