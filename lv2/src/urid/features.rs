use core::uri::AsUriRef;
use core::uri::Uri;
use core::Feature;
use std::collections::HashMap;
use std::error::Error;
use std::ffi::{CStr, CString};
use std::fmt;
use std::num::NonZeroU32;
use std::os::raw::{c_char, c_void};
use urid::cache::URIDCache;
use urid::{URIDMapper, URID};

#[derive(Debug)]
struct MapperPanickedError;

impl fmt::Display for MapperPanickedError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt("Mapper panicked", f)
    }
}

impl Error for MapperPanickedError {}
use std::cell::RefCell;

thread_local!(static MAP: RefCell< HashMap<CString, u32>> = RefCell::new(HashMap::new()));

unsafe extern "C" fn urid_map(
    _handle: ::lv2_sys::LV2_URID_Map_Handle,
    c_uri: *const c_char,
) -> ::lv2_sys::LV2_URID {
    let uri = CString::new(CStr::from_ptr(c_uri).to_str().unwrap()).unwrap();
    let mut ret = 0;

    MAP.with(|map_cell| {
        let mut map = map_cell.borrow_mut();

        match map.get(&uri) {
            Some(urid) => ret = *urid,
            None => {
                ret = map.len() as u32 + 1;
                map.insert(uri, ret);
            }
        }
    });

    println!(
        "URID map {:?} -> {}",
        CString::new(CStr::from_ptr(c_uri).to_str().unwrap()).unwrap(),
        ret
    );

    ret
}

unsafe extern "C" fn urid_map_with_mapper<T: URIDMapper>(
    handle: ::lv2_sys::LV2_URID_Map_Handle,
    uri: *const c_char,
) -> ::lv2_sys::LV2_URID {
    #[inline(always)]
    unsafe fn inner_urid_map<T: URIDMapper>(
        handle: ::lv2_sys::LV2_URID_Map_Handle,
        uri: *const c_char,
    ) -> Result<URID, Box<dyn Error>> {
        let value = ::std::panic::catch_unwind(|| {
            (*(handle as *const T)).map(Uri::from_cstr(CStr::from_ptr(uri))?)
        })
        .map_err(|_| MapperPanickedError)?;
        value
    }

    inner_urid_map::<T>(handle, uri)
        .map(NonZeroU32::get)
        .unwrap_or_else(|e| {
            eprintln!("Error in LV2 URID mapper: {}", e);
            0
        })
}

