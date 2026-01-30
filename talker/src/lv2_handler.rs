use std::sync::{Arc, LazyLock, Mutex};
use std::sync::mpsc::{Sender, Receiver};
use std::thread;
use std::ffi::{CString, CStr};
use std::str::FromStr;
use std::collections::HashMap;

use livi::{self, PortType};

use audio_format;

const MIN_BLOCK_SIZE: usize = audio_format::MIN_CHUNK_SIZE;
const MAX_BLOCK_SIZE: usize = audio_format::DEFAULT_CHUNK_SIZE;

enum WorkerOrder {
    Run,
}

pub struct Lv2Handler {
    pub world: livi::World,
    pub features: Arc<livi::Features>,
    workers_sender: Sender<WorkerOrder>,
}

impl Lv2Handler {
    pub fn new() -> Lv2Handler {
        let world = livi::World::new();

        let features = world.build_features(livi::FeaturesBuilder {
            min_block_length: MIN_BLOCK_SIZE,
            max_block_length: MAX_BLOCK_SIZE,
        });

        let worker_manager = features.worker_manager().clone();

        let (workers_sender, workers_receiver): (Sender<WorkerOrder>, Receiver<WorkerOrder>) =
        std::sync::mpsc::channel();

        let _ = thread::spawn(move || {
            loop {
                match workers_receiver.recv() {
                    Ok(_) => worker_manager.run_workers(),
                    Err(_) => break,
                }
            }
        });

        Lv2Handler { world, features, workers_sender}
    }

    pub fn plugin_ui_supported(&self, plugin: &livi::Plugin) -> bool {
        plugin.raw().uis().is_some()
    }
}

static INSTANCE: LazyLock<Mutex<Lv2Handler>> = LazyLock::new(|| Mutex::new(Lv2Handler::new()));

pub fn visit<F, R>(mut f: F) -> Result<R, failure::Error>
where
    F: FnMut(&Lv2Handler) -> Result<R, failure::Error>,
{
    let res = match (*INSTANCE).lock() {
        Ok(instance) => f(&instance),
        Err(e) => Err(failure::err_msg(format!(
            "lv2_handler::visite failed on lock : {}",
            e
        ))),
    };
    res
}

pub fn urid_map(uri: &CStr) -> lv2_raw::LV2Urid {
    visit(|lv2_handler| {
        Ok(lv2_handler.features.urid(&uri))
    }).unwrap()
}
pub fn urid_unmap(urid: lv2_raw::LV2Urid) -> Option<CString> {
    visit(|lv2_handler| {
        Ok(lv2_handler.features.uri(urid).map(|s| CString::from_str(&s).unwrap()))
    }).unwrap()
}

pub fn get_bundle_uri(plugin_uri: &str) -> Result<String, failure::Error> {
    visit(|lv2_handler| {
        match lv2_handler.world.plugin_by_uri(plugin_uri) {
            Some(plugin) => {
                let bundle_node = plugin.raw().bundle_uri();
                let bundle_uri = bundle_node.as_uri().unwrap_or("");
                Ok(bundle_uri.to_string())
            }
            None => Ok("".to_string()),
        }
    })
}

pub fn get_ears_indexes(plugin_uri: &str) -> Result<Vec<usize>, failure::Error> {
    visit(|lv2_handler| {
        match lv2_handler.world.plugin_by_uri(plugin_uri) {
            Some(plugin) => {
                let mut ears_indexes: Vec<usize> = vec![0; plugin.ports().count()];
                let mut ear_idx = 0;

                for port in plugin.ports() {
                    match port.port_type {
                        PortType::ControlInput | PortType::AudioInput | PortType::AtomSequenceInput | PortType::CVInput => {
                            ears_indexes[port.index.0] = ear_idx;
                            ear_idx += 1;
                        },
                        _ => (),
                    }
                }
                Ok(ears_indexes)
            }
            None => Err(failure::err_msg(format!("LV2 plugin {} not found.", plugin_uri))),
        }
    })
}

pub fn get_port_symbol_indexes(plugin_uri: &str) -> Result<HashMap<String, u32>, failure::Error> {
    visit(|lv2_handler| {
        match lv2_handler.world.plugin_by_uri(plugin_uri) {
            Some(plugin) => {
                let mut port_symbol_indexes = HashMap::new();

                for port in plugin.ports() {
                    port_symbol_indexes.insert(port.symbol, port.index.0 as u32);
                }
                Ok(port_symbol_indexes)
            }
            None => Err(failure::err_msg(format!("LV2 plugin {} not found.", plugin_uri))),
        }
    })
}


pub fn run_workers() -> Result<(), failure::Error> {
    match (*INSTANCE).lock() {
        Ok(instance) => instance.workers_sender.send(WorkerOrder::Run).map_err(|e| failure::err_msg(format!("Run LV2 workers : {}", e))),
        Err(e) => Err(failure::err_msg(format!("Run LV2 workers failed on lock : {}", e))),
    }
}
