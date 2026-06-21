use std::cell::RefCell;
use std::rc::Rc;

pub type Id = u32;
pub type Index = usize;

#[derive(Clone)]
pub struct Identifier {
    id: Id,
    name: String,
    model: String,
}

impl Identifier {
    pub fn new(id: Id, name: &str, model: &str) -> Self {
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

        Self {
            id,
            name,
            model: model.to_string(),
        }
    }

    pub fn id(&self) -> Id {
        self.id
    }
    pub fn set_id(&mut self, id: Id) {
        self.id = id;
    }
    pub fn name<'a>(&'a self) -> &'a str {
        &self.name
    }
    pub fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
    }

    pub fn model<'a>(&'a self) -> &'a str {
        &self.model
    }
    pub fn set_model(&mut self, model: &str) {
        self.model = model.to_string();
    }

    pub fn is(&self, id: Id) -> bool {
        self.id == id
    }
}

pub type RIdentifier = RefCell<Identifier>;

pub trait Identifiable {
    fn id(&self) -> Id;
    fn set_id(&self, id: Id);
    fn name(&self) -> String;
    fn set_name(&self, name: &str);
}
pub type CIdentifiable = RefCell<dyn Identifiable>;
pub type RIdentifiable = Rc<CIdentifiable>;
