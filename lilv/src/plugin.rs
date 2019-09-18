use world::World;
use port::inner::InnerPort;
use port::{ UnknownInputPort, UnknownOutputPort };
use std::fmt::Debug;
use std::fmt::Formatter;
use node::{Node, String, Uri};
use instance::{ResolvedPlugin, errors::MissingFeatureError};
use node::inner_node::node_from_ptr;
use node::OwnedNodeList;
use ::lv2::core::FeatureBuffer;

#[derive(Clone)]
pub struct Plugin<'w> {
    pub(crate) ptr: *const ::lilv_sys::LilvPlugin,
    pub(crate) world: &'w World
}

impl<'w> Plugin<'w> {
    pub(crate) fn new(ptr: *const ::lilv_sys::LilvPlugin, world: &World) -> Plugin {
        Plugin { ptr, world }
    }

    pub fn uri(&self) -> &Uri<'w> {
        unsafe { node_from_ptr(::lilv_sys::lilv_plugin_get_uri(self.ptr)) }
    }

    pub fn name(&self) -> Node<String> {
        unsafe { Node::new(::lilv_sys::lilv_plugin_get_name(self.ptr)) }
    }

    pub fn world(&self) -> &'w World {
        self.world
    }

    pub(crate) fn ports(&self) -> PortsIter {
        PortsIter::new(self)
    }

    pub fn inputs(&self) -> impl Iterator<Item=UnknownInputPort> {
        self.ports().filter_map(UnknownInputPort::from_inner)
    }

    pub fn outputs(&self) -> impl Iterator<Item=UnknownOutputPort> {
        self.ports().filter_map(UnknownOutputPort::from_inner)
    }

    pub fn resolve<'p, 'l, 'f>(&'p self, features: &'l FeatureBuffer<'f>) -> Result<ResolvedPlugin<'p, 'l, 'f>, MissingFeatureError> {
        ResolvedPlugin::new(self, features)
    }
/*
    pub fn instantiate<'f>(&self, sample_rate: f64, features: &FeatureList<'f>) -> Result<PluginInstance<'f>, PluginInstantiationError> {
        PluginInstance::new(self, sample_rate, features)
    }
*/
    pub fn supported_features(&self) -> OwnedNodeList<Uri> {
        unsafe { OwnedNodeList::new( ::lilv_sys::lilv_plugin_get_supported_features(self.ptr)) }
    }

    pub fn required_features(&self) -> OwnedNodeList<Uri> {
        unsafe { OwnedNodeList::new( ::lilv_sys::lilv_plugin_get_required_features(self.ptr)) }
    }

    pub fn optional_features(&self) -> OwnedNodeList<Uri> {
        unsafe { OwnedNodeList::new( ::lilv_sys::lilv_plugin_get_optional_features(self.ptr)) }
    }
}

impl<'a> Debug for Plugin<'a> {
    fn fmt(&self, f: & mut Formatter) -> Result<(), ::std::fmt::Error> {
        ::std::fmt::Debug::fmt(self.uri(), f)
    }
}

pub struct PortsIter<'p> {
    ports_count: u32,
    i: u32,
    plugin: &'p Plugin<'p>
}

impl<'a> PortsIter<'a> {
    fn new<'p>(plugin: &'p Plugin) -> PortsIter<'p> {
        PortsIter {
            plugin, i: 0,
            ports_count: unsafe { ::lilv_sys::lilv_plugin_get_num_ports(plugin.ptr) }
        }
    }
}

impl<'a> Iterator for PortsIter<'a> {
    type Item = InnerPort<'a>;

    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        if self.i >= self.ports_count { return None }
        let port = unsafe { ::lilv_sys::lilv_plugin_get_port_by_index(self.plugin.ptr, self.i) };
        self.i += 1;
        unsafe {
            Some(InnerPort::from_ptr_unchecked(port, self.plugin))
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let ports_count = self.ports_count as usize;
        (ports_count, Some(ports_count))
    }
}

impl<'a> ExactSizeIterator for PortsIter<'a> {
    fn len(&self) -> usize {
        self.ports_count as usize
    }
}
