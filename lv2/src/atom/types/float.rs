use urid::URID;
use atom::header::AtomType;
use core::uri::UriBound;
use atom::header::RawAtomType;
use atom::ToAtom;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct AtomFloat {
    inner: ::lv2_sys::LV2_Atom_Float
}

impl AtomFloat {
    pub fn size() -> u32 {
        ::std::mem::size_of::<f32>() as u32
    }

    #[inline]
    unsafe fn new_without_urid(value: f32) -> AtomFloat {
        AtomFloat {
            inner: ::lv2_sys::LV2_Atom_Float {
                body: value,
                atom: ::lv2_sys::LV2_Atom {
                    size: AtomFloat::size(),
                    type_: 0
                }
            }
        }
    }

    #[inline]
    pub unsafe fn new(urid: URID, value: f32) -> AtomFloat {
        AtomFloat {
            inner: ::lv2_sys::LV2_Atom_Float {
                body: value,
                atom: ::lv2_sys::LV2_Atom {
                    size: AtomFloat::size(),
                    type_: urid.get()
                }
            }
        }
    }
}

impl AtomType for AtomFloat {}
unsafe impl RawAtomType for AtomFloat {}
unsafe impl UriBound for AtomFloat {
    const URI: &'static [u8] = ::lv2_sys::LV2_ATOM__Float;
}

impl ToAtom for f32 {
    type AtomType = AtomFloat;

    fn to_atom(&self) -> <Self as ToAtom>::AtomType {
        unsafe { AtomFloat::new_without_urid(*self) }
    }
}