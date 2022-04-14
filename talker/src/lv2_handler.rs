use audio_format;

use livi;

use std::sync::Once;
use std::sync::{Arc, Mutex};

const MIN_BLOCK_SIZE: usize = 1;
const MAX_BLOCK_SIZE: usize = audio_format::DEFAULT_CHUNK_SIZE;

pub struct Lv2Handler {
    pub world: livi::World,
    pub features: Arc<livi::Features>,
}

impl Lv2Handler {
    pub fn new() -> Lv2Handler {
        let world = livi::World::new();

        let features = world.build_features(livi::FeaturesBuilder {
            min_block_length: MIN_BLOCK_SIZE,
            max_block_length: MAX_BLOCK_SIZE,
            worker_manager: std::sync::Arc::new(livi::WorkerManager::default()),
        });

        Lv2Handler { world, features }
    }
}

pub type MLv2Handler = Mutex<Lv2Handler>;

static mut OPT_INSTANCE: Option<MLv2Handler> = None;
static INIT: Once = Once::new();

pub fn visit<F, R>(mut f: F) -> Result<R, failure::Error>
where
    F: FnMut(&Lv2Handler) -> Result<R, failure::Error>,
{
    INIT.call_once(|| {
        let oinstance = Some(Mutex::new(Lv2Handler::new()));
        unsafe {
            OPT_INSTANCE = oinstance;
        }
    });

    let minstance;
    unsafe {
        match &OPT_INSTANCE {
            Some(minst) => {
                minstance = minst;
            }
            None => {
                return Err(failure::err_msg(
                    "lv2_handler::visite failed on instance acces!",
                ))
            }
        }
    }

    let res = match minstance.lock() {
        Ok(instance) => f(&instance),
        Err(e) => Err(failure::err_msg(format!(
            "lv2_handler::visite failed on lock : {}",
            e
        ))),
    };
    res
}
