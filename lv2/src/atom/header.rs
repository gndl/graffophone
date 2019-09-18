use urid::{URID, URIDCacheMapping};
use core::uri::UriBound;
use std::mem;
use atom::types::{AtomSequence, UnknownAtomSequence};
use units::Unit;

#[repr(C)]
pub struct Atom {
    inner: ::lv2_sys::LV2_Atom
}

impl Atom {
    #[inline]
    pub(crate) unsafe fn add_size(&mut self, size: u32) {
        self.inner.size += size
    }

    #[inline]
    pub(crate) fn from_raw(atom: &::lv2_sys::LV2_Atom) -> &Atom {
        unsafe { &*(atom as *const _ as *const Atom) }
    }

    #[inline]
    pub unsafe fn new_with_type(size: u32, type_: URID) -> Atom {
        Atom { inner: ::lv2_sys::LV2_Atom { size, type_: type_.get() } }
    }

    #[inline]
    pub unsafe fn new(size: u32) -> Atom {
        Atom { inner: ::lv2_sys::LV2_Atom { size, type_: 0 } }
    }

    #[inline]
    pub fn read_as<T: RawAtomType, C: URIDCacheMapping<T>>(&self, cache: &C) -> Option<&T> {
        if cache.get_urid().urid().get() == self.inner.type_ {
            Some(unsafe { mem::transmute(self) })
        } else {
            None
        }
    }

    pub fn read_sequence<
        U: Unit,
        C: URIDCacheMapping<UnknownAtomSequence> + URIDCacheMapping<U>
    >(&self, cache: &C) -> Option<&AtomSequence<U>> {
        self.read_as::<UnknownAtomSequence, _>(cache)
            .and_then(|s| s.with_timestamps(cache))
    }
}

pub unsafe trait RawAtomType: AtomType {}

pub trait AtomType: Sized + UriBound {
    #[inline]
    fn get_header(&self) -> &Atom {
        unsafe { &*(self as *const _ as *const Atom) }
    }

    #[inline]
    fn get_header_mut(&mut self) -> &mut Atom {
        unsafe { &mut *(self as *mut _ as *mut Atom) }
    }

    #[inline]
    fn get_type(&self) -> URID {
        unsafe { URID::new_unchecked(self.get_header().inner.type_) }
    }

    #[inline]
    fn get_size(&self) -> u32 {
        self.get_header().inner.size
    }

    #[inline]
    fn get_total_size(&self) -> usize {
        self.get_size() as usize + mem::size_of::<Atom>()
    }

    #[inline]
    fn update_type_id<C>(&mut self, cache: &C) where C: URIDCacheMapping<Self> {
        self.get_header_mut().inner.type_ = cache.get_urid().urid().get();
    }
}

unsafe impl UriBound for Atom {
    const URI: &'static [u8] = ::lv2_sys::LV2_ATOM_URI;
}
impl AtomType for Atom {}

// TODO: make tests
