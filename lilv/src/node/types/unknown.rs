use node::inner_node::InnerNode;
use std::fmt;
use node::node::NodeType;

fn fmt_full<'w, T: NodeType<'w>>(node: &T, f: &mut fmt::Formatter) -> fmt::Result {
    f.debug_tuple(T::NAME)
        .field(node)
        .finish()
}

pub struct UnknownNode<'w> {
    _node: InnerNode<'w>
}

impl<'w> UnknownNode<'w> {
    #[inline]
    pub fn is<T: NodeType<'w>>(&self) -> bool {
        T::node_matches_type(self)
    }

    #[inline]
    pub fn as_ref<T: NodeType<'w>>(&self) -> Result<&T, &Self> {
        T::ref_from_node(self)
    }
}

impl<'w> fmt::Debug for UnknownNode<'w> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Ok(n) = self.as_ref::<::node::Float>() { return fmt_full(n, f) }
        if let Ok(n) = self.as_ref::<::node::Uri>() { return fmt_full(n, f) }
        if let Ok(n) = self.as_ref::<::node::String>() { return fmt_full(n, f) }
        return f.debug_tuple("<Unknown Node>").finish()
    }
}

unsafe impl<'w> NodeType<'w> for UnknownNode<'w> {
    const NAME: &'static str = "Node";

    #[inline]
    fn node_matches_type(_node: &UnknownNode) -> bool { true }
}
