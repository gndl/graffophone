use std::cell::RefCell;
use std::rc::Rc;

use session::event_bus::{EventBus, Notification, REventBus};
use session::factory::Factory;
use session::session::Session;
use session::state::State;

const GSR: &str = "
Sinusoidal 1#Sinusoidal_1 
> frequence 440
> phase 0

track 2#track_2
> I 1#Sinusoidal_1:O
> gain 1

mixer 5#mixer_5
> volume 1
> track 2#track_2
";

pub struct SessionPresenter {
    session: Session,
    event_bus: REventBus,
}
pub type RSessionPresenter = Rc<RefCell<SessionPresenter>>;

impl SessionPresenter {
    pub fn new() -> SessionPresenter {
        Self {
            session: Session::new(GSR.to_string()).unwrap(),
            event_bus: EventBus::new_ref(),
        }
    }

    pub fn new_ref() -> RSessionPresenter {
        Rc::new(RefCell::new(SessionPresenter::new()))
    }

    pub fn session<'a>(&'a self) -> &'a Session {
        &self.session
    }

    pub fn event_bus<'a>(&'a self) -> &'a REventBus {
        &self.event_bus
    }

    pub fn manage_result(&mut self, result: Result<(), failure::Error>) {
        match result {
            Ok(()) => {}
            Err(e) => self
                .event_bus()
                .borrow()
                .notify(Notification::Error(format!("{}", e))),
        }
    }

    pub fn manage_state_result(&mut self, result: Result<State, failure::Error>) {
        match result {
            Ok(state) => self.event_bus.borrow().notify(Notification::State(state)),
            Err(e) => self
                .event_bus()
                .borrow()
                .notify(Notification::Error(format!("{}", e))),
        }
    }

    pub fn init(&mut self) {
        let res = Factory::visit(|factory| {
            Ok(self.event_bus().borrow().notify(Notification::TalkersRange(
                factory.get_categorized_talkers_label_model(),
            )))
        });
        self.manage_result(res);
        self.event_bus
            .borrow()
            .notify(Notification::State(self.session.state()))
    }

    pub fn add_talker(&mut self, talker_model: &str) {
        self.session.add_talker(talker_model);
    }

    pub fn set_start_tick(&mut self, t: i64) {
        let res = self.session.set_start_tick(t);
        self.manage_state_result(res);
        // self.event_bus
        //     .borrow()
        //     .notify(Notification::TimeRange(t, self.end_tick));
    }

    pub fn set_end_tick(&mut self, t: i64) {
        let res = self.session.set_end_tick(t);
        self.manage_state_result(res);
    }

    pub fn new_session(&mut self) {
        self.session = Session::new(GSR.to_string()).unwrap();
    }

    fn monitor_state(session_presenter_reference: &RSessionPresenter) {
        let this = session_presenter_reference.clone();

        gtk::timeout_add_seconds(1, move || {
            let state = this.borrow_mut().session.state();
            this.borrow()
                .event_bus
                .borrow()
                .notify(Notification::State(state));

            match state {
                State::Playing => glib::Continue(true),
                _ => glib::Continue(false),
            }
        });
    }

    pub fn play_or_pause(&mut self, monitor: &RSessionPresenter) {
        //        let mut this = session_presenter_reference.borrow_mut();

        let res = match self.session.state() {
            State::Stopped => {
                SessionPresenter::monitor_state(monitor);
                self.session.start()
            }
            State::Playing => self.session.pause(),
            State::Paused => {
                SessionPresenter::monitor_state(monitor);
                self.session.play()
            }
            State::Exited => Err(failure::err_msg("Player exited")),
        };
        self.manage_state_result(res);
    }

    pub fn stop(&mut self) {
        let res = self.session.stop();
        self.manage_state_result(res);
    }
}
