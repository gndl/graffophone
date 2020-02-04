use crate::state::State;
use std::cell::RefCell;
use std::rc::Rc;

pub enum Notification {
    State(State),
    Session,
    Tick(i64),
    TimeRange(i64, i64),
    Pause,
    End,
    Volume(f32),
    TalkersRange(Vec<(String, Vec<String>)>),
    NewTalker,
    TalkerChanged,
    TalkerRenamed(i64),
    TalkerSelected(i64),
    TalkerUnselected(i64),
    EarSelected(i64, i64),
    EarUnselected(i64, i64),
    VoiceSelected(i64, i64),
    VoiceUnselected(i64, i64),
    TalkSelected(i64, i64),
    CurveAdded,
    CurveRemoved,
    Info(String),
    Warning(String),
    Error(String),
}

pub trait TObserver {
    fn observe(&mut self, notification: &Notification);
}

pub type RObserver = Rc<RefCell<dyn TObserver>>;

pub struct EventBus {
    observers: Vec<RObserver>,
}

impl EventBus {
    pub fn new() -> EventBus {
        Self {
            observers: Vec::new(),
        }
    }

    //let observers : (notification -> unit) list ref = ref []

    //let addObserver o = observers := o :: !observers
    pub fn add_observer(&mut self, observer: RObserver) {
        self.observers.push(observer);
    }

    pub fn notify(&self, notification: Notification) {
        for obs in &self.observers {
            obs.borrow_mut().observe(&notification);
        }
        //  List.iter(fun observe -> observe notification) !observers
    }

    pub fn async_notify(&self, _notification: Notification) {
        // GtkThread.async notify notif
    }
}

pub type REventBus = Rc<EventBus>;
