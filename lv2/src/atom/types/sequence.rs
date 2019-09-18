use core::uri::UriBound;
use atom::header::AtomType;
use urid::{URID, URIDCacheMapping};
use units::{Unit, units::Frame};
use std::marker::PhantomData;
use std::mem;
use atom::event::Event;
use atom::header::RawAtomType;

#[repr(C)]
pub struct UnknownAtomSequence {
    inner: ::lv2_sys::LV2_Atom_Sequence
}

impl UnknownAtomSequence {
    #[inline]
    pub fn event_timestamp_type(&self) -> Option<URID> {
        URID::new(self.inner.body.unit)
    }

    #[inline]
    pub fn with_timestamps<U: Unit, C: URIDCacheMapping<U>>(&self, cache: &C) -> Option<&AtomSequence<U>> {
        if let Some(timestamp_type) = self.event_timestamp_type() {
            if timestamp_type == cache.get_urid().urid() {
                Some(unsafe { mem::transmute::<_, &AtomSequence<U>>(self) })
            } else {
                None
            }
        } else {
            if U::URI == Frame::URI {
                Some(unsafe { mem::transmute::<_, &AtomSequence<U>>(self) })
            } else {
                None
            }
        }
    }
}

unsafe impl UriBound for UnknownAtomSequence {
    const URI: &'static [u8] = ::lv2_sys::LV2_ATOM__Sequence;
}
impl AtomType for UnknownAtomSequence {}
unsafe impl RawAtomType for UnknownAtomSequence {}

#[repr(C)]
pub struct AtomSequence<U: Unit> {
    inner: ::lv2_sys::LV2_Atom_Sequence,
    _unit: PhantomData<U>
}

// TODO: Size tests for transmute

impl<U: Unit> AtomSequence<U> {
    #[inline]
    pub(crate) fn new_header<C: URIDCacheMapping<UnknownAtomSequence> + URIDCacheMapping<U>>(cache: &C) -> Self {
        Self {
            _unit: PhantomData,
            inner: ::lv2_sys::LV2_Atom_Sequence {
                atom: ::lv2_sys::LV2_Atom {
                    size: 8,
                    type_: URIDCacheMapping::<UnknownAtomSequence>::get_urid(cache).urid().get()
                },
                body: ::lv2_sys::LV2_Atom_Sequence_Body {
                    unit: URIDCacheMapping::<U>::get_urid(cache).urid().get(),
                    pad: 0
                }
            }
        }
    }

    #[inline]
    pub fn event_timestamp_type(&self) -> Option<URID> {
        URID::new(self.inner.body.unit)
    }

    #[inline]
    pub fn iter(&self) -> AtomSequenceIter<U> {
        AtomSequenceIter::new(self)
    }
}

unsafe impl<U: Unit> UriBound for AtomSequence<U> {
    const URI: &'static [u8] = ::lv2_sys::LV2_ATOM__Sequence;
}
impl<U: Unit> AtomType for AtomSequence<U> {}

pub struct AtomSequenceIter<'a, U: Unit + 'a> {
    sequence: &'a AtomSequence<U>,
    pointer: *mut ::lv2_sys::LV2_Atom_Event
}

impl<'a, U: Unit + 'a> AtomSequenceIter<'a, U> {
    #[inline]
    fn new(sequence: &'a AtomSequence<U>) -> Self {
        Self {
            sequence,
            pointer: unsafe { ::lv2_sys::lv2_atom_sequence_begin(&sequence.inner.body) }
        }
    }
}

impl<'a, U: Unit + 'a> Iterator for AtomSequenceIter<'a, U> {
    type Item = &'a Event<U>;

    #[inline]
    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        let is_end = unsafe { ::lv2_sys::lv2_atom_sequence_is_end(
            &self.sequence.inner.body,
            self.sequence.inner.atom.size,
            self.pointer)
        };

        if is_end { return None }

        let value = unsafe { Event::from_raw(self.pointer) };

        self.pointer = unsafe { ::lv2_sys::lv2_atom_sequence_next(self.pointer) };

        Some(value)
    }
}

impl<'a, U: Unit + 'a> IntoIterator for &'a AtomSequence<U> {
    type Item = &'a Event<U>;
    type IntoIter = AtomSequenceIter<'a, U>;

    #[inline]
    fn into_iter(self) -> <Self as IntoIterator>::IntoIter {
        AtomSequenceIter::new(self)
    }
}