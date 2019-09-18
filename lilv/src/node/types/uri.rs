use std::ffi::{CStr, CString};
use std::fmt;
use node::inner_node::*;
use world::World;
use node::node::NodeType;
use node::node::Node;

use node::types::unknown::UnknownNode;

pub struct Uri<'w> {
    _node: InnerNode<'w>
}

impl<'a> Uri<'a> {

    #[inline]
    pub fn from_str<T: Into<Vec<u8>>>(world: &World, string: T) -> Option<Node<Uri>> {
        Self::from_cstr(world, &CString::new(string).unwrap())
    }

    #[inline]
    pub fn from_cstr<'w>(world: &'w World, string: &CStr) -> Option<Node<'w, Uri<'w>>> {
        unsafe { Self::from_cstr_unbound(world.ptr(), string) }
    }

    #[inline]
    pub fn from_lv2_uri<'w>(world: &'w World, uri: &::lv2::core::uri::Uri) -> Node<'w, Uri<'w>> {
        unsafe { Self::from_cstr_unbound(world.ptr(), uri) }.unwrap()
    }

    pub(crate) unsafe fn from_cstr_unbound<'n>(world: *mut ::lilv_sys::LilvWorld, string: &CStr) -> Option<Node<'n, Uri<'n>>> {
        let node = ::lilv_sys::lilv_new_uri(world, string.as_ptr());
        if node.is_null() { return None }

        Some(Node::new(node))
    }

    pub fn to_cstr(&self) -> &CStr {
        let lilv_str = unsafe { ::lilv_sys::lilv_node_as_uri(node_get_ptr(self)) };
        unsafe { CStr::from_ptr(lilv_str) }
    }

    #[inline]
    pub fn as_lv2_uri(&self) -> &::lv2::core::uri::Uri {
        unsafe { ::lv2::core::uri::Uri::from_cstr_unchecked(self.to_cstr()) }
    }
}

impl<'a> fmt::Display for Uri<'a> {
    fn fmt(&self, f: & mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self.as_lv2_uri(), f)
    }
}

impl<'a> fmt::Debug for Uri<'a> {
    fn fmt(&self, f: & mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self.as_lv2_uri(), f)
    }
}

impl<'a> AsRef<Uri<'a>> for Uri<'a> {
    fn as_ref(&self) -> &Uri<'a> {
        self
    }
}

unsafe impl<'w> NodeType<'w> for Uri<'w> {
    const NAME: &'static str = "Uri";

    fn node_matches_type(node: &UnknownNode) -> bool {
        unsafe { ::lilv_sys::lilv_node_is_uri(node_get_ptr(node)) }
    }
}

impl<'w> PartialEq for Uri<'w> {
    #[inline]
    fn eq(&self, other: &Uri) -> bool {
        self == other.to_cstr()
    }
}

impl<'w> PartialEq<CStr> for Uri<'w> {
    #[inline]
    fn eq(&self, other: &CStr) -> bool {
        self.to_cstr() == other
    }
}

impl<'w> PartialEq<[u8]> for Uri<'w> {
    #[inline]
    fn eq(&self, other: &[u8]) -> bool {
        self.to_cstr().to_bytes() == other
    }
}

impl<'w> PartialEq<str> for Uri<'w> {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        self == other.as_bytes()
    }
}

impl<'w> PartialEq<::lv2::core::uri::Uri> for Uri<'w> {
    #[inline]
    fn eq(&self, other: &::lv2::core::uri::Uri) -> bool {
        self == other.as_cstr()
    }
}
