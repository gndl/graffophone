use lilv::port::buffer::CellBuffer;
use lilv::port::buffer::VecBuffer;
use std::rc::Rc;

pub type Audio = Rc<VecBuffer<f32>>;
pub type Control = Rc<CellBuffer<f32>>;
pub type Cv = Rc<VecBuffer<f32>>;

pub fn audio(value: Option<f32>, len: usize) -> Audio {
    Rc::new(VecBuffer::new(value.unwrap_or(0.), len))
}

pub fn control(value: Option<f32>) -> Control {
    Rc::new(CellBuffer::new(value.unwrap_or(0.)))
}

pub fn cv(value: Option<f32>, len: usize) -> Cv {
    Rc::new(VecBuffer::new(value.unwrap_or(0.), len))
}
