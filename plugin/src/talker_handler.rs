use crate::talker::Talker;

pub struct TalkerHandlerBase {
    pub category: String,
    pub model: String,
    pub label: String,
}

impl TalkerHandlerBase {
    pub fn new(category: &str, model: &str, label: &str) -> Self {
        Self {
            category: category.to_string(),
            model: model.to_string(),
            label: label.to_string(),
        }
    }

    pub fn category<'a>(&'a self) -> &'a String {
        &self.category
    }
    pub fn model<'a>(&'a self) -> &'a String {
        &self.model
    }
    pub fn label<'a>(&'a self) -> &'a String {
        &self.label
    }
}

pub trait TalkerHandler {
    fn base<'a>(&'a self) -> &'a TalkerHandlerBase;

    fn category<'a>(&'a self) -> &'a String {
        &self.base().category
    }
    fn model<'a>(&'a self) -> &'a String {
        &self.base().model
    }
    fn label<'a>(&'a self) -> &'a String {
        &self.base().label
    }

    fn make(&self) -> Result<Box<dyn Talker>, failure::Error>;
}
