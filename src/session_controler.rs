use std::cell::RefCell;
use std::rc::Rc;

use session::event_bus::{Notification, REventBus};
use session::factory::Factory;
use session::session::Session;

pub struct SessionControler {
    session: Session,
    talkers: Vec<String>,
}
pub type RSessionControler = Rc<RefCell<SessionControler>>;

impl SessionControler {
    pub fn new() -> SessionControler {
        Self {
            session: Session::new(),
            talkers: Vec::new(),
        }
    }

    pub fn new_ref() -> RSessionControler {
        Rc::new(RefCell::new(SessionControler::new()))
    }

    pub fn event_bus<'a>(&'a self) -> &'a REventBus {
        self.session.event_bus()
    }

    pub fn manage_result(&mut self, result: Result<(), failure::Error>) {
        match result {
            Ok(()) => {}
            Err(e) => self
                .session
                .event_bus()
                .borrow()
                .notify(Notification::Error(format!("{}", e))),
        }
    }

    pub fn init(&mut self) {
        let res = Factory::visit(|factory| {
            Ok(self
                .session
                .event_bus()
                .borrow()
                .notify(Notification::TalkersRange(
                    factory.get_categorized_talkers_label_model(),
                )))
        });
        self.manage_result(res);
    }

    pub fn talkers<'a>(&'a self) -> &'a Vec<String> {
        &self.talkers
    }
    pub fn add_talker(&mut self, talker_model: &str) {
        self.talkers.push(talker_model.to_string());
    }

    pub fn set_start_tick(&mut self, t: i64) {
        let res = self.session.set_start_tick(t);
        self.manage_result(res);
    }

    pub fn set_end_tick(&mut self, t: i64) {
        let res = self.session.set_end_tick(t);
        self.manage_result(res);
    }

    pub fn new_band(&mut self) {
        let res = self.session.new_band();
        self.manage_result(res);
    }
    /*
        pub fn start(&mut self) -> Result<(), failure::Error> {
            self.session.start()
        }
    */
    pub fn play(&mut self) {
        let res = self.session.play();
        self.manage_result(res);
    }
    /*
        pub fn pause(&mut self) -> Result<(), failure::Error> {
            self.session.pause()
        }
    */
    pub fn stop(&mut self) {
        let res = self.session.stop();
        self.manage_result(res);
    }
}
