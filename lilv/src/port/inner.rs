use plugin::Plugin;
use node::{UnknownNode, NodeType, Uri, Node, String, NodeList};
use std::fmt;
use node::inner_node::node_from_ptr;
use port::port::PortIndex;
use port::port::Port;

#[derive(Clone)]
pub struct InnerPort<'p> {
    ptr: *const ::lilv_sys::LilvPort,
    plugin: &'p Plugin<'p>
}

impl<'a> InnerPort<'a> {
    pub(crate) unsafe fn from_ptr_unchecked<'p>(ptr: *const ::lilv_sys::LilvPort, plugin: &'p Plugin<'p>) -> InnerPort<'p> {
        InnerPort {
            plugin,
            ptr
        }
    }

    #[inline]
    pub fn name(&self) -> Node<String> {
        unsafe { Node::new(::lilv_sys::lilv_port_get_name(self.plugin.ptr, self.ptr)) }
    }

    #[inline]
    pub fn symbol(&self) -> &String {
        unsafe { node_from_ptr(::lilv_sys::lilv_port_get_symbol(self.plugin.ptr, self.ptr)) }
    }

    pub fn range(&self) -> PortRange<UnknownNode> {
        PortRange::new(self)
    }

    pub fn plugin(&self) -> &'a Plugin<'a> {
        self.plugin
    }

    pub fn index(&self) -> PortIndex {
        unsafe { ::lilv_sys::lilv_port_get_index(self.plugin.ptr, self.ptr) }
    }

    #[inline]
    pub fn classes(&self) -> &NodeList<Uri> {
        unsafe { NodeList::new(::lilv_sys::lilv_port_get_classes(self.plugin.ptr, self.ptr)) }
    }

    pub fn fmt(&self, port_name: &str, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct(port_name)
            .field("name", &self.name())
            .finish()
    }
}

impl<'p> Port<'p> for InnerPort<'p> {
    #[inline]
    fn classes(&self) -> &NodeList<Uri> {
        self.classes()
    }

    #[inline]
    fn plugin(&self) -> &'p Plugin<'p> {
        self.plugin()
    }

    #[inline]
    fn name(&self) -> Node<String> {
        self.name()
    }

    #[inline]
    fn symbol(&self) -> &String {
        self.symbol()
    }

    #[inline]
    fn index(&self) -> u32 {
        self.index()
    }
}

#[derive(Debug)]
pub struct PortRange<'w, T: fmt::Debug + NodeType<'w> + 'w> {
    default: Option<Node<'w, T>>,
    minimum: Option<Node<'w, T>>,
    maximum: Option<Node<'w, T>>
}

impl<'w, T: NodeType<'w>> PortRange<'w, T> {
    pub(crate) fn new(port: &'w InnerPort<'w>) -> PortRange<T> {
        let mut default: *mut ::lilv_sys::LilvNode = ::std::ptr::null_mut();
        let mut minimum: *mut ::lilv_sys::LilvNode = ::std::ptr::null_mut();
        let mut maximum: *mut ::lilv_sys::LilvNode = ::std::ptr::null_mut();

        unsafe {
            ::lilv_sys::lilv_port_get_range(
                port.plugin.ptr,
                port.ptr,
                &mut default as *mut _,
                &mut minimum as *mut _,
                &mut maximum as *mut _
            );

            PortRange {
                default: Node::new_opt(default),
                minimum: Node::new_opt(minimum),
                maximum: Node::new_opt(maximum),
            }
        }
    }

    pub fn default(&self) -> Option<&T> {
        unimplemented!()
    }

    pub fn minimum(&self) -> Option<&T> {
        unimplemented!()
    }

    pub fn maximum(&self) -> Option<&T> {
        unimplemented!()
    }

    pub fn is<U: NodeType<'w>>(&self) -> bool {
        unimplemented!()
    }

    pub fn to<U: NodeType<'w>>(self) -> Result<PortRange<'w, U>, Self> {
        unimplemented!()
    }
}