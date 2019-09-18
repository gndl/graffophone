use node::inner_node::InnerNode;
use std::fmt;
use node::node::NodeType;
use node::inner_node::node_get_ptr;
use node::types::unknown::UnknownNode;

pub struct Float<'w> {
    _node: InnerNode<'w>
}

impl<'w> Float<'w> {
    pub fn value(&self) -> f32 {
        unsafe { ::lilv_sys::lilv_node_as_float(node_get_ptr(self)) }
    }
}

impl<'w> fmt::Display for Float<'w> {
    fn fmt(&self, f: & mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.value(), f)
    }
}

impl<'w> fmt::Debug for Float<'w> {
    fn fmt(&self, f: & mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self.value(), f)
    }
}

unsafe impl<'w> NodeType<'w> for Float<'w> {
    const NAME: &'static str = "Float";

    fn node_matches_type(node: &UnknownNode) -> bool {
        unsafe { ::lilv_sys::lilv_node_is_float(node_get_ptr(node)) }
    }
}
