use std::cell::RefCell;

pub struct Identifier {
    id: u32,
    name: String,
}

impl Identifier {
    pub fn new(name: &str, kind: &str, id: u32) -> Self {
        let name_kind_fm = |close_char| {
            if kind.is_empty() {
                format!("{}{}{}", name, id, close_char)
            } else {
                format!("{}{} {}{}", name, kind, id, close_char)
            }
        };

        let name = if name.is_empty() {
            if kind.is_empty() {
                "".to_string()
            } else {
                format!("{} {}", kind, id)
            }
        } else {
            if name.ends_with("(") {
                name_kind_fm(")")
            } else if name.ends_with("[") {
                name_kind_fm("]")
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
pub type MIdentifier = RefCell<Identifier>;

pub trait Identifiable {
    fn id(&self) -> u32;
    fn name(&self) -> String;
    fn set_name(&self, name: &str) -> ();
    fn depends_of(&self, id: u32) -> bool;
}
