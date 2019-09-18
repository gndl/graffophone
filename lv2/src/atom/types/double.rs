use urid::URID;
use atom::header::AtomType;
use core::uri::UriBound;
use atom::header::RawAtomType;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct AtomDouble {
    inner: ::lv2_sys::LV2_Atom_Double
}

impl AtomDouble {
    pub fn size() -> u32 {
        ::std::mem::size_of::<f64>() as u32
    }

    #[inline]
    pub unsafe fn new(urid: URID, value: f64) -> AtomDouble {
        AtomDouble {
            inner: ::lv2_sys::LV2_Atom_Double {
                body: value,
                atom: ::lv2_sys::LV2_Atom {
                    size: AtomDouble::size(),
                    type_: urid.get()
                }
            }
        }
    }
}

impl AtomType for AtomDouble {}
unsafe impl RawAtomType for AtomDouble {}
unsafe impl UriBound for AtomDouble {
    const URI: &'static [u8] = ::lv2_sys::LV2_ATOM__Double;
}