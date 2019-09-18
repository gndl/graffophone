use node::inner_node::InnerNode;
use std::ffi::CStr;
use std::fmt;
use node::node::NodeType;
use node::inner_node::node_get_ptr;
use node::types::unknown::UnknownNode;

pub struct String<'w> {
    _node: InnerNode<'w>
}

impl<'w> String<'w> {
    #[inline]
    pub fn to_cstr(&self) -> &CStr {
        let lilv_str = unsafe { ::lilv_sys::lilv_node_as_string(node_get_ptr(self)) };
        unsafe { CStr::from_ptr(lilv_str) }
    }

    #[inline]
    pub fn to_str(&self) -> &str {
        self.to_cstr().to_str().unwrap()
    }
}

impl<'w> fmt::Display for String<'w> {
    fn fmt(&self, f: & mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self.to_str(), f)
    }
}

impl<'w> fmt::Debug for String<'w> {
    fn fmt(&self, f: & mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self.to_str(), f)
    }
}

unsafe impl<'w> NodeType<'w> for String<'w> {
    const NAME: &'static str = "String";

    #[inline]
    fn node_matches_type(node: &UnknownNode) -> bool {
        unsafe { ::lilv_sys::lilv_node_is_string(node_get_ptr(node)) }
    }
}

impl<'w> PartialEq for String<'w> {
    #[inline]
    fn eq(&self, other: &String) -> bool {
        self == other.to_cstr()
    }
}

impl<'w> PartialEq<CStr> for String<'w> {
    #[inline]
    fn eq(&self, other: &CStr) -> bool {
        self.to_cstr() == other
    }
}

impl<'w> PartialEq<[u8]> for String<'w> {
    #[inline]
    fn eq(&self, other: &[u8]) -> bool {
        self.to_cstr().to_bytes() == other
    }
}

impl<'w> PartialEq<str> for String<'w> {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        self == other.as_bytes()
    }
}
