use std::ptr::NonNull;
use std::marker::PhantomData;

pub struct InputSampledData<T: Copy> {
    pointer: NonNull<T>,
    sample_count: u32,
    _phantom: PhantomData<T>
}

impl<T: Copy> InputSampledData<T> {
    #[inline]
    pub unsafe fn new(pointer: NonNull<()>, sample_count: u32) -> Self {
        Self { pointer: pointer.cast(), sample_count, _phantom: PhantomData }
    }

    #[inline]
    pub fn data(&self) -> &[T] {
        unsafe { ::std::slice::from_raw_parts(self.pointer.as_ptr(), self.sample_count as usize) }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.sample_count as usize
    }
}

pub struct OutputSampledData<T: Copy> {
    pointer: NonNull<T>,
    sample_count: u32,
    _phantom: PhantomData<T>
}

impl<T: Copy> OutputSampledData<T> {
    #[inline]
    pub unsafe fn new(pointer: NonNull<()>, sample_count: u32) -> Self {
        Self { pointer: pointer.cast(), sample_count, _phantom: PhantomData }
    }

    #[inline]
    pub fn data(&self) -> &mut [T] { // TODO: do not expose directly a slice that can be read from
        unsafe { ::std::slice::from_raw_parts_mut(self.pointer.as_ptr(), self.sample_count as usize) }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.sample_count as usize
    }
}
