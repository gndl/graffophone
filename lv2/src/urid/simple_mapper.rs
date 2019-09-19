use std::cell::UnsafeCell;
use std::error::Error;
use std::num::NonZeroU32;

use core::uri::{Uri, UriBuf};
use urid::{URIDMapper, URID};

#[derive(Default)]
pub struct SimpleMapper {
    uris: UnsafeCell<Vec<UriBuf>>,
}

impl SimpleMapper {
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }
}

impl URIDMapper for SimpleMapper {
    fn map(&self, uri: &Uri) -> Result<URID, Box<dyn Error>> {
        let uris = unsafe { &mut *self.uris.get() }; // Please actually better handle borrows than that, this is just a test
        let index = uris
            .iter()
            .enumerate()
            .find(|(_, u)| &uri == u)
            .map(|(i, _)| i);
        Ok(NonZeroU32::new(match index {
            Some(i) => (i + 1) as u32,
            None => {
                uris.push(uri.into());
                uris.len() as u32
            }
        })
        .unwrap())
    }

    fn unmap(&self, urid: URID) -> Option<&Uri> {
        let uris = unsafe { &*self.uris.get() };
        uris.get(urid.get() as usize - 1)
            .map(::std::borrow::Borrow::borrow)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use core::uri::IntoUri;
    use urid::features::{URIDMap, URIDUnmap};

    #[test]
    fn urid_mapper() {
        let mapper = SimpleMapper::new();
        let feature_map = URIDMap::new(&mapper);
        let feature_unmap = URIDUnmap::new(&mapper);

        let uri = ::std::ffi::CString::new("Hello, world!").unwrap();

        assert_eq!(feature_map.map(&uri).unwrap().get(), 1);
        assert_eq!(
            feature_map.map(&"Hello, world!".into_uri()).unwrap().get(),
            1
        );
        assert_eq!(
            feature_unmap.unmap(NonZeroU32::new(1).unwrap()).unwrap(),
            &uri
        );
    }
}
