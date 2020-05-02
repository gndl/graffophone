use std::rc::Rc;
use std::sync::{Arc, Mutex};

use talker::audio_format::AudioFormat;
use talker::identifier::RIdentifier;
use talker::talker::RTalker;
use talker::talker::Talker;

use crate::feedback;
use crate::feedback::Feedback;
use crate::mixer::{Mixer, RMixer};
use crate::output::ROutput;
use crate::plugins_manager::PluginsManager;
use crate::track::Track;

pub struct Factory {
    plugins_manager: PluginsManager,
}

pub type RFactory = Rc<Factory>;

static mut OPT_INSTANCE: Option<Arc<Mutex<Factory>>> = None;

impl Factory {
    pub fn new() -> Factory {
        Self {
            plugins_manager: PluginsManager::new(),
        }
    }

    pub fn new_ref() -> RFactory {
        Rc::new(Factory::new())
    }

    pub fn get_categorized_talkers_label_model(&self) -> Vec<(String, Vec<(String, String)>)> {
        self.plugins_manager.get_categorized_talkers_label_model()
    }

    pub fn make_talker(
        &self,
        model: &str,
        oid: Option<u32>,
        oname: Option<&str>,
    ) -> Result<RTalker, failure::Error> {
        let tkr = self.plugins_manager.make_talker(model)?;
        Factory::set_identity(tkr.borrow().identifier(), oid, oname);
        Ok(tkr)
    }

    pub fn make_track(
        &self,
        oid: Option<u32>,
        oname: Option<&str>,
    ) -> Result<Track, failure::Error> {
        let trk = Track::new();
        Factory::set_identity(trk.identifier(), oid, oname);
        Ok(trk)
    }

    pub fn make_mixer(
        &self,
        oid: Option<u32>,
        oname: Option<&str>,
        tracks: Option<Vec<Track>>,
        outputs: Option<Vec<ROutput>>,
    ) -> Result<RMixer, failure::Error> {
        let rmixer = Mixer::new_ref(tracks, outputs);
        Factory::set_identity(rmixer.borrow_mut().identifier(), oid, oname);
        Ok(rmixer)
    }

    pub fn make_output(
        &self,
        oid: Option<u32>,
        oname: Option<&str>,
        model: &str,
        _attributs: Option<&Vec<(&str, &str, &str)>>,
    ) -> Result<ROutput, failure::Error> {
        if model == feedback::MODEL {
            let output = Feedback::new_ref(2, AudioFormat::chunk_size())?;
            Factory::set_identity(output.borrow().identifier(), oid, oname);
            Ok(output)
        } else {
            Err(failure::err_msg(format!("Unknown output model {}!", model)))
        }
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
        unsafe {
            match &OPT_INSTANCE {
                Some(factory) => {
                    let factory = factory.clone();
                    let factory = factory.lock().unwrap();
                    f(&factory)
                }
                None => {
                    OPT_INSTANCE = Some(Arc::new(Mutex::new(Factory::new())));
                    Factory::visit(f)
                }
            }
        }
    }
}
