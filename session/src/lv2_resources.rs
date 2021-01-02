use std::cell::RefCell;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::ffi::{CStr, CString};
use std::iter::Extend;
use std::rc::Rc;
use std::sync::atomic::AtomicU32;
use std::sync::Arc;
use std::sync::RwLock;

//use sha3::Digest;

pub struct Lv2Resources {
    pub world: lilv::World,
    pub audio_port: lilv::Node,
    pub atom_port: lilv::Node,
    pub control_port: lilv::Node,
    pub input_port: lilv::Node,
    pub output_port: lilv::Node,
    pub urid_map: UridMapFeature<'static>,
}

impl Lv2Resources {
    pub fn new() -> Lv2Resources {
        let world = lilv::World::new();
        world.load_all();

        let audio_port = world.new_uri("http://lv2plug.in/ns/lv2core#AudioPort");
        let atom_port = world.new_uri("http://lv2plug.in/ns/ext/atom#AtomPort");
        let control_port = world.new_uri("http://lv2plug.in/ns/lv2core#ControlPort");
        let input_port = world.new_uri("http://lv2plug.in/ns/lv2core#InputPort");
        let output_port = world.new_uri("http://lv2plug.in/ns/lv2core#OutputPort");

        Self {
            world,
            audio_port,
            atom_port,
            control_port,
            input_port,
            output_port,
            urid_map: UridMapFeature::default(),
        }
    }

    pub fn is_audio(&self, port: &lilv::Port) -> bool {
        port.classes().contains(&self.audio_port)
    }

    pub fn is_atom(&self, port: &lilv::Port) -> bool {
        port.classes().contains(&self.atom_port)
    }

    pub fn is_control(&self, port: &lilv::Port) -> bool {
        port.classes().contains(&self.control_port)
    }

    pub fn is_output(&self, port: &lilv::Port) -> bool {
        port.classes().contains(&self.output_port)
    }

    pub fn is_input(&self, port: &lilv::Port) -> bool {
        port.classes().contains(&self.input_port)
    }
}

/// The underlying buffer backing the data for an atom event.
type Lv2AtomEventBuffer = [u8; 16];

/// An single atom event.
#[repr(packed)]
struct Lv2AtomEvent {
    header: lv2_raw::LV2AtomEvent,
    pub buffer: Lv2AtomEventBuffer,
}

impl Lv2AtomEvent {
    /// Create a new atom event with the given time and type. The event can be filled in by setting
    /// the bytes in buffer and calling `set_size`.
    pub fn new(time_in_frames: i64, my_type: u32) -> Lv2AtomEvent {
        let mut event: Lv2AtomEvent = unsafe { std::mem::MaybeUninit::uninit().assume_init() };
        event.header.time_in_frames = time_in_frames;
        event.header.body.mytype = my_type;
        event.header.body.size = 0;
        event
    }

    /// Set the size of the atom. Must be less than or equal to the size of the buffer.
    pub fn set_size(&mut self, size: usize) {
        debug_assert!(size < self.buffer.len(), "{} < {}", size, self.buffer.len());
        self.header.body.size = size as u32;
    }

    /// Return a pointer to the header of the atom.
    #[allow(safe_packed_borrows)]
    pub fn as_ptr(&self) -> *const lv2_raw::LV2AtomEvent {
        &self.header
    }
}

/// An atom sequence.
struct Lv2AtomSequence {
    buffer: Vec<lv2_raw::LV2AtomSequence>,
}

impl Lv2AtomSequence {
    /// Create a new sequence that can hold about desired_capacity bytes.
    pub fn new(desired_capacity: usize) -> Lv2AtomSequence {
        let len = desired_capacity / std::mem::size_of::<lv2_raw::LV2AtomSequence>();
        let mut buffer = Vec::with_capacity(len);
        buffer.resize_with(len, || lv2_raw::LV2AtomSequence {
            atom: lv2_raw::LV2Atom { size: 0, mytype: 0 },
            body: lv2_raw::LV2AtomSequenceBody { unit: 0, pad: 0 },
        });
        let mut seq = Lv2AtomSequence { buffer };
        seq.clear();
        seq
    }

