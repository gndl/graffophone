use std::ffi::c_void;
use std::cell::Cell;
use std::fmt;
use std::ops::Deref;

pub unsafe trait BufferType: ::lv2::core::PortType {
    type BufferImpl: PortBuffer + 'static;
}

pub unsafe trait PortBuffer {
    fn get_ptr(&self) -> *mut c_void;
    fn get_size(&self) -> usize;
}

#[derive(Clone, Default)]
pub struct CellBuffer<T: Copy = f32> {
    inner: Cell<T>
}

impl<T: Copy> CellBuffer<T> {
    #[inline]
    pub fn new(value: T) -> Self {
        Self { inner: Cell::new(value) }
    }

    #[inline]
    pub fn get(&self) -> T { self.inner.get() }

    #[inline]
    pub fn set(&self, value: T) {
        self.inner.replace(value);
    }
}

unsafe impl<T: Copy> PortBuffer for CellBuffer<T> {
    #[inline]
    fn get_ptr(&self) -> *mut c_void {
        self.inner.as_ptr() as _
    }

    #[inline]
    fn get_size(&self) -> usize {
        1
    }
}

unsafe impl BufferType for ::lv2::core::ports::Control {
    type BufferImpl = CellBuffer<f32>;
}

impl<T: Copy + fmt::Debug> fmt::Debug for CellBuffer<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        fmt::Debug::fmt(&self.get(), f)
    }
}

#[derive(Clone)]
pub struct VecBuffer<T: Copy = f32> {
    inner: Vec<Cell<T>>
}

impl<T: Copy> VecBuffer<T> {
    #[inline]
    pub fn new(initial_value: T, capacity: usize) -> Self {
        Self { inner: vec![Cell::new(initial_value); capacity] }
    }

    #[inline]
    pub fn get(&self) -> &[Cell<T>] {
        &*(self.inner)
    }

    #[inline]
    pub fn set_all<I: IntoIterator<Item=T>>(&self, iter: I) {
        for (src, dst) in iter.into_iter().zip(&self.inner) {
            dst.set(src)
        }
    }
}

unsafe impl<T: Copy> PortBuffer for VecBuffer<T> {
    #[inline]
    fn get_ptr(&self) -> *mut c_void {
        self.inner.as_ptr() as _
    }

    #[inline]
    fn get_size(&self) -> usize {
        self.inner.len()
    }

    // type BufferType = SampleDependentBufferType<T>;
}

impl<T: Copy + fmt::Debug> fmt::Debug for VecBuffer<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        fmt::Debug::fmt(&self.get(), f)
    }
}

unsafe impl BufferType for ::lv2::core::ports::Audio {
    type BufferImpl = VecBuffer<f32>;
}

unsafe impl BufferType for ::lv2::core::ports::CV {
    type BufferImpl = VecBuffer<f32>;
}


impl<T: Copy> Deref for VecBuffer<T> {
    type Target = [Cell<T>];

    fn deref(&self) -> &[Cell<T>] {
        &*self.inner
    }
}