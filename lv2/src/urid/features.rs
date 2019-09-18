use core::Feature;
use urid::{URID, URIDMapper};
use std::os::raw::{c_void, c_char};
use std::ffi::CStr;
use std::fmt;
use std::error::Error;
use core::uri::AsUriRef;
use core::uri::Uri;
use std::num::NonZeroU32;
use urid::cache::URIDCache;

#[derive(Debug)]
struct MapperPanickedError;

impl fmt::Display for MapperPanickedError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt("Mapper panicked", f)
    }
}

impl Error for MapperPanickedError {}

unsafe extern "C" fn urid_map<T: URIDMapper>(handle: ::lv2_sys::LV2_URID_Map_Handle, uri: *const c_char) -> ::lv2_sys::LV2_URID {
    #[inline(always)]
    unsafe fn inner_urid_map<T: URIDMapper>(handle: ::lv2_sys::LV2_URID_Map_Handle, uri: *const c_char) -> Result<URID, Box<Error>> {
        let value = ::std::panic::catch_unwind(
            ||(&*(handle as *const T)).map( Uri::from_cstr(CStr::from_ptr(uri))?)
        ).map_err(|_| MapperPanickedError)?;
        value
    }

    inner_urid_map::<T>(handle, uri)
        .map(NonZeroU32::get)
        .unwrap_or_else(|e| {
        eprintln!("Error in LV2 URID mapper: {}", e);
        0
    })
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct URIDMap {
    inner: ::lv2_sys::LV2_URID_Map
}

unsafe impl Feature for URIDMap {
    const URI: &'static [u8] = ::lv2_sys::LV2_URID_MAP_URI;
}

impl URIDMap {
    #[inline]
    pub fn new<T: URIDMapper>(mapper: &T) -> URIDMap { // FIXME: mapper needs an additional lifetime check here
        URIDMap {
            inner: ::lv2_sys::LV2_URID_Map {
                handle: mapper as *const T as *const c_void as *mut c_void,
                map: Some(urid_map::<T>)
            }
        }
    }

    pub fn map<T: AsUriRef + ?Sized>(&self, uri: &T) -> Option<URID> {
        let inner_map = self.inner.map.expect("Inner LV2 URID map function is null");
        let uri = uri.as_uri().unwrap();
        NonZeroU32::new(unsafe { inner_map(self.inner.handle, uri.as_ptr()) })
    }

    #[inline]
    pub fn to_cache<C: URIDCache>(&self) -> C {
        C::new(self)
    }
}

unsafe extern "C" fn urid_unmap<T: URIDMapper>(handle: ::lv2_sys::LV2_URID_Unmap_Handle, urid: ::lv2_sys::LV2_URID) -> *const c_char {
    #[inline(always)]
    unsafe fn inner_urid_unmap<'a, T: URIDMapper + 'a>(handle: ::lv2_sys::LV2_URID_Map_Handle, urid: ::lv2_sys::LV2_URID) -> Result<Option<&'a Uri>, Box<Error>> {
        ::std::panic::catch_unwind(|| {
            (&*(handle as *const T)).unmap(NonZeroU32::new(urid).unwrap()) //TODO: handle error
        }).map_err(|_| MapperPanickedError.into())
    }

    inner_urid_unmap::<T>(handle, urid)
        .map(|uri| uri.map_or(::std::ptr::null(), |u| { u.as_ptr() }))
        .unwrap_or_else(|e| {
            eprintln!("Error in LV2 URID unmapper: {}", e);
            ::std::ptr::null()
        })
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct URIDUnmap {
    inner: ::lv2_sys::LV2_URID_Unmap
}

unsafe impl Feature for URIDUnmap {
    const URI: &'static [u8] = ::lv2_sys::LV2_URID_UNMAP_URI;
}

impl URIDUnmap {
    #[inline]
    pub fn new<T: URIDMapper>(mapper: &T) -> URIDUnmap { // FIXME: mapper needs an additional lifetime check here
        URIDUnmap {
            inner: ::lv2_sys::LV2_URID_Unmap {
                handle: mapper as *const T as *const c_void as *mut c_void,
                unmap: Some(urid_unmap::<T>)
            }
        }
    }

    #[inline]
    pub fn unmap(&self, urid: URID) -> Option<&Uri> {
        let inner_unmap = self.inner.unmap.expect("Unmap function is null");
        unsafe {
            let ptr = inner_unmap(self.inner.handle, urid.get());
            if ptr.is_null() { return None }
            Some(Uri::from_cstr_unchecked(CStr::from_ptr(ptr)))
        }
    }
}