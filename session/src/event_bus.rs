use std::cell::RefCell;
use std::rc::Rc;

use talker::identifier::{Id, Index};

use crate::state::State;

pub enum Notification {
    State(State),
    NewSession(String),
    SessionSaved,
    SessionSavedAs(String),
    Tick(i64),
    TimeRange(i64, i64),
    Pause,
    End,
    Volume(f32),
    TalkersRange(Vec<(String, Vec<(String, String)>)>),
    NewTalker,
    TalkerChanged,
    TalkerRenamed(Id),
    EarSelected(Id, Index, Index, Index),
    EarUnselected(Id, Index, Index, Index),
    EarAddInSelected(Id, Index, Index, Index),
    EarAddInUnselected(Id, Index, Index, Index),
    EarValueSelected(Id, Index, Index, Index),
    VoiceSelected(Id, Index),
    VoiceUnselected(Id, Index),
    TalkSelected(Id, Index),
    SelectionChanged,
    EditTalkerData(Id),
    CurveAdded,
    CurveRemoved,
    Info(String),
    Warning(String),
    Error(String),
}
pub type RObserver = Box<dyn Fn(&Notification)>;

pub struct EventBus {
    observers: Vec<RObserver>,
}

pub type REventBus = Rc<RefCell<EventBus>>;

impl EventBus {
    pub fn new() -> EventBus {
        Self {
            observers: Vec::new(),
        }
    }

    pub fn new_ref() -> REventBus {
        Rc::new(RefCell::new(EventBus::new()))
    }

    pub fn add_observer(&mut self, observer: RObserver) {
        self.observers.push(observer);
    }

    pub fn notify(&self, notification: Notification) {
        for obs in &self.observers {
            obs(&notification);
        }
    }

    pub fn notify_error(&self, error: failure::Error) {
        self.notify(Notification::Error(format!("{}", error)));
    }

    pub fn notify_notifications_result(
        &self,
        notifications_result: Result<Vec<Notification>, failure::Error>,
    ) {
        match notifications_result {
            Ok(notifications) => {
                for notification in notifications {
                    self.notify(notification);
                }
            }
            Err(e) => self.notify_error(e),
        }
    }
}
