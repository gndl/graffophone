use node::inner_node::node_from_ptr;
use node::UnknownNode;
use std::fmt;
use std::marker::PhantomData;
use std::ops::Deref;

pub unsafe trait NodeType<'w>: Sized + fmt::Debug {
    const NAME: &'static str;

    fn node_matches_type(node: &UnknownNode) -> bool;

    unsafe fn ref_from_node_unchecked<'a>(node: &'a UnknownNode<'w>) -> &'a Self {
        ::std::mem::transmute(node)
    }

    fn ref_from_node<'a>(node: &'a UnknownNode<'w>) -> Result<&'a Self, &'a UnknownNode<'w>> {
        if !Self::node_matches_type(&node) { return Err(node) }
        Ok(unsafe { Self::ref_from_node_unchecked(node) })
    }
}

pub struct Node<'w, T: NodeType<'w>> {
    ptr: &'w mut ::lilv_sys::LilvNode,
    _marker: PhantomData<T>
}

impl<'w, T: NodeType<'w>> Node<'w, T> {
    #[inline(always)]
    pub(crate) unsafe fn new(ptr: *mut ::lilv_sys::LilvNode) -> Self {
        Node { ptr: &mut *ptr, _marker: PhantomData }
    }

    #[inline]
    pub(crate) unsafe fn new_opt(ptr: *mut ::lilv_sys::LilvNode) -> Option<Self> {
        ptr.as_mut().map(|ptr| Node { ptr, _marker: PhantomData })
    }
}

impl<'w, T: NodeType<'w> + fmt::Debug + 'w> fmt::Debug for Node<'w, T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self.deref(), f)
    }
}

impl<'w, T: NodeType<'w> + fmt::Display + 'w> fmt::Display for Node<'w, T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self.deref(), f)
    }
}

impl<'w, T: NodeType<'w> + 'w> Deref for Node<'w, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        unsafe { node_from_ptr(self.ptr) }
    }
}

impl<'w, T: NodeType<'w> + 'w> AsRef<T> for Node<'w, T> {
    fn as_ref(&self) -> &T {
        &*self
    }
}

impl<'w, T: NodeType<'w>> Drop for Node<'w, T> {
    #[inline]
    fn drop(&mut self) {
        unsafe { ::lilv_sys::lilv_node_free(self.ptr) }
    }
}
