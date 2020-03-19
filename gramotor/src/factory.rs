use std::rc::Rc;

use granode::audio_format::AudioFormat;
use granode::identifier::RIdentifier;
use granode::talker::RTalker;
use granode::talker::Talker;

use crate::mixer::{Mixer, RMixer};
use crate::output::ROutput;
use crate::playback;
use crate::playback::Playback;
use crate::plugins_manager::PluginsManager;
use crate::track::Track;

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
        if model != playback::MODEL {
            return Err(failure::err_msg(format!("Unknown output model {}!", model)));
        }
        let output = Playback::new_ref(2, AudioFormat::chunk_size())?;
        Factory::set_identity(output.borrow().identifier(), oid, oname);
        Ok(output)
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
}
