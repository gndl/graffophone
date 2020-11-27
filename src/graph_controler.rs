use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

use talker::identifier::Identifiable;
use talker::identifier::{Id, Index};
use talker::talker::{RTalker, Talker, TalkerBase};

use crate::session_presenter::RSessionPresenter;
use session::event_bus::{Notification, REventBus};

pub struct GraphControler {
    selected_ear: Option<(RTalker, i32)>,
    selected_voice: Option<(RTalker, i32)>,
    new_talker: Option<RTalker>,
    selected_talkers: HashSet<Id>,
    control_key_pressed: bool,
    shift_key_pressed: bool,
    alt_key_pressed: bool,
    session_presenter: RSessionPresenter,
}

pub type RGraphControler = Rc<RefCell<GraphControler>>;

impl GraphControler {
    pub fn new(session_presenter: RSessionPresenter) -> GraphControler {
        Self {
            selected_ear: None,
            selected_voice: None,
            new_talker: None,
            selected_talkers: HashSet::new(),
            control_key_pressed: false,
            shift_key_pressed: false,
            alt_key_pressed: false,
            session_presenter,
        }
    }

    pub fn new_ref(session_presenter: RSessionPresenter) -> RGraphControler {
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

    pub fn set_selected_talker(&mut self, tkr: &RTalker) {
        let tkr_id = tkr.borrow().id();

        if self.control_key_pressed || self.selected_talkers.len() < 2 {
            if self.selected_talkers.contains(&tkr_id) {
                self.selected_talkers.remove(&tkr_id);
                self.session_presenter
                    .borrow()
                    .notify(Notification::TalkerUnselected(tkr_id));
            } else {
                self.selected_talkers.insert(tkr_id);
                self.session_presenter
                    .borrow()
                    .notify(Notification::TalkerSelected(tkr_id));
            }
        } else {
            for id in &self.selected_talkers {
                self.session_presenter
                    .borrow()
                    .notify(Notification::TalkerUnselected(*id));
            }
            self.selected_talkers.clear();
            self.selected_talkers.insert(tkr_id);
            self.session_presenter
                .borrow()
                .notify(Notification::TalkerSelected(tkr_id));
        }
    }

    pub fn set_talker_name(&mut self, tkr: &RTalker, v: &str) {
        tkr.borrow_mut().set_name(v);

        self.session_presenter
            .borrow()
            .notify(Notification::TalkerRenamed(tkr.borrow().id()));
    }

    pub fn set_talker_data(&mut self, tkr: &RTalker, v: &str, fly: bool) {
        match tkr.borrow_mut().set_data_from_string(v) {
            Ok(state) => {
                if !fly {
                    self.session_presenter
                        .borrow()
                        .notify(Notification::TalkerChanged);
                }
            }
            Err(e) => self.session_presenter.borrow().notify_error(e),
        }
    }

    pub fn set_talker_ear_value_by_index(
        &mut self,
        tkr: &RTalker,
        ear_idx: usize,
        talk_idx: usize,
        value: f32,
        fly: bool,
    ) {
        match tkr
            .borrow()
            .set_ear_value_by_index(ear_idx, talk_idx, value)
        {
            Ok(state) => {
                if !fly {
                    self.session_presenter
                        .borrow()
                        .notify(Notification::TalkerChanged);
                }
            }
            Err(e) => self.session_presenter.borrow().notify_error(e),
        }
    }

    pub fn add_talker_ear_value_by_index(&mut self, tkr: &RTalker, ear_idx: usize, value: f32) {
        match tkr.borrow().add_ear_value_by_index(ear_idx, value) {
            Ok(state) => {
                self.session_presenter
                    .borrow()
                    .notify(Notification::TalkerChanged);
            }
            Err(e) => self.session_presenter.borrow().notify_error(e),
        }
    }

    /*
    pub fn  selectEar talker index =
      match self.selected_ear with
      | Some (tkr, idx) -> {
          if talker != tkr || index <> idx {
            self.selected_ear = Some (talker, index);
            Bus.notify(Bus.EarSelected (talker#getId, index))
          }
          else {
            self.selected_ear = None
          };
          Bus.notify(Bus.EarUnselected (tkr#getId, idx));
        }
      | None -> {
          match self.selected_voice with
          | None ->
            self.selected_ear = Some (talker, index);
            Bus.notify(Bus.EarSelected (talker#getId, index))
          | Some (tkr, idx) ->
            if tkr != talker {
              let voice = tkr#getVoices.(idx) in
              talker#setEarToVoiceByIndex index voice;
              self.selected_voice = None;
              Bus.notify(Bus.VoiceUnselected (tkr#getId, idx));
              Bus.notify Bus.TalkerChanged;
            }
        }


    pub fn  selectVoice talker index =
      match self.selected_voice with
      | Some (tkr, idx) -> {
          if talker != tkr || index <> idx {
            self.selected_voice = Some (talker, index);
            Bus.notify(Bus.VoiceSelected (talker#getId, index))
          }
          else {
            self.selected_voice = None
          };
          Bus.notify(Bus.VoiceUnselected (tkr#getId, idx));
        }
      | None -> {
          match self.selected_ear with
          | None ->
            self.selected_voice = Some (talker, index);
            Bus.notify(Bus.VoiceSelected (talker#getId, index))
          | Some (tkr, idx) ->
            if tkr != talker {
              let voice = talker#getVoices.(index) in
              tkr#setEarToVoiceByIndex idx voice;
              self.selected_ear = None;
              Bus.notify(Bus.EarUnselected (tkr#getId, idx));
              Bus.notify Bus.TalkerChanged;
            }
        }


    pub fn  showVoice (talker:Tkr.c) port){trace("showVoice tkr "^soi talker#getId^" port "^soi port);
      Bus.notify (Bus.TalkSelected(talker#getId, port));


    pub fn  addEar (talker:Tkr.c) (rootIndex:int) =

      match self.selected_voice with
        None -> ()
      | Some (tkr, idx) ->
        talker#addEarToVoiceByIndex rootIndex (tkr#getVoices.(idx));
        Bus.notify Bus.TalkerChanged


    pub fn  supEar (talker:Tkr.c) (index:int) =
      talker#supEarByIndex index;
      Bus.notify Bus.TalkerChanged;


    pub fn  addNewTalker talker =

      ignore(match self.selected_voice
             with None -> ()
                | Some (tkr, idx) -> Bus.notify(Bus.VoiceUnselected (tkr#getId, idx)));

      ignore(match self.selected_ear
             with None -> ()
                | Some (tkr, idx) -> Bus.notify(Bus.EarUnselected (tkr#getId, idx)));

      ignore(match self.new_talker with
            None -> ()
          | Some tkr ->
            self.selected_voice = Some (tkr, 0);
            Bus.notify(Bus.VoiceSelected (tkr#getId, 0))
        };

      self.new_talker = Some talker;


    pub fn  getNewTalker){self.new_talker

    pub fn  deleteTalker (talker:Tkr.c)){trace("Delete talker "^ talker#getName);
      Session.supTalker talker;
      Bus.notify Bus.TalkerChanged;

                                         */
}
