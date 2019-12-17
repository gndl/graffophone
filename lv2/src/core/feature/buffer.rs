use core::feature::descriptor::FeatureDescriptor;
use core::feature::feature::{Feature, RawFeatureDescriptor};
use std::iter;
use std::rc::Rc;

pub struct FeatureBuffer /*<'a>*/ {
    features: Vec<FeatureDescriptor /*<'a>*/>, // TODO: SmallVec here ?
    descriptors: Vec<*const ::lv2_sys::LV2_Feature>,
}

impl FeatureBuffer /*<'a>*/ {
    /*
    pub fn new<'f: 'i, 'i, T: IntoIterator<Item = &'i FeatureDescriptor<'f>>>(
        iter: T,
    ) -> FeatureBuffer<'f> {
        FeatureBuffer::from_vec(iter.into_iter().cloned().collect())
    }
     */
    pub fn new() -> Self {
        FeatureBuffer {
            features: Vec::new(),
            descriptors: Vec::new(),
        }
    }
    pub fn from_vec(features: Vec<FeatureDescriptor /*<'a>*/>) -> Self {
        let descriptors = features
            .iter()
            .map(|f| &f.inner as *const _)
            .chain(iter::once(::std::ptr::null()))
            .collect();

        FeatureBuffer {
            features,
            descriptors,
        }
    }

    #[inline]
    pub fn raw_descriptors_with_nul(&self) -> *const *const RawFeatureDescriptor {
        self.descriptors.as_ptr() as *const *const RawFeatureDescriptor
    }

    pub fn find<'a, T: Feature>(&'a self) -> Option<&'a T> {
        self.features
            .iter()
            .filter_map(FeatureDescriptor::as_feature)
            .next()
    }

    #[inline]
    pub fn features(&self) -> &[FeatureDescriptor] {
        &self.features
    }
}
pub type SharedFeatureBuffer = Rc<FeatureBuffer>;
