use std::cell::RefCell;
use std::rc::Rc;

use talker::identifier::{Id, Index};

use crate::state::State;

pub enum Notification {
    State(State),
    Session,
    Tick(i64),
    TimeRange(i64, i64),
    Pause,
    End,
    Volume(f32),
    TalkersRange(Vec<(String, Vec<(String, String)>)>),
    NewTalker,
    TalkerChanged,
    TalkerRenamed(Id),
    TalkerSelected(Id),
    TalkerUnselected(Id),
    EarSelected(Id, Index, Index),
    EarUnselected(Id, Index, Index),
    VoiceSelected(Id, Index),
    VoiceUnselected(Id, Index),
    TalkSelected(Id, Index),
    SelectionChanged,
    CurveAdded,
    CurveRemoved,
    Info(String),
    Warning(String),
    Error(String),
}

// pub trait Observer {
//     fn observe(&mut self, notification: &Notification);
// }

// pub type RObserver = Rc<RefCell<dyn Observer>>;
// pub type Observer = dyn FnMut(&Notification);
// pub type RObserver = RefCell<dyn Observer>;
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
            //            obs.borrow_mut().observe(&notification);
            obs(&notification);
        }
    }

    pub fn async_notify(&self, _notification: Notification) {
        // GtkThread.async notify notif
    }
}
