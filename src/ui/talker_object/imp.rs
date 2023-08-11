use std::cell::RefCell;
use glib::{ParamSpec, Properties, Value};
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

// Object holding the state
#[derive(Properties, Default)]
#[properties(wrapper_type = super::TalkerObject)]
pub struct TalkerObject {
    #[property(get, set)]
    label:RefCell<String>,
    #[property(get, set)]
    model:RefCell<String>,
}

// The central trait for subclassing a GObject
#[glib::object_subclass]
impl ObjectSubclass for TalkerObject {
    const NAME: &'static str = "GraffophoneTalkerObject";
    type Type = super::TalkerObject;
}
// Trait shared by all GObjects
impl ObjectImpl for TalkerObject {
    fn properties() -> &'static [ParamSpec] {
        Self::derived_properties()
    }

    fn set_property(&self, id: usize, value: &Value, pspec: &ParamSpec) {
        self.derived_set_property(id, value, pspec)
    }

    fn property(&self, id: usize, pspec: &ParamSpec) -> Value {
        self.derived_property(id, pspec)
    }
}