    /// Clear all events in the sequence.
    #[inline(always)]
    pub fn clear(&mut self) {
        unsafe { lv2_raw::atomutils::lv2_atom_sequence_clear(self.as_mut_ptr()) }
    }

    /// Append an event to the sequence. If there is no capacity for it, then it will not be
    /// appended.
    #[inline(always)]
    pub fn append_event(&mut self, event: &Lv2AtomEvent) {
        unsafe {
            lv2_raw::atomutils::lv2_atom_sequence_append_event(
                self.as_mut_ptr(),
                self.capacity() as u32,
                event.as_ptr(),
            )
        };
    }

    /// Return a mutable pointer to the underlying data.
    pub fn as_mut_ptr(&mut self) -> *mut lv2_raw::LV2AtomSequence {
        self.buffer.as_mut_ptr()
    }

    /// Get the capacity of the sequence.
    pub fn capacity(&self) -> usize {
        let slice: &[lv2_raw::LV2AtomSequence] = &self.buffer;
        std::mem::size_of_val(slice)
    }
}

impl std::fmt::Debug for Lv2AtomSequence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let capacity = self.capacity();
        f.debug_struct("Lv2AtomSequence")
            .field("capacity", &capacity)
            .finish()
    }
}

/// An implementation of LV2 URID map.
enum UridMapFeatureImpl<'a> {
    /// A native Rust implementation.
    Native(Box<UridMapFeatureNativeImpl>),
    /// An abstract implementation exposed through the LV2_URID_Map handle and function pointer.
    Abstract(&'a lv2_raw::LV2UridMap),
}

impl<'a> UridMapFeatureImpl<'a> {
    pub fn map(&self, uri: &CStr) -> u32 {
        match self {
            UridMapFeatureImpl::Native(f) => f.map(uri),
            UridMapFeatureImpl::Abstract(f) => {
                let handle = f.handle;
                (f.map)(handle, uri.as_ptr())
            }
        }
    }
}

/// Provides the urid map feature for LV2. See documentation for urid map at
/// http://lv2plug.in/ns/ext/urid/#map.
// The fields are actually referenced as void ptrs within feature and data.
#[allow(dead_code)]
pub struct UridMapFeature<'a> {
    feature: lv2_raw::LV2Feature,
    data: Option<Box<lv2_raw::LV2UridMap>>,
    urid_map_impl: UridMapFeatureImpl<'a>,
}

unsafe impl Send for UridMapFeature<'static> {}
unsafe impl Sync for UridMapFeature<'static> {}

impl Default for UridMapFeature<'static> {
    /// Create the default instance for UridMapFeature with no registered URIs. URIs will register
    /// themselves with the `get` method.
    fn default() -> UridMapFeature<'static> {
        let mut urid_map_impl: Box<UridMapFeatureNativeImpl> = Box::default();
        let mut data = Box::new(lv2_raw::LV2UridMap {
            handle: urid_map_impl.as_mut() as *mut UridMapFeatureNativeImpl
                as *mut std::ffi::c_void,
            map: urid_map_feature_native_impl_map,
        });
        UridMapFeature {
            feature: lv2_raw::LV2Feature {
                uri: UridMapFeature::URI.as_ptr() as *const ::std::os::raw::c_char,
                data: data.as_mut() as *mut lv2_raw::LV2UridMap as *mut std::ffi::c_void,
            },
            data: Some(data),
            urid_map_impl: UridMapFeatureImpl::Native(urid_map_impl),
        }
    }
}

extern "C" fn urid_map_feature_native_impl_map(
    handle: *mut std::ffi::c_void,    /*Type is UridMapFeatureNativeImpl*/
    uri: *const std::os::raw::c_char, /*CStr*/
) -> u32 {
    let self_ptr = handle as *const UridMapFeatureNativeImpl;
    unsafe {
        match self_ptr.as_ref() {
            Some(self_ref) => self_ref.map(CStr::from_ptr(uri)),
            None => {
                eprintln!("URID Map had null handle for UridMapFeatureNativeImpl.");
                0
            }
        }
    }
}

