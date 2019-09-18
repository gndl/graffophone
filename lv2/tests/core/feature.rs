use lv2::core::*;

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
struct MyFeature(u8);

unsafe impl Feature for MyFeature {
    const URI: &'static [u8] = b"http://proko.eu/my_feature\0";
}

#[derive(Copy, Clone)]
struct MyVecFeature<'a>(pub &'a Vec<u8>);

unsafe impl<'a> Feature for MyVecFeature<'a> {
    const URI: &'static [u8] = b"http://proko.eu/vec_feature\0";
}

#[test]
fn simple_feature() {
    let f = MyFeature(42);

    let desc: FeatureDescriptor = (&f).into();
    assert_eq!(desc.as_feature::<MyFeature>().unwrap(), &MyFeature(42))
}

#[test]
fn lifetime_feature() {
    let buf = vec![42, 69];
    let f = MyVecFeature(&buf);

    let desc: FeatureDescriptor = (&f).into();
    let fo: &MyVecFeature = desc.as_feature().unwrap();
    assert_eq!(fo.0, &vec![42u8, 69])
}

#[test]
fn feature_buffer() {
    let vec = vec![42, 69];
    let feature = MyFeature(21);
    let feature_vec = MyVecFeature(&vec);

    let buf = FeatureBuffer::new(&[feature.descriptor(), feature_vec.descriptor()]);

    assert!(buf.features()[0].as_feature::<MyFeature>().is_some());
    assert!(buf.features()[1].as_feature::<MyVecFeature>().is_some());
    assert_eq!(buf.features().len(), 2);

    let feature: &MyFeature = buf.find().unwrap();
    assert_eq!(feature, &MyFeature(21))
}

#[test]
fn feature_list() {
    let vec = vec![42, 69];
    let feature = MyFeature(21);
    let feature_vec = MyVecFeature(&vec);
    let buf = FeatureBuffer::new(&[feature.descriptor(), feature_vec.descriptor()]);

    let list = unsafe { FeatureList::from_raw(buf.raw_descriptors_with_nul()) };
    assert_eq!(list.find::<MyFeature>(), Some(&MyFeature(21)))
}
/*
pub struct MyFeatureSet<'a> {
    my_feature: Option<MyFeature>,
    my_vec_feature: Option<MyVecFeature<'a>>
}

impl<'a> FeatureSet<'a> for MyFeatureSet<'a> {
    fn to_list(&'a self) -> FeatureBuffer<'a>  {
        let mut vec = Vec::with_capacity(2);

        if let Some(feature) = &self.my_feature { vec.push(Feature::descriptor(feature)) }
        if let Some(feature) = &self.my_vec_feature { vec.push(Feature::descriptor(feature)) }

        FeatureBuffer::from_vec(vec)
    }
}

static FEATURESET_URIS: &'static [&'static [u8]] = &[
    MyFeature::URI,
    MyVecFeature::URI
];

#[test]
fn feature_sets() { // TODO
    let buf = vec![42, 69];

    let feature_set = MyFeatureSet {
        my_feature: Some(MyFeature(21)),
        my_vec_feature: Some(MyVecFeature(&buf)),
    };

    let list = feature_set.to_list();


}
*/