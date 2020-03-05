use std::rc::Rc;

use gpplugin::audio_format::AudioFormat;
use gpplugin::talker::RTalker;

use crate::output::ROutput;
use crate::playback_output::Playback;
use crate::plugins_manager::PluginsManager;

pub struct Factory {
    plugins_manager: PluginsManager,
}

pub type RFactory = Rc<Factory>;

impl Factory {
    pub fn new() -> Factory {
        let mut plugins_manager = PluginsManager::new();
        plugins_manager.load_plugins();
        Self { plugins_manager }
    }

    pub fn new_ref() -> RFactory {
        Rc::new(Factory::new())
    }

    pub fn make_talker(&self, model: &str, name: Option<&str>) -> Result<RTalker, failure::Error> {
        self.plugins_manager.make_talker(model, name)
    }

    pub fn make_output(
        &self,
        _name: &str,
        _kind: &str,
        _attributs: &Vec<(&str, &str, &str)>,
    ) -> Result<ROutput, failure::Error> {
        Playback::new_ref(2, AudioFormat::chunk_size())
    }
}
