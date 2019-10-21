use node::inner_node::node_from_ptr;
use node::{String, Uri};
use world::World;

#[derive(Clone)]
pub struct PluginClass<'w> {
    pub(crate) ptr: *const ::lilv_sys::LilvPluginClass,
    pub(crate) world: &'w World,
}

impl<'w> PluginClass<'w> {
    pub(crate) fn new(ptr: *const ::lilv_sys::LilvPluginClass, world: &World) -> PluginClass {
        PluginClass { ptr, world }
    }

    pub fn parent_uri(&self) -> &Uri<'w> {
        unsafe { node_from_ptr(::lilv_sys::lilv_plugin_class_get_parent_uri(self.ptr)) }
    }

    pub fn uri(&self) -> &Uri<'w> {
        unsafe { node_from_ptr(::lilv_sys::lilv_plugin_class_get_uri(self.ptr)) }
    }

    pub fn label(&self) -> &String<'w> {
        unsafe { node_from_ptr(::lilv_sys::lilv_plugin_class_get_label(self.ptr)) }
    }
}
