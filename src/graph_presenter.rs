use std::cell::RefCell;
use std::collections::HashSet;
use std::collections::HashMap;
use std::rc::Rc;

use talker::identifier::Identifiable;
use talker::identifier::{Id, Index};
use talker::talker::RTalker;

use session::band::{EarHum, Operation};
use session::event_bus::{Notification, REventBus};
use session::mixer;

use crate::session_presenter::RSessionPresenter;

pub struct GraphPresenter {
    selected_hum: Option<(Id, Index, Index, Index)>,
    selected_hum_add_in: Option<(Id, Index, Index, Index)>,
    selected_voice: Option<(Id, Index)>,
    new_talker: Option<Id>,
    selected_talkers: HashSet<Id>,
    selected_data_talker: Option<Id>,
    minimized_talkers: HashSet<Id>,
    all_talkers_minimized: bool,
    multi_selection: bool,
    solo_track: Option<(Id, Index)>,
    mute_tracks: HashMap<Id, HashSet<Index>>,
    session_presenter: RSessionPresenter,
    event_bus: REventBus,
}

pub type RGraphPresenter = Rc<RefCell<GraphPresenter>>;

impl GraphPresenter {
    pub fn new_ref(session_presenter: &RSessionPresenter, event_bus: &REventBus) -> RGraphPresenter {
        let mut mute_tracks = HashMap::new();

        for mxr_id in session_presenter.borrow().mixers().keys() {
            mute_tracks.insert(*mxr_id, HashSet::new());
        }

        Rc::new(RefCell::new(Self {
            selected_hum: None,
            selected_hum_add_in: None,
            selected_voice: None,
            new_talker: None,
            selected_talkers: HashSet::new(),
            selected_data_talker: None,
            minimized_talkers: HashSet::new(),
            all_talkers_minimized: false,
            multi_selection: false,
            solo_track: None,
            mute_tracks,
            session_presenter: session_presenter.clone(),
            event_bus: event_bus.clone(),
        }))
    }

    pub fn init(&mut self) {
        self.selected_hum = None;
        self.selected_hum_add_in = None;
        self.selected_voice = None;
        self.new_talker = None;
        self.selected_talkers.clear();
        self.selected_data_talker = None;
        self.minimized_talkers.clear();
        self.multi_selection = false;
        self.solo_track = None;
        self.mute_tracks.clear();

        for mxr_id in self.session_presenter.borrow().mixers().keys() {
            self.mute_tracks.insert(*mxr_id, HashSet::new());
        }
    }

    pub fn get_talker(&self, talker_id: Id) -> RTalker {
        self.session_presenter.borrow().find_talker(talker_id).unwrap().clone()
    }

    pub fn talker_selected(&self, talker_id: Id) -> bool {
        self.selected_talkers.contains(&talker_id)
    }

