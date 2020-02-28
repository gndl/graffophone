use lv2::core::{uri::Uri, ExtensionData, SharedFeatureBuffer};
use port::buffer::BufferType;
use port::buffer::PortBuffer;
use port::PortHandle;
use port::PortIndex;
use std::ffi::CStr;
//use std::marker::PhantomData;
use Plugin;

mod buffers;
pub mod errors;
use self::buffers::*;
use self::errors::*;
use std::borrow::Borrow;
/*
pub struct ResolvedPlugin<'p, 'l, 'f: 'l> {
    plugin: &'p Plugin<'p>,
    feature_list: &'l FeatureBuffer<'f>,
}

impl<'p, 'l, 'f: 'l> ResolvedPlugin<'p, 'l, 'f> {
    pub(crate) fn new(
        plugin: &'p Plugin<'p>,
        feature_list: &'l FeatureBuffer<'f>,
    ) -> Result<Self, MissingFeatureError> {
        // TODO: actually check stuff here
        Ok(Self {
            plugin,
            feature_list,
        })
    }

    pub fn instantiate(
        &self,
        sample_rate: f64,
    ) -> Result<PluginInstance<'f>, PluginInstantiationError> {
        PluginInstance::new(self.plugin, sample_rate, self.feature_list)
    }
}
*/
//pub struct PluginInstance<'f> {
pub struct PluginInstance {
    ptr: *mut ::lilv_sys::LilvInstance,
    buffers: Buffers,
    activated: bool,
    //    _marker: PhantomData<&'f u8>,
    //    features: SharedFeatureBuffer,
}

//impl<'f> PluginInstance<'f> {
impl PluginInstance {
    pub fn new(
        plugin: &Plugin,
        sample_rate: f64,
        //        features: &FeatureBuffer<'f>,
        shared_features: SharedFeatureBuffer,
    ) -> Result<Self, PluginInstantiationError> {
        let features = &*shared_features;
        let ptr = unsafe {
            ::lilv_sys::lilv_plugin_instantiate(
                plugin.ptr,
                sample_rate,
                features.raw_descriptors_with_nul() as _,
            )
        };
        if ptr.is_null() {
            return Err(PluginInstantiationError);
        }

        let buffers = Buffers::new(plugin);

        Ok(Self {
            ptr,
            buffers,
            activated: false,
            //            _marker: PhantomData,
            //          features: shared_features,
        })
    }

    #[inline]
    pub fn get_port_buffer<T: BufferType>(&self, index: PortIndex) -> Option<&T::BufferImpl> {
        self.buffers.get::<T>(index)
    }

    #[inline]
    pub fn all_port_buffers<T: BufferType>(&self) -> impl Iterator<Item = &T::BufferImpl> {
        self.buffers.all::<T>()
    }

    pub fn connect_port<P: BufferType, R: Borrow<P::BufferImpl> + 'static>(
        &mut self,
        handle: PortHandle<P>,
        buffer: R,
    ) {
        unsafe {
            let buffer = self.buffers.set::<P, R>(handle.index(), buffer);
            ::lilv_sys::lilv_instance_connect_port(self.ptr, handle.index(), buffer.get_ptr())
        }
    }

    pub fn connect_port_index<P: BufferType, R: Borrow<P::BufferImpl> + 'static>(
        &mut self,
        index: u32,
        buffer: R,
    ) {
        unsafe {
            let buffer = self.buffers.set::<P, R>(index, buffer);
            ::lilv_sys::lilv_instance_connect_port(self.ptr, index, buffer.get_ptr())
        }
    }

    pub fn get_extension_data<E: ExtensionData>(&self) -> Option<&E> {
        unsafe {
            (::lilv_sys::lilv_instance_get_extension_data(self.ptr, E::URI.as_ptr() as _)
                as *const E)
                .as_ref()
        }
    }

    #[inline]
    pub fn get_uri(&self) -> &Uri {
        unsafe {
            Uri::from_cstr_unchecked(CStr::from_ptr(::lilv_sys::lilv_instance_get_uri(self.ptr)))
        }
    }

    #[inline]
    pub fn activate(&mut self) {
        unsafe {
            ::lilv_sys::lilv_instance_activate(self.ptr);
        }
        self.activated = true;
    }

    #[inline]
    pub fn run(&mut self, sample_count: u32) {
        if !self.activated {
            panic!("Tried to run instance without activating it first")
        }
        unsafe { ::lilv_sys::lilv_instance_run(self.ptr, sample_count) }
    }

    #[inline]
    pub fn deactivate(&mut self) {
        unsafe {
            ::lilv_sys::lilv_instance_deactivate(self.ptr);
        }
        self.activated = false;
    }
    #[inline]
    pub fn activated(&self) -> bool {
        self.activated
    }
}

impl Drop for PluginInstance /*<'a>*/ {
    fn drop(&mut self) {
        if self.activated {
            self.deactivate()
        }
        unsafe { ::lilv_sys::lilv_instance_free(self.ptr) }
    }
}
