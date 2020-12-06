use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

use talker::identifier::Identifiable;
use talker::identifier::{Id, Index};
use talker::talker::{RTalker, Talker, TalkerBase};

use crate::session_presenter::RSessionPresenter;
use session::event_bus::{Notification, REventBus};

pub struct GraphControler {
    selected_talk: Option<(Id, Index, Index)>,
    selected_voice: Option<(Id, Index)>,
    new_talker: Option<Id>,
    selected_talkers: HashSet<Id>,
    control_key_pressed: bool,
    shift_key_pressed: bool,
    alt_key_pressed: bool,
    session_presenter: RSessionPresenter,
}

pub type RGraphControler = Rc<RefCell<GraphControler>>;

impl GraphControler {
    pub fn new(session_presenter: &RSessionPresenter) -> GraphControler {
        Self {
            selected_talk: None,
            selected_voice: None,
            new_talker: None,
            selected_talkers: HashSet::new(),
            control_key_pressed: false,
            shift_key_pressed: false,
            alt_key_pressed: false,
            session_presenter: session_presenter.clone(),
        }
    }

    pub fn new_ref(session_presenter: &RSessionPresenter) -> RGraphControler {
        Rc::new(RefCell::new(GraphControler::new(session_presenter)))
    }

    pub fn set_control_key_pressed(&mut self, v: bool) {
        self.control_key_pressed = v;
    }
    pub fn set_shift_key_pressed(&mut self, v: bool) {
        self.shift_key_pressed = v;
    }
    pub fn set_alt_key_pressed(&mut self, v: bool) {
        self.alt_key_pressed = v;
    }

    pub fn select_talker(&mut self, talker: &RTalker) -> Result<Vec<Notification>, failure::Error> {
        let tkr_id = talker.borrow().id();
        let mut notifications = Vec::new();

        if self.control_key_pressed || self.selected_talkers.len() < 2 {
            if self.selected_talkers.contains(&tkr_id) {
                self.selected_talkers.remove(&tkr_id);
                notifications.push(Notification::TalkerUnselected(tkr_id));
            } else {
                self.selected_talkers.insert(tkr_id);
                notifications.push(Notification::TalkerSelected(tkr_id));
            }
        } else {
            for id in &self.selected_talkers {
                notifications.push(Notification::TalkerUnselected(*id));
            }
            self.selected_talkers.clear();
            self.selected_talkers.insert(tkr_id);
            notifications.push(Notification::TalkerSelected(tkr_id));
        }
        Ok(notifications)
    }

    pub fn set_talker_name(
        &mut self,
        talker: &RTalker,
        v: &str,
    ) -> Result<Vec<Notification>, failure::Error> {
        talker.borrow_mut().set_name(v);
        Ok(vec![Notification::TalkerRenamed(talker.borrow().id())])
    }

    pub fn set_talker_data(
        &mut self,
        talker: &RTalker,
        v: &str,
        fly: bool,
    ) -> Result<Vec<Notification>, failure::Error> {
        talker.borrow_mut().set_data_from_string(v)?;

        if !fly {
            return Ok(vec![Notification::TalkerChanged]);
        }
        Ok(vec![])
    }

    pub fn set_talker_ear_talk_value_by_index(
        &mut self,
        talker: &RTalker,
        ear_idx: usize,
        talk_idx: usize,
        value: f32,
        fly: bool,
    ) -> Result<Vec<Notification>, failure::Error> {
        talker
            .borrow()
            .set_ear_talk_value_by_index(ear_idx, talk_idx, value)?;
        if !fly {
            return Ok(vec![Notification::TalkerChanged]);
        }
        Ok(vec![])
    }

    pub fn add_talker_ear_talk_value_by_index(
        &mut self,
        talker: &RTalker,
        ear_idx: usize,
        value: f32,
    ) -> Result<Vec<Notification>, failure::Error> {
        talker
            .borrow()
            .add_ear_talk_value_by_index(ear_idx, value)?;
        Ok(vec![Notification::TalkerChanged])
    }

    pub fn select_ear_talk(
        &mut self,
        talker: &RTalker,
        ear_idx: usize,
        talk_idx: usize,
    ) -> Result<Vec<Notification>, failure::Error> {
        let tkr_id = talker.borrow().id();
        let mut notifications = Vec::new();

        match self.selected_talk {
            Some((prev_tkr_id, prev_ear_idx, prev_talk_idx)) => {
                if tkr_id == prev_tkr_id && ear_idx == prev_ear_idx && talk_idx == prev_talk_idx {
                    self.selected_talk = None;
                } else {
                    self.selected_talk = Some((tkr_id, ear_idx, talk_idx));
                    notifications.push(Notification::EarSelected(tkr_id, ear_idx, talk_idx));
                }
                notifications.push(Notification::EarUnselected(
                    prev_tkr_id,
                    prev_ear_idx,
                    prev_talk_idx,
                ));
            }
            None => match self.selected_voice {
                None => {
                    self.selected_talk = Some((tkr_id, ear_idx, talk_idx));
                    notifications.push(Notification::EarSelected(tkr_id, ear_idx, talk_idx));
                }
                Some((voice_tkr_id, voice_port)) => {
                    if voice_tkr_id != tkr_id {
                        if let Some(voice_tkr) =
                            self.session_presenter.borrow().find_talker(voice_tkr_id)
                        {
                            talker.borrow().set_ear_talk_voice_by_index(
                                ear_idx, talk_idx, voice_tkr, voice_port,
                            )?;
                            self.selected_voice = None;
                            notifications
                                .push(Notification::VoiceUnselected(voice_tkr_id, voice_port));
                            notifications.push(Notification::TalkerChanged);
                        }
                    }
                }
            },
        }
        Ok(notifications)
    }

