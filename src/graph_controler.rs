use std::cell::RefCell;
use std::rc::Rc;

pub struct GraphControler {
    talkers: Vec<String>,
}

pub type RGraphControler = Rc<RefCell<GraphControler>>;

impl GraphControler {
    pub fn new() -> GraphControler {
        Self {
            talkers: Vec::new(),
        }
    }

    pub fn new_ref() -> RGraphControler {
        Rc::new(RefCell::new(GraphControler::new()))
    }

    pub fn talkers<'a>(&'a self) -> &'a Vec<String> {
        &self.talkers
    }
    pub fn add_talker(&mut self, talker_model: &str) {
        self.talkers.push(talker_model.to_string());
    }
}
