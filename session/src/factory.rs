use std::sync::{LazyLock, Mutex};

use talker::audio_format::AudioFormat;
use talker::identifier::{Id, Identifiable};
use talker::talker::RTalker;

use crate::audiofile_output::AudioFileOutput;
use crate::{audiofile_output, feedback};
use crate::feedback::Feedback;
use crate::mixer::{Mixer, RMixer};
use crate::output::{self, ROutput};
use crate::plugins_manager::PluginsManager;

#[derive(PartialEq, Debug, Clone)]
pub enum OutputParam {
    File(String, usize, String, String),
    Jack,
}

pub struct Factory {
    plugins_manager: PluginsManager,
}

pub type RFactory = Mutex<Factory>;

static INSTANCE: LazyLock<RFactory> = LazyLock::new(|| Mutex::new(Factory::new()));

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
        id: Id,
        name: &str,
        effective: bool,
    ) -> Result<RTalker, failure::Error> {
        let tkr = self.plugins_manager.make_talker(model, id, effective, false)?;
        tkr.set_name(name);
        Ok(tkr)
    }

    pub fn add_talker(
        &self,
        model: &str,
        id: Id,
        effective: bool,
    ) -> Result<RTalker, failure::Error> {
        self.plugins_manager.make_talker(model, id, effective, true)
    }

    pub fn make_mixer(
        id: Id,
        name: &str,
        oparent: Option<&RMixer>,
        outputs: Vec<ROutput>,
    ) -> Result<RMixer, failure::Error> {
        Mixer::new_ref(id, name, oparent, outputs)
    }

    pub fn make_output(
        model: &str,
        id: Id,
        name: &str,
        configuration: Option<&str>,
    ) -> Result<ROutput, failure::Error> {
        if model == audiofile_output::MODEL {
            match configuration {
                Some(conf) => {
                    let output = AudioFileOutput::from_backup(id, AudioFormat::chunk_size(), conf)?;

                    output.borrow().set_name(name);

                    Ok(output)
                },
                None => Err(failure::err_msg(format!("{} output need configuration date!", model))),
            }
        } else if model == feedback::MODEL {
            let output = Feedback::new_ref(AudioFormat::chunk_size())?;

            Ok(output)
        } else {
            Err(failure::err_msg(format!("Unknown output model {}!", model)))
        }
    }

    pub fn make_outputs(mixer_id: Id, outputs_params: &Vec<OutputParam>) -> Result<Vec<ROutput>, failure::Error> {
        let in_sample_rate = AudioFormat::sample_rate();
        let mut outputs = Vec::with_capacity(outputs_params.len());

        for (idx, op) in outputs_params.iter().enumerate() {
            match op {
                OutputParam::File(codec, out_sample_rate, channel_layout, file_path) => {
                    let output = AudioFileOutput::new_ref(
                        output::produce_output_id(mixer_id, idx),
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

    pub fn visit<F, R>(mut f: F) -> Result<R, failure::Error>
    where
        F: FnMut(&Factory) -> Result<R, failure::Error>,
    {
        let res = match (*INSTANCE).lock() {
            Ok(factory) => f(&factory),
            Err(_) => Err(failure::err_msg("Factory::visite failed on lock!")),
        };
        res
    }
}
