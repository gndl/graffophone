use plugin::Plugin;
use node::Uri;
use node::inner_node::node_get_ptr;
use std::path::Path;
use std::ffi::CString;
use std::fmt;

#[derive(Debug)]
pub struct WorldCreationError;

impl fmt::Display for WorldCreationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("Unable to create Lilv world")
    }
}

impl ::std::error::Error for WorldCreationError {}

pub struct World {
    ptr: &'static mut ::lilv_sys::LilvWorld
}

impl World {
    pub fn new() -> Result<World, WorldCreationError> {
        let world = World::empty()?;
        unsafe { ::lilv_sys::lilv_world_load_all(world.ptr) };
        Ok(world)
    }

    pub fn empty() -> Result<World, WorldCreationError> {
        let ptr = unsafe { ::lilv_sys::lilv_world_new() };
        if ptr.is_null() { return Err(WorldCreationError) }
        Ok( World { ptr: unsafe {&mut *ptr} })
    }

    pub fn load_bundle(&self, path: &Path) {
        use std::ops::Deref;
        let uri_str = String::from("file://") + path.to_str().unwrap() + "/";
        let uri_cstr = CString::new(uri_str).unwrap();
        let uri = Uri::from_cstr(self, &uri_cstr).unwrap();
        let uri_node = node_get_ptr(uri.deref());
        unsafe { ::lilv_sys::lilv_world_load_bundle(self.ptr as *const _ as *mut _, uri_node) }
    }

    #[inline]
    pub(crate) fn ptr(&self) -> &'static mut ::lilv_sys::LilvWorld {
        unsafe { &mut *(self.ptr as *const _ as *mut _) }
    }

    pub(crate) fn plugins_collection(&self) -> *const ::lilv_sys::LilvPlugins {
        unsafe { ::lilv_sys::lilv_world_get_all_plugins(self.ptr) }
    }

    pub fn plugins(&self) -> PluginsIter {
        PluginsIter::new(self.plugins_collection(), self)
    }

    pub fn get_plugin_by_uri<'a, U: ?Sized>(&'a self, uri: &U) -> Option<Plugin<'a>> where Uri<'a>: PartialEq<U>{
        self.plugins().filter(|p| p.uri() == uri).next()
    }
}

impl Drop for World {
    //noinspection RsDropRef FIXME: Bug in IntelliJ-Rust
    fn drop(&mut self) {
        unsafe {
            ::lilv_sys::lilv_world_free(self.ptr)
        }
    }
}

pub struct PluginsIter<'a> {
    collection: *const ::lilv_sys::LilvPlugins,
    iter: *mut ::lilv_sys::LilvIter,
    world: &'a World
}

impl<'a> PluginsIter<'a> {
    fn new(collection: *const ::lilv_sys::LilvPlugins, world: &'a World) -> PluginsIter<'a> {
        PluginsIter {
            collection, world,
            iter: unsafe { ::lilv_sys::lilv_plugins_begin(collection) },
        }
    }
}

impl<'a> Iterator for PluginsIter<'a> {
    type Item = Plugin<'a>;

    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        if unsafe { ::lilv_sys::lilv_plugins_is_end(self.collection, self.iter) } { return None };
        let plugin = unsafe { ::lilv_sys::lilv_plugins_get(self.collection, self.iter) };
        self.iter = unsafe { ::lilv_sys::lilv_plugins_next(self.collection, self.iter) };
        Some(Plugin::new(plugin, self.world))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl<'a> ExactSizeIterator for PluginsIter<'a> {
    fn len(&self) -> usize {
        unsafe { ::lilv_sys::lilv_plugins_size(self.collection) as usize }
    }
}
