mod imp;

use gtk::glib;

glib::wrapper! {
    pub struct TalkerObject(ObjectSubclass<imp::TalkerObject>);
}

impl TalkerObject {
    pub fn new(label: &str, model: &str) -> Self {
        glib::Object::builder()
            .property("label", label)
            .property("model", model)
            .build()
    }
}
