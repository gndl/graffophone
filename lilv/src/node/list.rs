use node::node::NodeType;
use std::marker::PhantomData;
use node::inner_node::node_from_ptr;
use std::ops::Deref;

pub struct NodeList<'w, T: NodeType<'w>> {
    _marker: PhantomData<&'w T>
}

impl<'w, T: NodeType<'w>> NodeList<'w, T> {
    #[inline(always)]
    pub(crate) unsafe fn new<'a>(ptr: *const ::lilv_sys::LilvNodes) -> &'a mut NodeList<'w, T> {
        &mut *(ptr as *mut _)
    }

    #[inline(always)]
    fn ptr(&self) -> *const ::lilv_sys::LilvNodes {
        self as *const _ as *const _
    }

    pub fn size(&self) -> u32 {
        unsafe { ::lilv_sys::lilv_nodes_size(self.ptr()) }
    }

    pub fn merge(&self, other: &NodeList<'w, T>) -> OwnedNodeList<'w, T> {
        unsafe { OwnedNodeList::new(::lilv_sys::lilv_nodes_merge(self.ptr(), other.ptr())) }
    }
}

impl<'w, T: NodeType<'w> + 'w> IntoIterator for &'w NodeList<'w, T> {
    type Item = &'w T;
    type IntoIter = NodeIter<'w, T>;

    fn into_iter(self) -> <Self as IntoIterator>::IntoIter {
        NodeIter::new(self, unsafe { ::lilv_sys::lilv_nodes_begin(self.ptr()) })
    }
}

pub struct NodeIter<'w, T: NodeType<'w> + 'w> {
    iter: *mut ::lilv_sys::LilvIter,
    collection: &'w NodeList<'w, T>,
    _marker: PhantomData<T>
}

impl<'w, T: NodeType<'w>> NodeIter<'w, T> {
    #[inline]
    fn new(collection: &'w NodeList<'w, T>, iter: *mut ::lilv_sys::LilvIter) -> Self {
        NodeIter { collection, iter, _marker: PhantomData }
    }
}

impl<'w, T: NodeType<'w> + 'w> Iterator for NodeIter<'w, T> {
    type Item = &'w T;

    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        if unsafe { ::lilv_sys::lilv_nodes_is_end(self.collection.ptr(),  self.iter) } {
            return None
        }

        let value = unsafe { node_from_ptr(::lilv_sys::lilv_nodes_get(self.collection.ptr(), self.iter)) };
        self.iter = unsafe { ::lilv_sys::lilv_nodes_next(self.collection.ptr(), self.iter) };
        Some(value)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.collection.size() as usize;
        (size, Some(size))
    }
}

pub struct OwnedNodeList<'w, T: NodeType<'w>> {
    ptr: &'w mut NodeList<'w, T>,
}

impl<'w, T: NodeType<'w>> OwnedNodeList<'w, T> {
    #[inline(always)]
    pub(crate) unsafe fn new(ptr: *mut ::lilv_sys::LilvNodes) -> Self {
        OwnedNodeList { ptr: NodeList::new(ptr) }
    }
}

impl<'w, T: NodeType<'w>> Drop for OwnedNodeList<'w, T> {
    #[inline]
    fn drop(&mut self) {
        unsafe { ::lilv_sys::lilv_nodes_free(self.ptr.ptr() as *mut _) }
    }
}

impl<'w, T: NodeType<'w>> Deref for OwnedNodeList<'w, T> {
    type Target = NodeList<'w, T>;

    #[inline]
    fn deref(&self) -> &<Self as Deref>::Target {
        self.ptr
    }
}