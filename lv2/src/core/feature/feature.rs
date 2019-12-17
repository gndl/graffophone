use core::feature::buffer::FeatureBuffer;
use core::feature::descriptor::FeatureDescriptor;
use core::uri::UriBound;

/// Represents extension data for a given feature.
/// # Unsafety
/// Since extension data is passed to plugin as a raw pointer,
/// structs implementing this trait must be `#[repr(C)]`.
pub unsafe trait Feature: Sized + Copy {
    const URI: &'static [u8];

    #[inline]
    fn descriptor(&self) -> FeatureDescriptor {
        FeatureDescriptor::from_feature(self)
    }
}

unsafe impl<F: Feature> UriBound for F {
    const URI: &'static [u8] = <F as Feature>::URI;
}

#[repr(transparent)]
pub struct RawFeatureDescriptor {
    pub(crate) inner: ::lv2_sys::LV2_Feature,
}

pub trait FeatureSet /*<'a>*/ {
    fn to_list(&self) -> FeatureBuffer /*<'a>*/;
}
