use urid::{URIDOf, features::URIDMap};
use core::uri::UriBound;

pub trait URIDCache {
    fn new(mapper: &URIDMap) -> Self;
}

pub trait URIDCacheMapping<U: UriBound>: URIDCache {
    fn get_urid(&self) -> URIDOf<U>;
}
