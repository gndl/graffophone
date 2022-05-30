use std::cell::RefCell;
use std::rc::Rc;

use talker::identifier::{Id, Index};
use talker::talker::RTalker;

use session::band::Operation;
use session::event_bus::{EventBus, Notification, REventBus};
use session::factory::Factory;
use session::session::Session;
use session::state::State;

const GSR: &str = "
Sinusoidal 2#Sin 1
> freq <- 440
> phase <- 0

mixer 1#mixer 1
> volume <- 0.01
> Tracks.0.In <- 2:Out
> Tracks.0.gain <- 1
> Tracks.0.left <- 1
> Tracks.0.right <- 1
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

    pub fn new_session(&mut self) {
        self.exit();
        self.session = Session::new(GSR.to_string()).unwrap();
        self.notify_new_session();
    }

    pub fn open_session(&mut self, filename: &str) {
        self.exit();
        match Session::from_file(filename) {
            Ok(session) => {
                self.session = session;
                self.notify_new_session();
            }
            Err(e) => self.notify_error(e),
        }
    }

    pub fn save_session(&mut self) {
        self.manage_result(self.session.save());
    }

    pub fn save_session_as(&mut self, filename: &str) {
        let res = self.session.save_as(filename);
        self.manage_result(res);
    }

    pub fn session<'a>(&'a self) -> &'a Session {
        &self.session
    }

    pub fn event_bus<'a>(&'a self) -> &'a REventBus {
        &self.event_bus
    }

    pub fn notify(&self, notification: Notification) {
        self.event_bus().borrow().notify(notification);
    }

    pub fn notify_new_session(&mut self) {
        self.notify(Notification::NewSession(
            self.session.filename().to_string(),
        ));
        let state = self.session.state();
        self.notify(Notification::State(state));
    }

    pub fn notify_error(&self, error: failure::Error) {
        self.notify(Notification::Error(format!("{}", error)));
    }

    pub fn manage_result(&self, result: Result<(), failure::Error>) {
        match result {
            Ok(()) => {}
            Err(e) => self.notify_error(e),
        }
    }

    pub fn manage_state_result(&self, result: Result<State, failure::Error>) {
        match result {
            Ok(state) => self.event_bus.borrow().notify(Notification::State(state)),
            Err(e) => self.notify_error(e),
        }
    }

    pub fn init(&mut self) {
        let res = Factory::visit(|factory| {
            Ok(self.event_bus().borrow().notify(Notification::TalkersRange(
                factory.get_categorized_talkers_label_model(),
            )))
        });
        self.manage_result(res);
        self.notify_new_session();
    }

    pub fn find_talker<'a>(&'a self, talker_id: Id) -> Option<&'a RTalker> {
        self.session.talkers().get(&talker_id)
    }

    pub fn add_talker(&mut self, talker_model: &str) {
        let res = self.session.add_talker(talker_model);
        self.manage_state_result(res);
    }

    pub fn set_talker_data(&mut self, talker_id: Id, data: &str) {
        self.modify_band(&Operation::SetTalkerData(talker_id, data.to_string()));
        self.notify(Notification::TalkerChanged);
    }

    pub fn find_compatible_hum_with_voice_in_ear(
        &self,
        talker_id: Id,
        ear_idx: Index,
        voice_tkr_id: Id,
        voice_port: Index,
    ) -> Result<Index, failure::Error> {
        // TODO : do the job
        Ok(0)
    }

    pub fn modify_band(&mut self, operation: &Operation) {
        let res = self.session.modify_band(operation);
        self.manage_state_result(res);
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

    fn monitor_state(session_presenter_reference: &RSessionPresenter) {
        let this = session_presenter_reference.clone();

        glib::timeout_add_seconds_local(1, move || {
            let state = this.borrow_mut().session.state();
            this.borrow().notify(Notification::State(state));

            match state {
                State::Playing => glib::Continue(true),
                _ => glib::Continue(false),
            }
        });
    }

    pub fn play_or_pause(&mut self, monitor: &RSessionPresenter) {
        let (res, monitor_state) = match self.session.state() {
            State::Stopped => (self.session.start(), true),
            State::Playing => (self.session.pause(), false),
            State::Paused => (self.session.play(), true),
            State::Exited => (Err(failure::err_msg("Player exited")), false),
        };
        self.manage_state_result(res);

        if monitor_state {
            SessionPresenter::monitor_state(monitor);
        }
    }

    pub fn stop(&mut self) {
        let res = self.session.stop();
        self.manage_state_result(res);
    }

    pub fn exit(&mut self) {
        let res = self.session.exit();
        self.manage_state_result(res);
    }
}
