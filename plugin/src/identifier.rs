use std::cell::RefCell;

pub struct Identifier {
    id: u32,
    name: String,
}

impl Identifier {
    pub fn new(name: &str, model: &str, id: u32) -> Self {
        let name_model_fm = |close_char| {
            if model.is_empty() {
                format!("{}{}{}", name, id, close_char)
            } else {
                format!("{}{} {}{}", name, model, id, close_char)
            }
        };

        let name = if name.is_empty() {
            if model.is_empty() {
                "".to_string()
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

        Self { id: id, name: name }
    }

    pub fn id(&self) -> u32 {
        self.id
    }
    pub fn name<'a>(&'a self) -> &'a String {
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
