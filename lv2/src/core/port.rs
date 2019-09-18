use core::uri::Uri;
use std::ptr::NonNull;

pub trait PortType: 'static + Sized {
    const NAME: &'static str;
    const URI: &'static [u8];

    type InputPortType: Sized;
    type OutputPortType: Sized;

    unsafe fn input_from_raw(pointer: NonNull<()>, sample_count: u32) -> Self::InputPortType;
    unsafe fn output_from_raw(pointer: NonNull<()>, sample_count: u32) -> Self::OutputPortType;

    #[inline]
    fn uri() -> &'static Uri {
        unsafe { Uri::from_bytes_unchecked(Self::URI) }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum PortDirection {
    Input,
    Output
}