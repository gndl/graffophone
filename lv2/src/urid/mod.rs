use core::uri::Uri;
use std::error::Error;
use std::num::NonZeroU32;

pub type URID = NonZeroU32;
mod cache;
mod simple_mapper;
mod uridof;

pub use self::cache::*;
pub use self::simple_mapper::*;
pub use self::uridof::*;

pub mod features;

pub trait URIDMapper {
    fn map(&self, uri: &Uri) -> Result<URID, Box<dyn Error>>;
    fn unmap(&self, urid: URID) -> Option<&Uri>;
}

#[cfg(test)]
mod test {
    use std::mem::{align_of, size_of};
    use urid::URID;

    #[test]
    fn test_urid_layout() {
        assert_eq!(align_of::<URID>(), align_of::<::lv2_sys::LV2_URID>());
        assert_eq!(size_of::<URID>(), size_of::<::lv2_sys::LV2_URID>());
    }
}
