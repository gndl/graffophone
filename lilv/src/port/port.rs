use lv2::core::PortDirection;
use lv2::core::PortType;
use node::{Node, NodeList, String, Uri};
use std::ffi::CStr;
use std::fmt;
use std::marker::PhantomData;
use Plugin;

pub type PortIndex = u32;

#[derive(Copy)]
pub struct PortHandle<T: PortType> {
    index: PortIndex,
    _marker: PhantomData<T>,
}

impl<T: PortType> Clone for PortHandle<T> {
    #[inline]
    fn clone(&self) -> Self {
        PortHandle {
            index: self.index,
            _marker: PhantomData,
        }
    }
}

impl<T: PortType> PortHandle<T> {
    pub fn index(&self) -> u32 {
        self.index
    }
}

impl<T: PortType> fmt::Debug for PortHandle<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple(T::NAME).field(&self.index).finish()
    }
}

pub trait Port<'p> {
    fn classes(&self) -> &NodeList<Uri>;
    fn plugin(&self) -> &'p Plugin<'p>;
    fn name(&self) -> Node<String>;
    fn symbol(&self) -> &String;
    fn index(&self) -> PortIndex;

    #[inline]
    fn has_class<'u, U: ?Sized>(&'u self, class_uri: &U) -> bool
    where
        Uri<'u>: PartialEq<U>,
    {
        self.classes()
            .into_iter()
            .filter(|u| *u == class_uri)
            .next()
            .is_some()
    }

    #[inline]
    fn is<PT: PortType>(&self) -> bool {
        self.has_class(PT::uri())
    }

    #[inline]
    fn optional(&self) -> bool {
        self.has_class(unsafe {
            CStr::from_bytes_with_nul_unchecked(::lilv_sys::LV2_CORE__connectionOptional as &[_])
        })
    }

    #[inline]
    fn try_handle<PT: PortType>(&self) -> Option<PortHandle<PT>> {
        if self.is::<PT>() {
            Some(PortHandle {
                index: self.index(),
                _marker: PhantomData,
            })
        } else {
            None
        }
    }

    fn direction(&self) -> PortDirection {
        if self.has_class(unsafe {
            CStr::from_bytes_with_nul_unchecked(::lilv_sys::LV2_CORE__InputPort as &[_])
        }) {
            PortDirection::Input
        } else if self.has_class(unsafe {
            CStr::from_bytes_with_nul_unchecked(::lilv_sys::LV2_CORE__OutputPort as &[_])
        }) {
            PortDirection::Output
        } else {
            unreachable!()
        }
    }
}

pub trait TypedPort<'p>: Port<'p> {
    type PortType: PortType;
    type UntypedPortType: Port<'p>;

    unsafe fn from_untyped_ref(untyped: &Self::UntypedPortType) -> &Self;
    unsafe fn from_untyped(untyped: Self::UntypedPortType) -> Self;

    #[inline]
    fn handle(&self) -> PortHandle<Self::PortType> {
        PortHandle {
            index: self.index(),
            _marker: PhantomData,
        }
    }
}
