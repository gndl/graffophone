use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

use talker::identifier::Identifiable;
use talker::identifier::{Id, Index};
use talker::talker::RTalker;

use session::band::Operation;
use session::event_bus::Notification;

use crate::session_presenter::RSessionPresenter;

pub struct GraphPresenter {
    selected_hum: Option<(Id, Index, Index, Index)>,
    selected_hum_add_in: Option<(Id, Index, Index, Index)>,
    selected_voice: Option<(Id, Index)>,
    new_talker: Option<Id>,
    selected_talkers: HashSet<Id>,
    minimized_talkers: HashSet<Id>,
    control_key_pressed: bool,
    shift_key_pressed: bool,
    alt_key_pressed: bool,
    session_presenter: RSessionPresenter,
}

pub type RGraphPresenter = Rc<RefCell<GraphPresenter>>;

impl GraphPresenter {
    pub fn new(session_presenter: &RSessionPresenter) -> GraphPresenter {
        Self {
            selected_hum: None,
            selected_hum_add_in: None,
            selected_voice: None,
            new_talker: None,
            selected_talkers: HashSet::new(),
            minimized_talkers: HashSet::new(),
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

    pub fn talker_minimized(&self, talker_id: Id) -> bool {
        self.minimized_talkers.contains(&talker_id)
    }

    pub fn voice_selected(&self, talker_id: Id, voice_idx: Index) -> bool {
        match self.selected_voice {
            None => false,
            Some((tkr_id, vc_idx)) => talker_id == tkr_id && voice_idx == vc_idx,
        }
    }

    pub fn ear_hum_selected(
        &self,
        talker_id: Id,
        ear_idx: Index,
        set_idx: Index,
        hum_idx: Index,
    ) -> bool {
        match self.selected_hum {
            None => false,
            Some((tkr_id, e_idx, s_idx, h_idx)) => {
                talker_id == tkr_id && ear_idx == e_idx && set_idx == s_idx && hum_idx == h_idx
            }
        }
    }

    pub fn ear_hum_add_in_selected(
        &self,
        talker_id: Id,
        ear_idx: Index,
        set_idx: Index,
        hum_idx: Index,
    ) -> bool {
        match self.selected_hum_add_in {
            None => false,
            Some((tkr_id, e_idx, s_idx, h_idx)) => {
                talker_id == tkr_id && ear_idx == e_idx && set_idx == s_idx && hum_idx == h_idx
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

    pub fn minimize_talker(&mut self, talker_id: Id) -> Result<Vec<Notification>, failure::Error> {
        if self.minimized_talkers.contains(&talker_id) {
            self.minimized_talkers.remove(&talker_id);
        } else {
            self.minimized_talkers.insert(talker_id);
        }
        Ok(vec![Notification::TalkerChanged])
    }

    pub fn set_talker_name(
        &mut self,
        talker: &RTalker,
        v: &str,
    ) -> Result<Vec<Notification>, failure::Error> {
        talker.set_name(v);
        Ok(vec![
            Notification::TalkerRenamed(talker.id()),
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

    pub fn set_talker_ear_talk_value(
        &mut self,
        talker_id: Id,
        ear_idx: Index,
        set_idx: Index,
        hum_idx: Index,
        talk_idx: Index,
        value: f32,
        fly: bool,
    ) -> Result<Vec<Notification>, failure::Error> {
        self.session_presenter
            .borrow_mut()
            .modify_band(&Operation::SetEarTalkValue(
                talker_id, ear_idx, set_idx, hum_idx, talk_idx, value,
            ));

        if !fly {
            return Ok(vec![Notification::TalkerChanged]);
        }
        Ok(vec![])
    }

    pub fn notify_talker_changed(&self) {
        self.session_presenter
            .borrow()
            .notify(Notification::TalkerChanged);
    }
    /*
        pub fn add_talker_ear_set_value(
            &mut self,
            talker_id: Id,
            ear_idx: Index,
            value: f32,
        ) -> Result<Vec<Notification>, failure::Error> {
            self.session_presenter
                .borrow_mut()
                .modify_band(&Operation::AddSetValueToEar(talker_id, ear_idx, value));
            Ok(vec![Notification::TalkerChanged])
        }
    */
    pub fn select_ear_hum(
        &mut self,
        talker_id: Id,
        ear_idx: Index,
        set_idx: Index,
        hum_idx: Index,
    ) -> Result<Vec<Notification>, failure::Error> {
        let mut notifications = Vec::new();
        let mut selection_changer = false;

        if let Some((prev_tkr_id, prev_ear_idx, prev_set_idx, prev_hum_idx)) = self.selected_hum {
            if talker_id == prev_tkr_id
                && ear_idx == prev_ear_idx
                && set_idx == prev_set_idx
                && hum_idx == prev_hum_idx
            {
                self.selected_hum = None;
            } else {
                self.selected_hum = Some((talker_id, ear_idx, set_idx, hum_idx));
                notifications.push(Notification::EarSelected(
                    talker_id, ear_idx, set_idx, hum_idx,
                ));
            }
            notifications.push(Notification::EarUnselected(
                prev_tkr_id,
                prev_ear_idx,
                prev_set_idx,
                prev_hum_idx,
            ));
            selection_changer = true;
        } else {
            if let Some((voice_tkr_id, voice_port)) = self.selected_voice {
                if voice_tkr_id == talker_id {
                    self.selected_hum = Some((talker_id, ear_idx, set_idx, hum_idx));
                    notifications.push(Notification::EarSelected(
                        talker_id, ear_idx, set_idx, hum_idx,
                    ));
                    selection_changer = true;
                } else {
                    self.session_presenter
                        .borrow_mut()
                        .modify_band(&Operation::SetEarHumVoice(
                            talker_id,
                            ear_idx,
                            set_idx,
                            hum_idx,
                            voice_tkr_id,
                            voice_port,
                        ));
                    notifications.push(Notification::TalkerChanged);
                }
                self.selected_voice = None;
                notifications.push(Notification::VoiceUnselected(voice_tkr_id, voice_port));
            } else {
                self.selected_hum = Some((talker_id, ear_idx, set_idx, hum_idx));
                notifications.push(Notification::EarSelected(
                    talker_id, ear_idx, set_idx, hum_idx,
                ));
                selection_changer = true;
            }
            if let Some((prev_tkr_id, prev_ear_idx, prev_set_idx, prev_hum_idx)) =
                self.selected_hum_add_in
            {
                self.selected_hum_add_in = None;
                notifications.push(Notification::EarAddInUnselected(
                    prev_tkr_id,
                    prev_ear_idx,
                    prev_set_idx,
                    prev_hum_idx,
                ));
                selection_changer = true;
            }
        }
        if selection_changer {
            notifications.push(Notification::SelectionChanged);
        }
        Ok(notifications)
    }

    pub fn select_ear_hum_add_in(
        &mut self,
        talker_id: Id,
        ear_idx: Index,
        set_idx: Index,
        hum_idx: Index,
    ) -> Result<Vec<Notification>, failure::Error> {
        let mut notifications = Vec::new();
        let mut selection_changer = false;

        if let Some((prev_tkr_id, prev_ear_idx, prev_set_idx, prev_hum_idx)) =
            self.selected_hum_add_in
        {
            if talker_id == prev_tkr_id
                && ear_idx == prev_ear_idx
                && set_idx == prev_set_idx
                && hum_idx == prev_hum_idx
            {
                self.selected_hum_add_in = None;
            } else {
                self.selected_hum_add_in = Some((talker_id, ear_idx, set_idx, hum_idx));
                notifications.push(Notification::EarAddInSelected(
                    talker_id, ear_idx, set_idx, hum_idx,
                ));
            }
            notifications.push(Notification::EarAddInUnselected(
                prev_tkr_id,
                prev_ear_idx,
                prev_set_idx,
                prev_hum_idx,
            ));
            selection_changer = true;
        } else {
            if let Some((voice_tkr_id, voice_port)) = self.selected_voice {
                if voice_tkr_id == talker_id {
                    self.selected_hum_add_in = Some((talker_id, ear_idx, set_idx, hum_idx));
                    notifications.push(Notification::EarAddInSelected(
                        talker_id, ear_idx, set_idx, hum_idx,
                    ));
                    selection_changer = true;
                } else {
                    self.session_presenter
                        .borrow_mut()
                        .modify_band(&Operation::AddVoiceToEarHum(
                            talker_id,
                            ear_idx,
                            set_idx,
                            hum_idx,
                            voice_tkr_id,
                            voice_port,
                        ));
                    notifications.push(Notification::TalkerChanged);
                }
                self.selected_voice = None;
                notifications.push(Notification::VoiceUnselected(voice_tkr_id, voice_port));
            } else {
                self.selected_hum_add_in = Some((talker_id, ear_idx, set_idx, hum_idx));
                notifications.push(Notification::EarAddInSelected(
                    talker_id, ear_idx, set_idx, hum_idx,
                ));
                selection_changer = true;
            }
            if let Some((prev_tkr_id, prev_ear_idx, prev_set_idx, prev_hum_idx)) = self.selected_hum
            {
                self.selected_hum = None;
                notifications.push(Notification::EarUnselected(
                    prev_tkr_id,
                    prev_ear_idx,
                    prev_set_idx,
                    prev_hum_idx,
                ));
                selection_changer = true;
            }
        }
        if selection_changer {
            notifications.push(Notification::SelectionChanged);
        }
        Ok(notifications)
    }

    pub fn select_voice(
        &mut self,
        talker_id: Id,
        voice_port: Index,
    ) -> Result<Vec<Notification>, failure::Error> {
        let mut notifications = Vec::new();

        if let Some((prev_tkr_id, prev_port)) = self.selected_voice {
            if talker_id == prev_tkr_id && voice_port == prev_port {
                self.selected_voice = None
            } else {
                self.selected_voice = Some((talker_id, voice_port));
                notifications.push(Notification::VoiceSelected(talker_id, voice_port));
            }

            notifications.push(Notification::VoiceUnselected(prev_tkr_id, prev_port));
            notifications.push(Notification::SelectionChanged);
        } else {
            if let Some((ear_tkr_id, ear_idx, set_idx, hum_idx)) = self.selected_hum {
                if talker_id == ear_tkr_id {
                    self.selected_voice = Some((talker_id, voice_port));
                    notifications.push(Notification::VoiceSelected(talker_id, voice_port));
                    notifications.push(Notification::SelectionChanged);
                } else {
                    self.session_presenter
                        .borrow_mut()
                        .modify_band(&Operation::SetEarHumVoice(
                            ear_tkr_id, ear_idx, set_idx, hum_idx, talker_id, voice_port,
                        ));

                    notifications.push(Notification::TalkerChanged);
                }
                self.selected_hum = None;
                notifications.push(Notification::EarUnselected(
                    ear_tkr_id, ear_idx, set_idx, hum_idx,
                ));
            } else {
                self.selected_voice = Some((talker_id, voice_port));
                notifications.push(Notification::VoiceSelected(talker_id, voice_port));
                notifications.push(Notification::SelectionChanged);
            }
            if let Some((ear_tkr_id, ear_idx, set_idx, hum_idx)) = self.selected_hum_add_in {
                if talker_id == ear_tkr_id {
                    self.selected_voice = Some((talker_id, voice_port));
                    notifications.push(Notification::VoiceSelected(talker_id, voice_port));
                    notifications.push(Notification::SelectionChanged);
                } else {
                    self.session_presenter
                        .borrow_mut()
                        .modify_band(&Operation::AddVoiceToEarHum(
                            ear_tkr_id, ear_idx, set_idx, hum_idx, talker_id, voice_port,
                        ));

                    notifications.push(Notification::TalkerChanged);
                }
                self.selected_hum_add_in = None;
                notifications.push(Notification::EarUnselected(
                    ear_tkr_id, ear_idx, set_idx, hum_idx,
                ));
            } else {
                self.selected_voice = Some((talker_id, voice_port));
                notifications.push(Notification::VoiceSelected(talker_id, voice_port));
                notifications.push(Notification::SelectionChanged);
            }
        }
        Ok(notifications)
    }

    pub fn show_voice(
        &self,
        talker: &RTalker,
        voice_port: Index,
    ) -> Result<Vec<Notification>, failure::Error> {
        Ok(vec![
            Notification::TalkSelected(talker.id(), voice_port),
            Notification::SelectionChanged,
        ])
    }

    pub fn add_ear_talk(
        &mut self,
        talker_id: Id,
        ear_idx: Index,
        set_idx: Index,
        hum_idx: Index,
    ) -> Result<Vec<Notification>, failure::Error> {
        match self.selected_voice {
            None => (),
            Some((voice_tkr_id, voice_port)) => {
                self.session_presenter
                    .borrow_mut()
                    .modify_band(&Operation::AddVoiceToEarHum(
                        talker_id,
                        ear_idx,
                        set_idx,
                        hum_idx,
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
        set_idx: Index,
        hum_idx: Index,
        talk_idx: Index,
    ) -> Result<Vec<Notification>, failure::Error> {
        self.session_presenter
            .borrow_mut()
            .modify_band(&Operation::SupEarTalk(
                talker_id, ear_idx, set_idx, hum_idx, talk_idx,
            ));

        Ok(vec![Notification::TalkerChanged])
    }

    pub fn add_ear_set(
        &mut self,
        talker_id: Id,
        ear_idx: Index,
    ) -> Result<Vec<Notification>, failure::Error> {
        if let Some((voice_tkr_id, voice_port)) = self.selected_voice {
            let hum_idx = self
                .session_presenter
                .borrow()
                .find_compatible_hum_with_voice_in_ear(
                    talker_id,
                    ear_idx,
                    voice_tkr_id,
                    voice_port,
                )?;

            self.session_presenter
                .borrow_mut()
                .modify_band(&Operation::AddSetVoiceToEar(
                    talker_id,
                    ear_idx,
                    hum_idx,
                    voice_tkr_id,
                    voice_port,
                ));
        } else {
            self.session_presenter
                .borrow_mut()
                .modify_band(&Operation::AddSetValueToEar(talker_id, ear_idx, 0, 0.));
        }
        Ok(vec![Notification::TalkerChanged])
    }

    pub fn sup_ear_set(
        &self,
        talker_id: Id,
        ear_idx: Index,
        set_idx: Index,
    ) -> Result<Vec<Notification>, failure::Error> {
        self.session_presenter
            .borrow_mut()
            .modify_band(&Operation::SupEarSet(talker_id, ear_idx, set_idx));

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
        match self.selected_hum {
            None => (),
            Some((talk_tkr_id, ear_idx, set_idx, hum_idx)) => notifications.push(
                Notification::EarUnselected(talk_tkr_id, ear_idx, set_idx, hum_idx),
            ),
        }

        match self.new_talker {
            None => (),
            Some(tkr_id) => {
                self.selected_voice = Some((tkr_id, 0));
                notifications.push(Notification::VoiceSelected(tkr_id, 0));
            }
        }

        self.new_talker = Some(talker.id());
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
