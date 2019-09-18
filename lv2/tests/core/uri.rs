use lv2::core::uri::*;

#[test]
fn test_uri() {
    let uri: UriBuf = "http://hello".into_uri().unwrap();
    assert_eq!(format!("{:?}", uri), "<http://hello>");
}
