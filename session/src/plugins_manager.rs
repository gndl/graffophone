use std::collections::HashMap;
use std::iter::Extend;

use talker::lv2_handler;
use talker::rtalker;
use talker::talker::{RTalker, TalkerBase, TalkerCab};
use talker::talker_handler::TalkerHandlerBase;
use talkers::abs_sine::{self, AbsSine};
use talkers::accumulator::{self, Accumulator};
use talkers::adsrp::{self, ADSRp};
use talkers::audiofile_input::{self, AudioFileInput};
use talkers::bounded_sinusoidal::{self, BoundedSinusoidal};
use talkers::bounded_square::{self, BoundedSquare};
use talkers::envelope_shaper::{self, EnvelopeShaper};
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
            PluginsManager::tkr_hr_kv(AudioFileInput::descriptor()),
            PluginsManager::tkr_hr_kv(Average::descriptor()),
            PluginsManager::tkr_hr_kv(BoundedSinusoidal::descriptor()),
            PluginsManager::tkr_hr_kv(BoundedSquare::descriptor()),
            PluginsManager::tkr_hr_kv(EnvelopeShaper::descriptor()),
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

    pub fn make_internal_talker(&self, model: &String, base: TalkerBase) -> Result<RTalker, failure::Error> {
        if model == abs_sine::MODEL {
            Ok(rtalker!(AbsSine::new(base)?))
        } else if model == accumulator::MODEL {
            Ok(rtalker!(Accumulator::new(base)?))
        } else if model == adsrp::MODEL {
            Ok(rtalker!(ADSRp::new(base)?))
        } else if model == audiofile_input::MODEL {
            Ok(rtalker!(AudioFileInput::new(base)?))
        } else if model == hub::MODEL {
            Ok(rtalker!(Hub::new(base)?))
        } else if model == bounded_sinusoidal::MODEL {
            Ok(rtalker!(BoundedSinusoidal::new(base)?))
        } else if model == bounded_square::MODEL {
            Ok(rtalker!(BoundedSquare::new(base)?))
        } else if model == math::AVERAGE_MODEL {
            Ok(rtalker!(Average::new(base)?))
        } else if model == envelope_shaper::MODEL {
            Ok(rtalker!(EnvelopeShaper::new(base)?))
        } else if model == fuzz::MODEL {
            Ok(rtalker!(Fuzz::new(base)?))
        } else if model == parabolic::MODEL {
            Ok(rtalker!(Parabolic::new(base)?))
        } else if model == math::PRODUCT_MODEL {
            Ok(rtalker!(Product::new(base)?))
        } else if model == round::MODEL {
            Ok(rtalker!(Round::new(base)?))
        } else if model == second_degree_frequency_progression::MODEL {
            Ok(rtalker!(SecondDegreeFrequencyProgression::new(110., 0., 1., 1., base)?))
        } else if model == sinusoidal::MODEL {
            Ok(rtalker!(Sinusoidal::new(base)?))
        } else if model == sinusoidal_fptg::MODEL {
            Ok(rtalker!(SinusoidalFPTG::new(base)?))
        } else if model == square::MODEL {
            Ok(rtalker!(Square::new(base)?))
        } else if model == math::SUM_MODEL {
            Ok(rtalker!(Sum::new(base)?))
        } else if model == math::TANH_SUM_MODEL {
            Ok(rtalker!(TanhSum::new(base)?))
        } else if model == tseq::MODEL {
            Ok(rtalker!(Tseq::new(base)?))
        } else {
            Err(failure::err_msg("Unknown talker MODEL"))
        }
    }

    pub fn mk_tkr(&self, ph: &PluginHandler, effective: bool) -> Result<RTalker, failure::Error> {
        
        match &ph.plugin_type {
            PluginType::Lv2 => lv2_handler::visit(|lv2_handler| {
                let base = TalkerBase::new(ph.base.label(), ph.base.model(), effective);
                Ok(rtalker!(Lv2::new(lv2_handler, ph.base.model(), base)?))
            }),
            PluginType::Internal => {
                let base = TalkerBase::new(ph.base.label(), ph.base.model(), effective);
                self.make_internal_talker(ph.base.model(), base)
            },
        }
    }

    pub fn make_talker(&self, model: &str, effective: bool) -> Result<RTalker, failure::Error> {
        match self.handlers.get(model) {
            Some(ph) => self.mk_tkr(ph, effective),
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
/*
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
    */
}
