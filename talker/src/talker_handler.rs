use crate::talker::Talker;

pub struct TalkerHandlerBase {
    pub categories: Vec<String>,
    pub model: String,
    pub label: String,
}

impl TalkerHandlerBase {
    pub fn with_multi_categories(categories: Vec<String>, model: &str, label: &str) -> Self {
        Self {
            categories: categories,
            model: model.to_string(),
            label: label.to_string(),
        }
    }
    pub fn builtin(category: &str, model: &str, label: &str) -> Self {
        let builtin_label = format!("G {}", label);
        TalkerHandlerBase::with_multi_categories(vec![category.to_string()], &model, &builtin_label)
    }

    pub fn categories<'a>(&'a self) -> &'a Vec<String> {
        &self.categories
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

    fn categories<'a>(&'a self) -> &'a Vec<String> {
        &self.base().categories
    }
    fn model<'a>(&'a self) -> &'a String {
        &self.base().model
    }
    fn label<'a>(&'a self) -> &'a String {
        &self.base().label
    }

    fn make(&self) -> Result<Box<dyn Talker>, failure::Error>;
}
