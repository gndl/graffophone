use crate::talker::Talker;

pub struct TalkerHandlerBase {
    pub id: String,
    pub name: String,
    pub category: String,
}

impl TalkerHandlerBase {
    pub fn new(id: &str, name: &str, category: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            category: category.to_string(),
        }
    }

    pub fn id<'a>(&'a self) -> &'a String {
        &self.id
    }
    pub fn name<'a>(&'a self) -> &'a String {
        &self.name
    }
    pub fn category<'a>(&'a self) -> &'a String {
        &self.category
    }
}

pub trait TalkerHandler {
    fn base<'a>(&'a self) -> &'a TalkerHandlerBase;

    fn id<'a>(&'a self) -> &'a String {
        &self.base().id
    }
    fn name<'a>(&'a self) -> &'a String {
        &self.base().name
    }
    fn category<'a>(&'a self) -> &'a String {
        &self.base().category
    }

    fn make(&self) -> Result<Box<dyn Talker>, failure::Error>;
}
