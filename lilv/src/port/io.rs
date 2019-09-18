use InnerPort;
use lv2::core::PortType;
use std::marker::PhantomData;
use port::port::TypedPort;
use port::port::Port;
use node::{Node, String, Uri, NodeList};
use Plugin;
use std::fmt;
use port::port::PortIndex;
use std::ffi::CStr;

#[repr(transparent)]
pub struct UnknownInputPort<'p> {
    inner: InnerPort<'p>
}

impl<'p> fmt::Debug for UnknownInputPort<'p> {
    fn fmt(&self, f: & mut fmt::Formatter) -> fmt::Result {
        self.inner.fmt("InputPort<?>", f)
    }
}
#[repr(transparent)]
pub struct UnknownOutputPort<'p> {
    inner: InnerPort<'p>
}

impl<'p> fmt::Debug for UnknownOutputPort<'p> {
    fn fmt(&self, f: & mut fmt::Formatter) -> fmt::Result {
        self.inner.fmt("OutputPort<?>", f)
    }
}

macro_rules! impl_port {
    ($($innerpath:tt)+) => {
        #[inline]
        fn classes(&self) -> &NodeList<Uri> {
            self.$($innerpath)*.classes()
        }

        #[inline]
        fn plugin(&self) -> &'p Plugin<'p> {
            self.$($innerpath)*.plugin()
        }

        #[inline]
        fn name(&self) -> Node<String> {
            self.$($innerpath)*.name()
        }

        #[inline]
        fn symbol(&self) -> &String {
            self.$($innerpath)*.symbol()
        }

        #[inline]
        fn index(&self) -> PortIndex {
            self.$($innerpath)*.index()
        }
    }
}

fn typed_from_untyped<'p, P: Port<'p>, PT: PortType, T: TypedPort<'p, PortType=PT, UntypedPortType=P>>(port: P) -> Option<T> {
    if !port.has_class(PT::uri()) { return None; }
    Some(unsafe { T::from_untyped(port) })
}

fn typed_from_untyped_ref<'p, P: Port<'p>, PT: PortType, T: TypedPort<'p, PortType=PT, UntypedPortType=P>>(port: &P) -> Option<&T> {
    if !port.has_class(PT::uri()) { return None; }
    Some(unsafe { T::from_untyped_ref(port) })
}

impl<'p> Port<'p> for UnknownInputPort<'p> { impl_port!(inner); }
impl<'p> Port<'p> for UnknownOutputPort<'p> { impl_port!(inner); }

impl<'p> UnknownInputPort<'p> {
    pub(crate) fn from_inner(inner: InnerPort<'p>) -> Option<Self> {
        if !inner.has_class(unsafe { CStr::from_bytes_with_nul_unchecked(::lilv_sys::LV2_CORE__InputPort as &[_]) }) { return None; }
        Some(Self { inner })
    }

    #[inline]
    pub fn as_typed<T: PortType>(&self) -> Option<&InputPort<'p, T>> {
        typed_from_untyped_ref(self)
    }

    #[inline]
    pub fn into_typed<T: PortType>(self) -> Option<InputPort<'p, T>> {
        typed_from_untyped(self)
    }
}

impl<'p> UnknownOutputPort<'p> {
    pub(crate) fn from_inner(inner: InnerPort<'p>) -> Option<Self> {
        if !inner.has_class(unsafe { CStr::from_bytes_with_nul_unchecked(::lilv_sys::LV2_CORE__OutputPort as &[_]) }) { return None; }
        Some(Self { inner })
    }

    #[inline]
    pub fn as_typed<T: PortType>(&self) -> Option<&OutputPort<'p, T>> {
        typed_from_untyped_ref(self)
    }

    #[inline]
    pub fn into_typed<T: PortType>(self) -> Option<OutputPort<'p, T>> {
        typed_from_untyped(self)
    }
}
#[repr(transparent)]
pub struct InputPort<'p, T: PortType> {
    inner: UnknownInputPort<'p>,
    _phantom: PhantomData<T>
}

impl<'p, T: PortType> fmt::Debug for InputPort<'p, T> {
    fn fmt(&self, f: & mut fmt::Formatter) -> fmt::Result {
        self.inner.inner.fmt(&format!("InputPort<{}>", T::NAME), f)
    }
}
#[repr(transparent)]
pub struct OutputPort<'p, T: PortType> {
    inner: UnknownOutputPort<'p>,
    _phantom: PhantomData<T>
}

impl<'p, T: PortType> fmt::Debug for OutputPort<'p, T> {
    fn fmt(&self, f: & mut fmt::Formatter) -> fmt::Result {
        self.inner.inner.fmt(&format!("OutputPort<{}>", T::NAME), f)
    }
}

impl<'p, T: PortType> Port<'p> for InputPort<'p, T> { impl_port!(inner.inner); }
impl<'p, T: PortType> Port<'p> for OutputPort<'p, T> { impl_port!(inner.inner); }

impl<'p, T: PortType> TypedPort<'p> for InputPort<'p, T> {
    type PortType = T;
    type UntypedPortType = UnknownInputPort<'p>;

    unsafe fn from_untyped_ref<'a>(untyped: &'a <Self as TypedPort<'p>>::UntypedPortType) -> &'a Self {
        &*(untyped as *const _ as *const _)
    }

    unsafe fn from_untyped(inner: <Self as TypedPort<'p>>::UntypedPortType) -> Self {
        Self { inner, _phantom: PhantomData }
    }
}

impl<'p, T: PortType> TypedPort<'p> for OutputPort<'p, T> {
    type PortType = T;
    type UntypedPortType = UnknownOutputPort<'p>;

    unsafe fn from_untyped_ref<'a>(untyped: &'a <Self as TypedPort<'p>>::UntypedPortType) -> &'a Self {
        &*(untyped as *const _ as *const _)
    }

    unsafe fn from_untyped(inner: <Self as TypedPort<'p>>::UntypedPortType) -> Self {
        Self { inner, _phantom: PhantomData }
    }
}
