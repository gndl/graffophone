use std::collections::HashMap;
use std::iter::Extend;

use talker::identifier::Identifiable;
use talker::lv2_handler;
use talker::rtalker;
use talker::talker::{RTalker, TalkerCab};
use talker::talker_handler::TalkerHandlerBase;
use talkers::abs_sine::{self, AbsSine};
use talkers::accumulator::{self, Accumulator};
use talkers::adsrp::{self, ADSRp};
use talkers::bounded_sinusoidal::{self, BoundedSinusoidal};
use talkers::bounded_square::{self, BoundedSquare};
use talkers::env_shaper::{self, EnvShaper};
use talkers::fuzz::{self, Fuzz};
use talkers::hub::{self, Hub};
use talkers::lv2::Lv2;
use talkers::math::{self, Average, Product, Sum, TanhSum};
use talkers::parabolic::{self, Parabolic};
use talkers::round::{self, Round};
use talkers::second_degree_frequency_progression::{self, SecondDegreeFrequencyProgression};
use talkers::sinusoidal::{self, Sinusoidal};
use talkers::sinusoidal_fptg::{self, SinusoidalFPTG};
use talkers::square::{self, Square};
use talkers::tseq::tseq::{self, Tseq};

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
                let mut categories = Vec::new();
                for classe in plugin.classes() {
                    let category = match classe.find(" Plugin") {
                        Some(ep) => classe.get(..ep).unwrap(),
                        None => classe,
                    };
                    categories.push(category.to_string());
                }

                handlers.insert(
                    plugin.uri(),
                    PluginHandler {
                        base: TalkerHandlerBase::with_multi_categories(
                            categories,
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
            PluginsManager::tkr_hr_kv(Accumulator::descriptor()),
            PluginsManager::tkr_hr_kv(ADSRp::descriptor()),
            PluginsManager::tkr_hr_kv(Average::descriptor()),
            PluginsManager::tkr_hr_kv(BoundedSinusoidal::descriptor()),
            PluginsManager::tkr_hr_kv(BoundedSquare::descriptor()),
            PluginsManager::tkr_hr_kv(EnvShaper::descriptor()),
            PluginsManager::tkr_hr_kv(Fuzz::descriptor()),
            PluginsManager::tkr_hr_kv(Hub::descriptor()),
            PluginsManager::tkr_hr_kv(Parabolic::descriptor()),
            PluginsManager::tkr_hr_kv(Product::descriptor()),
            PluginsManager::tkr_hr_kv(Round::descriptor()),
            PluginsManager::tkr_hr_kv(SecondDegreeFrequencyProgression::descriptor()),
            PluginsManager::tkr_hr_kv(Sinusoidal::descriptor()),
            PluginsManager::tkr_hr_kv(SinusoidalFPTG::descriptor()),
            PluginsManager::tkr_hr_kv(Square::descriptor()),
            PluginsManager::tkr_hr_kv(Sum::descriptor()),
            PluginsManager::tkr_hr_kv(TanhSum::descriptor()),
            PluginsManager::tkr_hr_kv(Tseq::descriptor()),
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
        if model == abs_sine::MODEL {
            Ok(rtalker!(AbsSine::new()?))
        } else if model == accumulator::MODEL {
            Ok(rtalker!(Accumulator::new()?))
        } else if model == adsrp::MODEL {
            Ok(rtalker!(ADSRp::new()?))
        } else if model == hub::MODEL {
            Ok(rtalker!(Hub::new()?))
        } else if model == bounded_sinusoidal::MODEL {
            Ok(rtalker!(BoundedSinusoidal::new()?))
        } else if model == bounded_square::MODEL {
            Ok(rtalker!(BoundedSquare::new()?))
        } else if model == math::AVERAGE_MODEL {
            Ok(rtalker!(Average::new()?))
        } else if model == env_shaper::MODEL {
            Ok(rtalker!(EnvShaper::new()?))
        } else if model == fuzz::MODEL {
            Ok(rtalker!(Fuzz::new()?))
        } else if model == parabolic::MODEL {
            Ok(rtalker!(Parabolic::new()?))
        } else if model == math::PRODUCT_MODEL {
            Ok(rtalker!(Product::new()?))
        } else if model == round::MODEL {
            Ok(rtalker!(Round::new()?))
        } else if model == second_degree_frequency_progression::MODEL {
            Ok(rtalker!(SecondDegreeFrequencyProgression::new(
                110., 0., 1., 1.
            )?))
        } else if model == sinusoidal::MODEL {
            Ok(rtalker!(Sinusoidal::new()?))
        } else if model == sinusoidal_fptg::MODEL {
            Ok(rtalker!(SinusoidalFPTG::new()?))
        } else if model == square::MODEL {
            Ok(rtalker!(Square::new()?))
        } else if model == math::SUM_MODEL {
            Ok(rtalker!(Sum::new()?))
        } else if model == math::TANH_SUM_MODEL {
            Ok(rtalker!(TanhSum::new()?))
        } else if model == tseq::MODEL {
            Ok(rtalker!(Tseq::new()?))
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
            let plugin_categories_count = ph.base.categories.len();

            for category in &ph.base.categories {
                if plugin_categories_count == 1 || category != "Plugin" {
                    match categories_map.get_mut(category) {
                        Some(category_talkers) => {
                            category_talkers.push((ph.base.label(), model));
                        }
                        None => {
                            let mut category_talkers = Vec::new();
                            category_talkers.push((ph.base.label(), model));
                            categories_map.insert(category, category_talkers);
                        }
                    }
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
            println!("Plugin {}", ph.base.model());

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
