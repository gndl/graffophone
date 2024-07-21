use std::sync::Once;
use std::sync::{Arc, Mutex};

use talker::audio_format::AudioFormat;
use talker::identifier::RIdentifier;
use talker::talker::RTalker;

use crate::audiofile_output::AudioFileOutput;
use crate::{audiofile_output, feedback};
use crate::feedback::Feedback;
use crate::mixer::{Mixer, RMixer};
use crate::output::ROutput;
use crate::plugins_manager::PluginsManager;

#[derive(PartialEq, Debug, Clone)]
pub enum OutputParam {
    File(String, usize, String, String),
    Jack,
}

pub struct Factory {
    plugins_manager: PluginsManager,
}

pub type RFactory = Arc<Mutex<Factory>>;

static mut OPT_INSTANCE: Option<RFactory> = None;
static INIT: Once = Once::new();

fn provide_instance() -> Result<RFactory, failure::Error> {
    INIT.call_once(|| {
        let oinstance = Some(Arc::new(Mutex::new(Factory::new())));
        unsafe {
            OPT_INSTANCE = oinstance;
        }
    });

    unsafe {
        match &OPT_INSTANCE {
            Some(instance) => Ok(instance.clone()),
            None => Err(failure::err_msg(
                "Factory::visite failed on instance acces!",
            )),
        }
    }
}

impl Factory {
    pub fn new() -> Factory {
        Self {
            plugins_manager: PluginsManager::new(),
        }
    }

    pub fn get_categorized_talkers_label_model(&self) -> Vec<(String, Vec<(String, String)>)> {
        self.plugins_manager.get_categorized_talkers_label_model()
    }

    pub fn make_talker(
        &self,
        model: &str,
        oid: Option<u32>,
        oname: Option<&str>, effective: bool,
    ) -> Result<RTalker, failure::Error> {
        let tkr = self.plugins_manager.make_talker(model, effective)?;
        Factory::set_identity(tkr.identifier(), oid, oname);
        Ok(tkr)
    }

    pub fn make_mixer(
        id: u32,
        name: &str,
        oparent: Option<&RMixer>,
        outputs: Vec<ROutput>,
    ) -> Result<RMixer, failure::Error> {
        let rmixer = Mixer::new_ref(oparent, outputs)?;
        Factory::set_identity(rmixer.borrow().identifier(), Some(id), Some(name));
        Ok(rmixer)
    }

    pub fn make_output(
        model: &str,
        oid: Option<u32>,
        oname: Option<&str>,
        configuration: Option<&str>,
    ) -> Result<ROutput, failure::Error> {

        if model == audiofile_output::MODEL {
            match configuration {
                Some(conf) => {
                    let output = AudioFileOutput::from_backup(AudioFormat::chunk_size(), conf)?;
                    Factory::set_identity(output.borrow().identifier(), oid, oname);
                    Ok(output)
                },
                None => Err(failure::err_msg(format!("{} output need configuration date!", model))),
            }
        } else if model == feedback::MODEL {
            let output = Feedback::new_ref(AudioFormat::chunk_size())?;
            Factory::set_identity(output.borrow().identifier(), oid, oname);
            Ok(output)
        } else {
            Err(failure::err_msg(format!("Unknown output model {}!", model)))
        }
    }

    pub fn make_outputs(outputs_params: &Vec<OutputParam>) -> Result<Vec<ROutput>, failure::Error> {
        let in_sample_rate = AudioFormat::sample_rate();
        let mut outputs = Vec::with_capacity(outputs_params.len());

        for op in outputs_params {
            match op {
                OutputParam::File(codec, out_sample_rate, channel_layout, file_path) => {
                    let output = AudioFileOutput::new_ref(
                        codec.as_str(),
                        in_sample_rate,
                        *out_sample_rate,
                        channel_layout,
                        file_path.as_str())?;

                        outputs.push(output);
                },
                _ => (),
            }
        }
        Ok(outputs)
    }

    fn set_identity(identifier: &RIdentifier, oid: Option<u32>, oname: Option<&str>) {
        match oid {
            Some(id) => identifier.borrow_mut().set_id(id),
            None => (),
        };
        match oname {
            Some(name) => identifier.borrow_mut().set_name(name),
            None => (),
        };
    }

    pub fn visit<F, R>(mut f: F) -> Result<R, failure::Error>
    where
        F: FnMut(&Factory) -> Result<R, failure::Error>,
    {
        let instance = provide_instance()?;

        let res = match instance.lock() {
            Ok(factory) => f(&factory),
            Err(_) => Err(failure::err_msg("Factory::visite failed on lock!")),
        };
        res
    }
}
