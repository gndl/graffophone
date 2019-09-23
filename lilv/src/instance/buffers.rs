use port::{PortIndex};
use std::borrow::Borrow;
use port::buffer::BufferType;
use std::any::Any;
use ::port::Port;
use ::plugin::Plugin;

struct BufferContainer<T: BufferType> {
    buffer: Box<dyn Borrow<T::BufferImpl>>
}

impl<T: BufferType> BufferContainer<T> {
    #[inline]
    fn new<R: Borrow<T::BufferImpl> + 'static>(buffer: R) -> Self {
        Self { buffer: Box::new(buffer) }
    }

    #[inline]
    fn downcast_ref(ptr: &Box<dyn Any>) -> Option<&T::BufferImpl> {
        ptr.downcast_ref::<Self>().map(|b| b.buffer.as_ref().borrow())
    }
}

pub struct BufferHandle {
    buffer: Option<Box<dyn Any>>,
    // index: PortIndex,
    // optional: bool,
    // direction: PortDirection,
}

impl BufferHandle {
    pub fn new<'p, P: Port<'p>>(_port: &P) -> Self {
        Self {
            buffer: None,
            // index: port.index(),
            // optional: port.optional(),
            // direction: port.direction()
        }
    }

    /*#[inline]
    pub fn index(&self) -> PortIndex {
        self.index
    }

    #[inline]
    pub fn direction(&self) -> PortDirection { self.direction }*/

    #[inline]
    pub unsafe fn set_buffer<
        T: BufferType,
        R: Borrow<T::BufferImpl> + 'static
    >(&mut self, buffer: R) -> &T::BufferImpl {
        self.buffer = Some(Box::new(BufferContainer::<T>::new(buffer)));
        self.inner::<T>().unwrap()
    }

    #[inline]
    pub fn inner<T: BufferType>(&self) -> Option<&T::BufferImpl> {
        self.buffer.as_ref().and_then(BufferContainer::<T>::downcast_ref)
    }
}

pub struct Buffers {
    buffers: Vec<BufferHandle>
}

impl Buffers {
    pub fn new(plugin: &Plugin) -> Self {
        Self {
            buffers: plugin.ports().map(|p| BufferHandle::new(&p)).collect()
        }
    }

    #[inline]
    pub fn get<T: BufferType>(&self, index: PortIndex) -> Option<&T::BufferImpl> {
        self.buffers.get(index as usize)?.inner::<T>()
    }

    #[inline]
    pub unsafe fn set<T: BufferType, R: Borrow<T::BufferImpl> + 'static>(
        &mut self,
        index: PortIndex,
        buffer: R
    ) -> &T::BufferImpl {
        self.buffers.get_mut(index as usize).unwrap().set_buffer::<T, R>(buffer)
    }

    #[inline]
    pub fn all<T: BufferType>(&self) -> impl Iterator<Item=&T::BufferImpl> {
        self.buffers.iter().filter_map(BufferHandle::inner::<T>)
    }
}