    pub fn selected_data_talker(&self) -> Option<Id> {
        self.selected_data_talker
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

    pub fn set_multi_selection(&mut self, multi_selection: bool) {
        self.multi_selection = multi_selection;
    }

    pub fn select_talker(&mut self, talker_id: Id) -> Result<Vec<Notification>, failure::Error> {
        let mut notifications = Vec::new();

        if self.multi_selection {
            if self.selected_talkers.contains(&talker_id) {
                self.selected_talkers.remove(&talker_id);
            } else {
                self.selected_talkers.insert(talker_id);
            }
        } else {
            if self.selected_talkers.len() == 1 && self.selected_talkers.contains(&talker_id) {
                self.selected_talkers.clear();
            }
            else {
                self.selected_talkers.clear();
                self.selected_talkers.insert(talker_id);
            }
        }
        notifications.push(Notification::SelectionChanged);
        Ok(notifications)
    }

    pub fn unselect_talker(&mut self, talker_id: Id) -> Result<Vec<Notification>, failure::Error> {
        let mut notifications = Vec::new();

        if self.selected_talkers.contains(&talker_id) {
            self.selected_talkers.remove(&talker_id);
            notifications.push(Notification::SelectionChanged);
        }
        Ok(notifications)
    }

    pub fn select_data_talker(
        &mut self,
        talker_id: Id,
    ) -> Result<Vec<Notification>, failure::Error> {
        let mut notifications = Vec::new();

        self.selected_talkers.clear();
        self.selected_talkers.insert(talker_id);
        self.selected_data_talker = Some(talker_id);

        notifications.push(Notification::SelectionChanged);
        notifications.push(Notification::EditTalkerData(talker_id));

        Ok(notifications)
    }

    pub fn unselect_data_talker(&mut self) -> Result<Vec<Notification>, failure::Error> {
        match self.selected_data_talker {
            Some(talker_id) => {
                self.selected_data_talker = None;
                self.unselect_talker(talker_id)
            },
            None => Ok(Vec::new()),
        }
    }

    pub fn talker_minimized(&self, talker_id: Id) -> bool {
        self.minimized_talkers.contains(&talker_id)
    }

    pub fn minimize_talker(&mut self, talker_id: Id) -> Result<Vec<Notification>, failure::Error> {
        if self.minimized_talkers.contains(&talker_id) {
            self.minimized_talkers.remove(&talker_id);
        } else {
            self.minimized_talkers.insert(talker_id);
        }
        Ok(vec![Notification::TalkerChanged])
    }

    pub fn toggle_talkers_face(&mut self) {
        if self.all_talkers_minimized {
            self.minimized_talkers.clear();
        }
        else {
            for tkr_id in self.session_presenter.borrow().talkers().keys() {
                self.minimized_talkers.insert(*tkr_id);
            }
        }
        self.all_talkers_minimized = !self.all_talkers_minimized;

        self.session_presenter.borrow().notify(Notification::TalkerChanged);
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

    pub fn set_talker_ear_hum(&mut self, ear_hum: EarHum) {
        self.session_presenter.borrow_mut().modify_band(
            &Operation::SetEarHum(ear_hum));

        self.event_bus.borrow().notify(Notification::TalkerChanged);
    }

    pub fn set_talker_ear_hum_value(
        &mut self,
        talker_id: Id,
        ear_idx: Index,
        set_idx: Index,
        hum_idx: Index,
        value: f32,
    ) {
        self.session_presenter.borrow_mut().modify_band(
            &Operation::SetEarHumValue(
                talker_id, ear_idx, set_idx, hum_idx, value));

        self.event_bus.borrow().notify(Notification::TalkerChanged);
    }

    pub fn set_talker_ear_hum_value_volatly(
        &mut self,
        talker_id: Id,
        ear_idx: Index,
        set_idx: Index,
        hum_idx: Index,
        value: f32,
    ) {
        self.session_presenter.borrow_mut().modify_band_volatly(
            &Operation::SetEarHumValue(
                talker_id, ear_idx, set_idx, hum_idx, value));
    }

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

    pub fn sup_talker(&self, talker_id: Id) -> Result<Vec<Notification>, failure::Error> {
        self.session_presenter
            .borrow_mut()
            .modify_band(&Operation::SupTalker(talker_id));

        Ok(vec![Notification::TalkerChanged])
    }

    pub fn duplicate_selected_talkers(&self) {
        for id in &self.selected_talkers {
            self.session_presenter.borrow_mut().duplicate_talker(*id);
        }
    }

    pub fn has_solo_track(&self) -> bool {
        self.solo_track.is_some()
    }

    pub fn is_solo_track(&self, mixer_id: Id, track_idx: Index) -> bool {
        match self.solo_track {
            Some((mxr_id, trk_idx)) => mxr_id == mixer_id && trk_idx == track_idx,
            None => false,
        }
    }

    pub fn set_solo_track(&mut self, mixer_id: Id, track_idx: Index) -> Result<Vec<Notification>, failure::Error> {

        if let Some((mxr_id, trk_idx)) = self.solo_track {
            if mxr_id == mixer_id && trk_idx == track_idx {
                self.solo_track = None;
            }
            else {
                self.solo_track = Some((mixer_id, track_idx));
            }
        }
        else {
            self.solo_track = Some((mixer_id, track_idx));
        }

        self.set_audible_tracks();

        Ok(vec![Notification::TalkerChanged])
    }

    pub fn is_mute_track(&self, mixer_id: Id, track_idx: Index) -> bool {
        self.mute_tracks.get(&mixer_id).expect("Mixer expected.").contains(&track_idx)
    }

    pub fn set_mute_track(&mut self, mixer_id: Id, track_idx: Index) -> Result<Vec<Notification>, failure::Error> {
        let muteds = self.mute_tracks.get_mut(&mixer_id).expect("Mixer expected.");

        if muteds.contains(&track_idx) {
            muteds.remove(&track_idx);
        }
        else {
            muteds.insert(track_idx);
        }

        self.set_audible_tracks();

        Ok(vec![Notification::TalkerChanged])
    }

    fn set_audible_tracks(&self) {

        if let Some((mxr_id, trk_idx)) = self.solo_track {
            self.session_presenter
            .borrow_mut()
            .set_audible_tracks(mxr_id, vec![trk_idx]);
        }
        else {
            for (mxr_id, muteds) in  &self.mute_tracks {
                let tracks_count = self.session_presenter.borrow().get_mixer_tracks_count(*mxr_id).unwrap();

                let mut audible_tracks = Vec::new();

                for trk_idx in 0..tracks_count {
                    if !muteds.contains(&trk_idx) {
                        audible_tracks.push(trk_idx);
                    }
                }
                self.session_presenter
                .borrow_mut()
                .set_audible_tracks(*mxr_id, audible_tracks);
            }
        }
    }

    pub fn add_mixer_track(&mut self, mixer_id: Id) -> Result<Vec<Notification>, failure::Error> {

        let notifications = self.add_ear_set(mixer_id, mixer::TRACKS_EAR_INDEX)?;

        self.set_audible_tracks();
        
        Ok(notifications)
    }

    pub fn sup_mixer_track(&mut self, mixer_id: Id, track_idx: Index) -> Result<Vec<Notification>, failure::Error> {

        if let Some((mxr_id, trk_idx)) = self.solo_track {
            if mxr_id == mixer_id{
                if trk_idx == track_idx {
                    self.solo_track = None
                }
                else if trk_idx > track_idx {
                    self.solo_track = Some((mxr_id, trk_idx - 1));
                }
            } 
        }

        let muteds = self.mute_tracks.get_mut(&mixer_id).expect("Mixer expected.");

        if !muteds.is_empty() {
            muteds.remove(&track_idx);

            let tracks_count = self.session_presenter.borrow().get_mixer_tracks_count(mixer_id).expect("Mixer expected.");

            for trk_idx in (track_idx + 1)..tracks_count {
                if muteds.contains(&trk_idx) {
                    
                    muteds.remove(&trk_idx);
                    muteds.insert(trk_idx - 1);
                }
            }
        }

        let notifications = self.sup_ear_set(mixer_id, mixer::TRACKS_EAR_INDEX, track_idx)?;

        self.set_audible_tracks();

        Ok(notifications)
    }


    pub fn backup_hum(&self, talker_id: Id, ear_idx: Index, set_idx: Index, hum_idx: Index) -> EarHum {
        self.session_presenter.borrow().session().backup_ear_hum(talker_id, ear_idx, set_idx, hum_idx).expect("Talker ear hum invalid")
    }
}
