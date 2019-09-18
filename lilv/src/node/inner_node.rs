use std::marker::PhantomData;
use world::World;
use node::node::NodeType;

#[inline]
pub unsafe fn node_from_ptr<'w, T>(ptr: *const ::lilv_sys::LilvNode) -> &'w T where T : NodeType<'w> {
    &*(ptr as *const T)
}

#[inline]
pub fn node_get_ptr<'w, T: Sized>(node: &T) -> *mut ::lilv_sys::LilvNode where T : NodeType<'w> {
    node as *const _ as *mut _
}

pub struct InnerNode<'w> {
    _world_phantom: PhantomData<&'w World>
}
