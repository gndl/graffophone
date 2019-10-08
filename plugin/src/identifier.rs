pub struct Identifier {
    id: u32,
    name: String,
}

impl Identifier {
    pub fn new() -> Self {
        Self {
            id: 0,
            name: String::new(),
        }
    }

    pub fn get_id(&self) -> u32 {
        self.id
    }
    pub fn get_name(&self) -> &str {
        &self.name
    }
    pub fn set_name(&mut self, name: &String) {
        self.name = name.to_string();
    }
}

pub trait Identifiable {
    fn get_id(&self) -> u32;
    fn get_name(&self) -> String;
    fn set_name(&self, name: String) -> ();
}