unsafe extern "C" fn urid_map_with_hashmap(
    handle: ::lv2_sys::LV2_URID_Map_Handle,
    uri: *const c_char,
) -> ::lv2_sys::LV2_URID {
    match (*(handle as *const HashMap<&CStr, u32>)).get(CStr::from_ptr(uri)) {
        Some(urid) => *urid,
        None => {
            (*(handle as *mut HashMap<&CStr, u32>)).insert(
                CStr::from_ptr(uri),
                (*(handle as *const HashMap<&CStr, u32>)).len() as u32 + 1,
            );
            (*(handle as *const HashMap<&CStr, u32>)).len() as u32
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct URIDMap {
    inner: ::lv2_sys::LV2_URID_Map,
}

unsafe impl Feature for URIDMap {
    const URI: &'static [u8] = ::lv2_sys::LV2_URID_MAP_URI;
}

impl<'a> URIDMap {
    #[inline]
    pub fn new() -> URIDMap {
        URIDMap {
            inner: ::lv2_sys::LV2_URID_Map {
                handle: ::std::ptr::null() as *const HashMap<CString, u32> as *const c_void
                    as *mut c_void,
                map: Some(urid_map),
            },
        }
    }

    #[inline]
    pub fn new_with_mapper<T: URIDMapper>(mapper: &'a T) -> URIDMap {
        URIDMap {
            inner: ::lv2_sys::LV2_URID_Map {
                handle: mapper as *const T as *const c_void as *mut c_void,
                map: Some(urid_map_with_mapper::<T>),
            },
        }
    }

    pub fn new_with_hashmap(mapper: &'a HashMap<CString, u32>) -> URIDMap {
        URIDMap {
            inner: ::lv2_sys::LV2_URID_Map {
                handle: mapper as *const HashMap<CString, u32> as *const c_void as *mut c_void,
                map: Some(urid_map_with_hashmap),
            },
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

unsafe extern "C" fn urid_unmap(
    _handle: ::lv2_sys::LV2_URID_Unmap_Handle,
    urid: ::lv2_sys::LV2_URID,
) -> *const c_char {
    let mut k = ::std::ptr::null();
    MAP.with(|map_cell| {
        let map = map_cell.borrow_mut();

        for (key, value) in map.iter() {
            if *value == urid {
                k = key.as_ptr();
            }
        }
    });
    println!("URID unmap {} -> {:?}", urid, k);
    k
}

unsafe extern "C" fn urid_unmap_with_mapper<T: URIDMapper>(
    handle: ::lv2_sys::LV2_URID_Unmap_Handle,
    urid: ::lv2_sys::LV2_URID,
) -> *const c_char {
    #[inline(always)]
    unsafe fn inner_urid_unmap<'a, T: URIDMapper + 'a>(
        handle: ::lv2_sys::LV2_URID_Map_Handle,
        urid: ::lv2_sys::LV2_URID,
    ) -> Result<Option<&'a Uri>, Box<dyn Error>> {
        ::std::panic::catch_unwind(|| {
            (&*(handle as *const T)).unmap(NonZeroU32::new(urid).unwrap()) //TODO: handle error
        })
        .map_err(|_| MapperPanickedError.into())
    }

    inner_urid_unmap::<T>(handle, urid)
        .map(|uri| uri.map_or(::std::ptr::null(), |u| u.as_ptr()))
        .unwrap_or_else(|e| {
            eprintln!("Error in LV2 URID unmapper: {}", e);
            ::std::ptr::null()
        })
}

unsafe extern "C" fn urid_unmap_with_hashmap(
    handle: ::lv2_sys::LV2_URID_Unmap_Handle,
    urid: ::lv2_sys::LV2_URID,
) -> *const c_char {
    let mut k = ::std::ptr::null();
    for (key, value) in (*(handle as *const HashMap<CString, u32>)).iter() {
        if *value == urid {
            k = key.as_ptr();
        }
    }
    k
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct URIDUnmap {
    inner: ::lv2_sys::LV2_URID_Unmap,
}

unsafe impl Feature for URIDUnmap {
    const URI: &'static [u8] = ::lv2_sys::LV2_URID_UNMAP_URI;
}

impl<'a> URIDUnmap {
    #[inline]
    pub fn new() -> URIDUnmap {
        URIDUnmap {
            inner: ::lv2_sys::LV2_URID_Unmap {
                handle: ::std::ptr::null() as *const URIDMap as *const c_void as *mut c_void,
                unmap: Some(urid_unmap),
            },
        }
    }

    pub fn new_with_mapper<T: URIDMapper>(mapper: &'a T) -> URIDUnmap {
        // FIXME: mapper needs an additional lifetime check here
        URIDUnmap {
            inner: ::lv2_sys::LV2_URID_Unmap {
                handle: mapper as *const T as *const c_void as *mut c_void,
                unmap: Some(urid_unmap_with_mapper::<T>),
            },
        }
    }

    pub fn new_with_hashmap(mapper: &'a HashMap<CString, u32>) -> URIDUnmap {
        URIDUnmap {
            inner: ::lv2_sys::LV2_URID_Unmap {
                handle: mapper as *const HashMap<CString, u32> as *const c_void as *mut c_void,
                unmap: Some(urid_unmap_with_hashmap),
            },
        }
    }

    #[inline]
    pub fn unmap(&self, urid: URID) -> Option<&Uri> {
        let inner_unmap = self.inner.unmap.expect("Unmap function is null");
        unsafe {
            let ptr = inner_unmap(self.inner.handle, urid.get());
            if ptr.is_null() {
                return None;
            }
            Some(Uri::from_cstr_unchecked(CStr::from_ptr(ptr)))
        }
    }
}
