use core::feature::Feature;

#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct HardRTCapable;

unsafe impl Feature for HardRTCapable {
    const URI: &'static [u8] = ::lv2_sys::LV2_CORE__hardRTCapable;
}

#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct InPlaceBroken;

unsafe impl Feature for InPlaceBroken {
    const URI: &'static [u8] = ::lv2_sys::LV2_CORE__inPlaceBroken;
}

#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct IsLive;

unsafe impl Feature for IsLive {
    const URI: &'static [u8] = ::lv2_sys::LV2_CORE__isLive;
}
