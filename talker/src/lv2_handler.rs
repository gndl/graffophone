use std::sync::{Arc, LazyLock, Mutex};
use std::sync::mpsc::{Sender, Receiver};
use std::thread;

use audio_format;
use livi;


const MIN_BLOCK_SIZE: usize = 1;
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

pub fn run_workers() -> Result<(), failure::Error> {
    match (*INSTANCE).lock() {
        Ok(instance) => instance.workers_sender.send(WorkerOrder::Run).map_err(|e| failure::err_msg(format!("Run LV2 workers : {}", e))),
        Err(e) => Err(failure::err_msg(format!("Run LV2 workers failed on lock : {}", e))),
    }
}