impl<'a> UridMapFeature<'a> {
    /// The URI for the urid map LV2 feature.
    pub const URI: &'static str = "http://lv2plug.in/ns/ext/urid#map\0";

    /// Get the urid map as an LV2_feature.
    pub fn as_lv2_feature(&self) -> &lv2_raw::LV2Feature {
        &self.feature
    }

    /// Get the id for the given uri. If the uri does not have an ID, it will be registered
    /// with a new one.
    ///
    /// Note: This method makes uses of mutexes and heap based maps; do not run in a realtime
    /// context. If needed, cache the returned IDs.
    pub fn map(&self, uri: &CStr) -> u32 {
        self.urid_map_impl.map(uri)
    }
}

impl<'a> From<&'a lv2_raw::LV2UridMap> for UridMapFeature<'a> {
    fn from(map: &'a lv2_raw::LV2UridMap) -> UridMapFeature<'a> {
        UridMapFeature {
            feature: lv2_raw::LV2Feature {
                uri: lv2_raw::LV2_URID__MAP.as_ptr() as *const ::std::os::raw::c_char,
                data: map as *const lv2_raw::LV2UridMap as *mut std::ffi::c_void,
            },
            data: None, /*The data is borrowed from map*/
            urid_map_impl: UridMapFeatureImpl::Abstract(map),
        }
    }
}

impl<'a> TryFrom<&'a lv2_raw::LV2Feature> for UridMapFeature<'a> {
    type Error = failure::Error;

    /// Convert the feature into a UridMapFeature. If the LV2 feature is not a URID map feature,
    /// then an error is returned.
    fn try_from(feature: &'a lv2_raw::LV2Feature) -> Result<UridMapFeature<'a>, failure::Error> {
        let feature_uri = unsafe { CStr::from_ptr(feature.uri) };
        if feature_uri.to_bytes() == lv2_raw::LV2_URID__MAP.as_bytes() {
            let urid_map_ptr = feature.data as *const lv2_raw::LV2UridMap;
            match unsafe { urid_map_ptr.as_ref() } {
                Some(r) => Ok(UridMapFeature::from(r)),
                None => Err(failure::err_msg("feature data is null")),
            }
        } else {
            Err(failure::err_msg("feature is not URID map"))
        }
    }
}

/// Implementation for uri map LV2 feature.
struct UridMapFeatureNativeImpl {
    map: RwLock<HashMap<CString, u32>>,
    next_id: AtomicU32,
}

impl Default for UridMapFeatureNativeImpl {
    /// Create a new UridMapFeatureNativeImpl. With no registered URIs.
    fn default() -> UridMapFeatureNativeImpl {
        UridMapFeatureNativeImpl {
            map: RwLock::default(),
            next_id: AtomicU32::new(1),
        }
    }
}

impl UridMapFeatureNativeImpl {
    /// Get the ID for the given uri. If the URI is not registered, then it will be registered
    /// with a new unique ID. This function makes use of a heap based hash map and mutex so it is
    /// not suitable for realtime execution. Results for important URIs should be cached.
    pub fn map(&self, uri: &CStr) -> u32 {
        if let Some(id) = self.map.read().unwrap().get(uri).copied() {
            return id;
        };
        let mut map = self.map.write().unwrap();
        // We check if the ID is present again in case it was inserted in the time between
        // releasing the read lock and regaining the write lock.
        if let Some(id) = map.get(uri).copied() {
            return id;
        }
        let id = self
            .next_id
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        println!("Mapped URI {:?} to {}.", uri, id);
        map.insert(CString::from(uri), id);
        id
    }
}

fn create_id(uri: &str) -> String {
    uri.to_string()
    // let hash = hex::encode(sha3::Sha3_256::digest(uri.as_bytes()));
    // format!("lv2_{}", hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_id_is_stable() {
        assert_eq!(
            create_id("http://drobilla.net/plugins/mda/EPiano"),
            "lv2_ec91337841c3308d790e6387353f5690835925f1230b8471682856e1733625d8",
        );
    }
}
