use core::PortType;
use atom::header::Atom;
use std::ptr::NonNull;
use std::ops::Deref;

pub struct AtomPort {
    pointer: NonNull<Atom>
}

impl PortType for AtomPort {
    const NAME: &'static str = "Atom";
    const URI: &'static [u8] = ::lv2_sys::LV2_ATOM__AtomPort;

    type InputPortType = Self;
    type OutputPortType = (); // TODO

    #[inline]
    unsafe fn input_from_raw(pointer: NonNull<()>, _sample_count: u32) -> Self::InputPortType {
        Self { pointer: pointer.cast() }
    }

    unsafe fn output_from_raw(_pointer: NonNull<()>, _sample_count: u32) -> Self::OutputPortType {
       unimplemented!()
    }
}

impl Deref for AtomPort {
    type Target = Atom;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { self.pointer.as_ref() }
    }
}
