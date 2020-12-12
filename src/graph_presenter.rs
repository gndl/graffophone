use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

use talker::identifier::Identifiable;
use talker::identifier::{Id, Index};
use talker::talker::{RTalker, Talker, TalkerBase};

use session::band::Operation;
use session::event_bus::{Notification, REventBus};

use crate::session_presenter::RSessionPresenter;

pub struct GraphPresenter {
    selected_talk: Option<(Id, Index, Index)>,
    selected_voice: Option<(Id, Index)>,
    new_talker: Option<Id>,
    selected_talkers: HashSet<Id>,
    control_key_pressed: bool,
    shift_key_pressed: bool,
    alt_key_pressed: bool,
    session_presenter: RSessionPresenter,
}

pub type RGraphPresenter = Rc<RefCell<GraphPresenter>>;

impl GraphPresenter {
    pub fn new(session_presenter: &RSessionPresenter) -> GraphPresenter {
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

    pub fn new_ref(session_presenter: &RSessionPresenter) -> RGraphPresenter {
        Rc::new(RefCell::new(GraphPresenter::new(session_presenter)))
    }

    pub fn talker_selected(&self, talker_id: Id) -> bool {
        self.selected_talkers.contains(&talker_id)
    }

    pub fn voice_selected(&self, talker_id: Id, voice_idx: Index) -> bool {
        match self.selected_voice {
            None => false,
            Some((tkr_id, vc_idx)) => talker_id == tkr_id && voice_idx == vc_idx,
        }
    }

    pub fn ear_talk_selected(&self, talker_id: Id, ear_idx: Index, talk_idx: Index) -> bool {
        match self.selected_talk {
            None => false,
            Some((tkr_id, e_idx, t_idx)) => {
                talker_id == tkr_id && ear_idx == e_idx && talk_idx == t_idx
            }
        }
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

    pub fn select_talker(&mut self, talker_id: Id) -> Result<Vec<Notification>, failure::Error> {
        let mut notifications = Vec::new();

        if self.control_key_pressed || self.selected_talkers.len() < 2 {
            if self.selected_talkers.contains(&talker_id) {
                self.selected_talkers.remove(&talker_id);
                notifications.push(Notification::TalkerUnselected(talker_id));
            } else {
                self.selected_talkers.insert(talker_id);
                notifications.push(Notification::TalkerSelected(talker_id));
            }
        } else {
            for id in &self.selected_talkers {
                notifications.push(Notification::TalkerUnselected(*id));
            }
            self.selected_talkers.clear();
            self.selected_talkers.insert(talker_id);
            notifications.push(Notification::TalkerSelected(talker_id));
        }
        notifications.push(Notification::SelectionChanged);
        Ok(notifications)
    }

    pub fn set_talker_name(
        &mut self,
        talker: &RTalker,
        v: &str,
    ) -> Result<Vec<Notification>, failure::Error> {
        talker.borrow_mut().set_name(v);
        Ok(vec![
            Notification::TalkerRenamed(talker.borrow().id()),
            Notification::TalkerChanged,
        ])
    }

    pub fn set_talker_data(
        &mut self,
        talker_id: Id,
        data: &str,
        fly: bool,
    ) -> Result<Vec<Notification>, failure::Error> {
        self.session_presenter
            .borrow_mut()
            .modify_band(&Operation::SetTalkerData(talker_id, data.to_string()));

        if !fly {
            return Ok(vec![Notification::TalkerChanged]);
        }
        Ok(vec![])
    }

    pub fn set_talker_ear_talk_value_by_index(
        &mut self,
        talker_id: Id,
        ear_idx: Index,
        talk_idx: Index,
        value: f32,
        fly: bool,
    ) -> Result<Vec<Notification>, failure::Error> {
        self.session_presenter
            .borrow_mut()
            .modify_band(&Operation::SetEarValue(talker_id, ear_idx, talk_idx, value));

        if !fly {
            return Ok(vec![Notification::TalkerChanged]);
        }
        Ok(vec![])
    }

    pub fn add_talker_ear_talk_value_by_index(
        &mut self,
        talker_id: Id,
        ear_idx: Index,
        value: f32,
    ) -> Result<Vec<Notification>, failure::Error> {
        self.session_presenter
            .borrow_mut()
            .modify_band(&Operation::AddValueToEar(talker_id, ear_idx, value));
        Ok(vec![Notification::TalkerChanged])
    }

    pub fn select_ear_talk(
        &mut self,
        talker_id: Id,
        ear_idx: Index,
        talk_idx: Index,
    ) -> Result<Vec<Notification>, failure::Error> {
        let mut notifications = Vec::new();

        match self.selected_talk {
            Some((prev_tkr_id, prev_ear_idx, prev_talk_idx)) => {
                if talker_id == prev_tkr_id && ear_idx == prev_ear_idx && talk_idx == prev_talk_idx
                {
                    self.selected_talk = None;
                } else {
                    self.selected_talk = Some((talker_id, ear_idx, talk_idx));
                    notifications.push(Notification::EarSelected(talker_id, ear_idx, talk_idx));
                }
                notifications.push(Notification::EarUnselected(
                    prev_tkr_id,
                    prev_ear_idx,
                    prev_talk_idx,
                ));
                notifications.push(Notification::SelectionChanged);
            }
            None => match self.selected_voice {
                None => {
                    self.selected_talk = Some((talker_id, ear_idx, talk_idx));
                    notifications.push(Notification::EarSelected(talker_id, ear_idx, talk_idx));
                    notifications.push(Notification::SelectionChanged);
                }
                Some((voice_tkr_id, voice_port)) => {
                    if voice_tkr_id == talker_id {
                        self.selected_talk = Some((talker_id, ear_idx, talk_idx));
                        notifications.push(Notification::EarSelected(talker_id, ear_idx, talk_idx));
                        notifications.push(Notification::SelectionChanged);
                    } else {
                        self.session_presenter
                            .borrow_mut()
                            .modify_band(&Operation::SetEarVoice(
                                talker_id,
                                ear_idx,
                                talk_idx,
                                voice_tkr_id,
                                voice_port,
                            ));
                        notifications.push(Notification::TalkerChanged);
                    }
                    self.selected_voice = None;
                    notifications.push(Notification::VoiceUnselected(voice_tkr_id, voice_port));
                }
            },
        }
        Ok(notifications)
    }

    pub fn select_voice(
        &mut self,
        talker_id: Id,
        voice_port: Index,
    ) -> Result<Vec<Notification>, failure::Error> {
        let mut notifications = Vec::new();

        match self.selected_voice {
            Some((prev_tkr_id, prev_port)) => {
                if talker_id == prev_tkr_id && voice_port == prev_port {
                    self.selected_voice = None
                } else {
                    self.selected_voice = Some((talker_id, voice_port));
                    notifications.push(Notification::VoiceSelected(talker_id, voice_port));
                }

                notifications.push(Notification::VoiceUnselected(prev_tkr_id, prev_port));
                notifications.push(Notification::SelectionChanged);
            }
            None => match self.selected_talk {
                None => {
                    self.selected_voice = Some((talker_id, voice_port));
                    notifications.push(Notification::VoiceSelected(talker_id, voice_port));
                    notifications.push(Notification::SelectionChanged);
                }
                Some((ear_tkr_id, ear_idx, talk_idx)) => {
                    if talker_id == ear_tkr_id {
                        self.selected_voice = Some((talker_id, voice_port));
                        notifications.push(Notification::VoiceSelected(talker_id, voice_port));
                        notifications.push(Notification::SelectionChanged);
                    } else {
                        self.session_presenter
                            .borrow_mut()
                            .modify_band(&Operation::SetEarVoice(
                                ear_tkr_id, ear_idx, talk_idx, talker_id, voice_port,
                            ));

                        notifications.push(Notification::TalkerChanged);
                    }
                    self.selected_talk = None;
                    notifications.push(Notification::EarUnselected(ear_tkr_id, ear_idx, talk_idx));
                }
            },
        }
        Ok(notifications)
    }

    pub fn show_voice(
        &self,
        talker: &RTalker,
        voice_port: Index,
    ) -> Result<Vec<Notification>, failure::Error> {
        Ok(vec![
            Notification::TalkSelected(talker.borrow().id(), voice_port),
            Notification::SelectionChanged,
        ])
    }

    pub fn add_ear_talk(
        &mut self,
        talker_id: Id,
        ear_idx: Index,
    ) -> Result<Vec<Notification>, failure::Error> {
        match self.selected_voice {
            None => (),
            Some((voice_tkr_id, voice_port)) => {
                self.session_presenter
                    .borrow_mut()
                    .modify_band(&Operation::AddVoiceToEar(
                        talker_id,
                        ear_idx,
                        voice_tkr_id,
                        voice_port,
                    ));

                return Ok(vec![Notification::TalkerChanged]);
            }
        }
        Ok(vec![])
    }

    pub fn sup_ear_talk(
        &self,
        talker_id: Id,
        ear_idx: Index,
        talk_idx: Index,
    ) -> Result<Vec<Notification>, failure::Error> {
        self.session_presenter
            .borrow_mut()
            .modify_band(&Operation::SupEar(talker_id, ear_idx, talk_idx));

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
        notifications.push(Notification::TalkerChanged);
        Ok(notifications)
    }

    pub fn new_talker(&self) -> Option<Id> {
        self.new_talker
    }

    pub fn sup_talker(&self, talker_id: Id) -> Result<Vec<Notification>, failure::Error> {
        self.session_presenter
            .borrow_mut()
            .modify_band(&Operation::SupTalker(talker_id));

        Ok(vec![Notification::TalkerChanged])
    }
}
