use std::cell::RefCell;
use std::sync::atomic::{AtomicU32, Ordering};

static ID_COUNT: AtomicU32 = AtomicU32::new(1);

pub struct Identifier {
    id: u32,
    name: String,
}

impl Identifier {
    pub fn new(name: &str, model: &str) -> Self {
	let id = ID_COUNT.fetch_add(1, Ordering::SeqCst);

        let name_model_fm = |close_char| {
            if model.is_empty() {
                format!("{}{}{}", name, id, close_char)
            } else {
                format!("{}{} {}{}", name, model, id, close_char)
            }
        };

        let name = if name.is_empty() {
            if model.is_empty() {
                format!("{}", id)
            } else {
                format!("{} {}", model, id)
            }
        } else {
            if name.ends_with("(") {
                name_model_fm(")")
            } else if name.ends_with("[") {
                name_model_fm("]")
            } else {
                name.to_string()
            }
        };

        Self { id, name }
    }

    pub fn id(&self) -> u32 {
        self.id
    }
    pub fn name<'a>(&'a self) -> &'a str {
        &self.name
    }
    pub fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
    }
    pub fn depends_of(&self, id: u32) -> bool {
        self.id == id
    }
}
pub type RIdentifier = RefCell<Identifier>;

pub trait Identifiable {
    fn id(&self) -> u32;
    fn name(&self) -> String;
    fn set_name(&self, name: &str);
    fn depends_of(&self, id: u32) -> bool;
}
