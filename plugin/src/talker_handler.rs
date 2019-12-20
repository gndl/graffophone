use crate::talker::Talker;

pub struct TalkerHandlerBase {
    pub id: String,
    pub model: String,
    pub category: String,
}

impl TalkerHandlerBase {
    pub fn new(id: &str, model: &str, category: &str) -> Self {
        Self {
            id: id.to_string(),
            model: model.to_string(),
            category: category.to_string(),
        }
    }

    pub fn id<'a>(&'a self) -> &'a String {
        &self.id
    }
    pub fn model<'a>(&'a self) -> &'a String {
        &self.model
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
    fn model<'a>(&'a self) -> &'a String {
        &self.base().model
    }
    fn category<'a>(&'a self) -> &'a String {
        &self.base().category
    }

    fn make(&self) -> Result<Box<dyn Talker>, failure::Error>;
}
