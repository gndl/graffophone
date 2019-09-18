use std::os::raw::{c_void};
use std::fmt;
use std::marker::PhantomData;
use core::uri::{Uri, UriBound};
use core::feature::feature::Feature;
use core::feature::feature::RawFeatureDescriptor;
use std::ffi::CStr;

#[derive(Copy, Clone)]
pub struct FeatureDescriptor<'a> {
    pub(crate) inner: ::lv2_sys::LV2_Feature,
    uri_len: usize,
    _lifetime: PhantomData<&'a u8>
}

impl<'a> FeatureDescriptor<'a> {
    #[inline]
    pub fn from_feature<T: Feature + UriBound>(feature: &'a T) -> FeatureDescriptor<'a> {
        let uri = T::uri();
        let data = if ::std::mem::size_of::<T>() == 0 {
            ::std::ptr::null_mut()
        } else {
            feature as *const T as *const c_void as *mut c_void
        };
        FeatureDescriptor {
            inner: ::lv2_sys::LV2_Feature { URI: uri.as_ptr(), data },
            uri_len: uri.to_bytes_with_nul().len(),
            _lifetime: PhantomData,
        }
    }

    #[inline]
    pub unsafe fn from_raw(raw: *const RawFeatureDescriptor) -> FeatureDescriptor<'a> {
        let inner = (*raw).inner;
        let uri_len = CStr::from_ptr(inner.URI).to_bytes_with_nul().len();

        FeatureDescriptor {
            inner, uri_len, _lifetime: PhantomData
        }
    }

    #[inline]
    pub fn uri(&self) -> &Uri {
        unsafe {
            let slice = ::std::slice::from_raw_parts(self.inner.URI as *const u8, self.uri_len);
            Uri::from_bytes_unchecked(slice)
        }
    }

    #[inline]
    pub fn matches_uri(&self, uri: &Uri) -> bool {
        self.uri() == uri
    }

    #[inline]
    pub fn as_feature<T: Feature>(&self) -> Option<&'a T> {
        let uri = unsafe { Uri::from_bytes_unchecked(T::URI) };
        if self.matches_uri(uri) {
            unsafe { (self.inner.data as *const T).as_ref() }
        } else {
            None
        }
    }

    #[inline]
    pub fn into_feature_ref<T: Feature>(self) -> Option<&'a T> {
        let uri = unsafe { Uri::from_bytes_unchecked(T::URI) };
        if self.matches_uri(uri) {
            unsafe { (self.inner.data as *const T).as_ref() }
        } else {
            None
        }
    }

    #[inline]
    pub fn as_raw(&self) -> *const RawFeatureDescriptor {
        self as *const _ as *const _
    }
}

impl<'a> fmt::Debug for FeatureDescriptor<'a> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("Feature")?;
        fmt::Debug::fmt(self.uri(), f)
    }
}

// TODO: use compiletest-rs

// ``compile_fail
//   use ::lv2::core::{Feature, FeatureDescriptor};
//   struct MyFeature<'a>(pub &'a Vec<u8>);
//
//    unsafe impl<'a> Feature for MyFeature<'a> {
//        const URI: &'static [u8] = b"http://proko.eu/boo\0";
//    }
//
//    let desc: FeatureDescriptor = {
//        let buf = vec![42, 69];
//        let f = MyFeature(&buf);
//
//        (&f).into()
//    };
//    let fo: &MyFeature = desc.as_feature().unwrap();
//    assert_eq!(fo.0, &vec![42u8, 69])
// ``

impl<'a, T: Feature> From<&'a T> for FeatureDescriptor<'a> {
    #[inline]
    fn from(feature: &'a T) -> Self {
        FeatureDescriptor::from_feature(feature)
    }
}

impl<'a, T: Feature> From<&'a mut T> for FeatureDescriptor<'a> {
    #[inline]
    fn from(feature: &'a mut T) -> Self {
        FeatureDescriptor::from_feature(feature)
    }
}