    pub fn select_voice(
        &mut self,
        talker: &RTalker,
        port: usize,
    ) -> Result<Vec<Notification>, failure::Error> {
        let tkr_id = talker.borrow().id();
        let mut notifications = Vec::new();

        match self.selected_voice {
            Some((prev_tkr_id, prev_port)) => {
                if tkr_id == prev_tkr_id && port == prev_port {
                    self.selected_voice = None
                } else {
                    self.selected_voice = Some((tkr_id, port));
                    notifications.push(Notification::VoiceSelected(tkr_id, port));
                }
                self.selected_voice = Some((tkr_id, port));
                notifications.push(Notification::VoiceUnselected(prev_tkr_id, prev_port));
            }
            None => match self.selected_talk {
                None => {
                    self.selected_voice = Some((tkr_id, port));
                    notifications.push(Notification::VoiceSelected(tkr_id, port));
                }
                Some((ear_tkr_id, ear_idx, talk_idx)) => {
                    if tkr_id != ear_tkr_id {
                        if let Some(ear_tkr) =
                            self.session_presenter.borrow().find_talker(ear_tkr_id)
                        {
                            talker
                                .borrow()
                                .set_ear_talk_voice_by_index(ear_idx, talk_idx, talker, port)?;
                            self.selected_talk = None;
                            notifications
                                .push(Notification::EarUnselected(ear_tkr_id, ear_idx, talk_idx));
                            notifications.push(Notification::TalkerChanged);
                        }
                    }
                }
            },
        }
        Ok(notifications)
    }

    pub fn show_voice(
        &self,
        talker: &RTalker,
        port: usize,
    ) -> Result<Vec<Notification>, failure::Error> {
        //trace("showVoice tkr "^soi talker#getId^" port "^soi port);
        Ok(vec![Notification::TalkSelected(talker.borrow().id(), port)])
    }

    pub fn add_ear_talk(
        &mut self,
        talker: &RTalker,
        ear_idx: Index,
    ) -> Result<Vec<Notification>, failure::Error> {
        match self.selected_voice {
            None => (),
            Some((voice_tkr_id, voice_port)) => {
                if let Some(voice_tkr) = self.session_presenter.borrow().find_talker(voice_tkr_id) {
                    talker
                        .borrow()
                        .add_ear_talk_voice_by_index(ear_idx, voice_tkr, voice_port)?;
                    return Ok(vec![Notification::TalkerChanged]);
                }
            }
        }
        Ok(vec![])
    }

    pub fn sup_ear_talk(
        &self,
        talker: &RTalker,
        ear_idx: usize,
        talk_idx: usize,
    ) -> Result<Vec<Notification>, failure::Error> {
        talker
            .borrow_mut()
            .sup_ear_talk_by_index(ear_idx, talk_idx)?;
        Ok(vec![Notification::TalkerChanged])
    }

    pub fn add_new_talker(
        &mut self,
        talker: &RTalker,
    ) -> Result<Vec<Notification>, failure::Error> {
        let mut notifications = Vec::new();

        match self.selected_voice {
            None => (),
            Some((voice_tkr_id, voice_port)) => {
                notifications.push(Notification::VoiceUnselected(voice_tkr_id, voice_port))
            }
        }
        match self.selected_talk {
            None => (),
            Some((talk_tkr_id, ear_idx, talk_idx)) => {
                notifications.push(Notification::EarUnselected(talk_tkr_id, ear_idx, talk_idx))
            }
        }

        match self.new_talker {
            None => (),
            Some(tkr_id) => {
                self.selected_voice = Some((tkr_id, 0));
                notifications.push(Notification::VoiceSelected(tkr_id, 0));
            }
        }

        self.new_talker = Some(talker.borrow().id());
        Ok(notifications)
    }

    pub fn new_talker(&self) -> Option<Id> {
        self.new_talker
    }

    pub fn sup_talker(&self, talker: &RTalker) -> Result<Vec<Notification>, failure::Error> {
        self.session_presenter.borrow_mut().sup_talker(talker);
        Ok(vec![Notification::TalkerChanged])
    }
}
