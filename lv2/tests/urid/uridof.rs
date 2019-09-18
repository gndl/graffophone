use lv2::urid::features::URIDMap;
use lv2::urid::{URID, URIDOf, SimpleMapper};

use lv2::units::units::Frame;

#[test]
fn urid_mapper() {
    let mapper = SimpleMapper::new();
    let feature_map = URIDMap::new(&mapper);

    let urid = URIDOf::<Frame>::map(&feature_map);

    assert_eq!(urid, URID::new(1).unwrap());
}