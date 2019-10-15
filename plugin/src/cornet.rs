use lilv::port::buffer::VecBuffer;
use std::rc::Rc;

pub type Cornet = Rc<VecBuffer<f32>>;


pub fn new(len: usize) -> Cornet {
Rc::new(VecBuffer::new(0f32, len))
}
