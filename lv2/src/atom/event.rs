use ::units::Unit;
use std::marker::PhantomData;
use atom::header::{Atom, RawAtomType};
use std::mem;
use urid::URIDCacheMapping;

#[repr(C)]
pub struct Event<U: Unit> {
    inner: ::lv2_sys::LV2_Atom_Event,
    _unit: PhantomData<U>
}

impl<U: Unit> Event<U> {
    #[inline]
    pub(crate) unsafe fn from_raw<'a>(raw: *const ::lv2_sys::LV2_Atom_Event) -> &'a Event<U> {
        &*(raw as *const _)
    }

    #[inline]
    pub fn body(&self) -> &Atom {
        Atom::from_raw(&self.inner.body)
    }

    #[inline]
    pub fn read_as<T: RawAtomType, C: URIDCacheMapping<T>>(&self, cache: &C) -> Option<&T> {
        self.body().read_as(cache)
    }

    #[inline]
    pub fn time(&self) -> U {
        unsafe { mem::transmute_copy(&self.inner.time) }
    }

    #[inline]
    pub fn header_size() -> usize {
        mem::size_of::<::lv2_sys::LV2_Atom_Event__bindgen_ty_1>()
    }
}
