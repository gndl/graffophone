use std::cell::RefCell;
use std::rc::Rc;
use std::str::FromStr;

use gtk::glib;

use ::session::channel;
use talker::identifier::{Id, Index};
use talker::talker::RTalker;
use talker::Identifier;

use crate::session::band::Operation;
use crate::session::event_bus::{Notification, REventBus};
use crate::session::factory::{Factory, OutputParam};
use crate::session::session::{self, Session};
use crate::session::state::State;

use crate::mixer_presenter::MixerPresenter;
use crate::output_presenter::{self, OutputPresenter};
use crate::util;

const GSR: &str = "
Sinusoidal 2#Sin 1
>0<440
>1<0
>2<1

Mixer 1#Mixer 1
>0<0.1
>1.0.0.0<2:0
>1.0.1.0<1
>1.0.2.0<1
>1.0.3.0<1
";

pub struct SessionPresenter {
    session: Session,
    state: State,
    mixers_presenters: Vec<MixerPresenter>,
    event_bus: REventBus,
}
pub type RSessionPresenter = Rc<RefCell<SessionPresenter>>;

impl SessionPresenter {
    pub fn new_ref(event_bus: &REventBus) -> RSessionPresenter {
        let mut session = Session::new(GSR.to_string()).unwrap();
        let state = session.state();

        Rc::new(RefCell::new(Self {
            session,
            state,
            mixers_presenters: Vec::new(),
            event_bus: event_bus.clone(),
        }))
    }

    pub fn new_session(&mut self) {
        self.exit();
        self.receive_new_session(Session::new(GSR.to_string()));
    }

    pub fn open_session(&mut self, filename: &str) {
        self.exit();
        self.receive_new_session(Session::from_file(filename));
    }

    pub fn save_session(&mut self) {
        self.manage_result(self.session.save(), Some(Notification::SessionSaved));
    }

    pub fn save_session_as(&mut self, filename: &str) {
        let res = self.session.save_as(filename);
        self.manage_result(res, Some(Notification::SessionSavedAs(self.session.filename().to_string())));
    }

    pub fn session(&self) -> &Session {
        &self.session
    }

    fn receive_new_session(&mut self, result: Result<Session, failure::Error>) {
        match result {
            Ok(session) => {
                self.session = session;
                self.notify_new_session();
                self.check_state();
            },
            Err(e) => self.event_bus.borrow().notify_error(e),
        }
    }

    fn notify_new_session(&self) {
        self.event_bus.borrow().notify(Notification::NewSession(
            self.session.filename().to_string(),
        ));
    }

    fn manage_result(&self, result: Result<(), failure::Error>, on_ok: Option<Notification>) {
        match result {
            Ok(()) => {
                if let Some(notification) = on_ok {
                    self.event_bus.borrow().notify(notification)
                }
            }
            Err(e) => self.event_bus.borrow().notify_error(e),
        }
    }

    fn manage_state(&mut self, state: State) {

        if state != self.state {
            self.event_bus.borrow().notify(Notification::State(state));
            self.state = state;
        }
    }

    fn manage_state_result(&mut self, result: Result<State, failure::Error>) -> bool {
        match result {
            Ok(state) => {
                self.manage_state(state);
                true
            },
            Err(e) => {
                self.event_bus.borrow().notify_error(e);
                false
            },
        }
    }

    pub fn check_state(&mut self) -> State {
        let state = self.session.state();
        self.manage_state(state);
        self.state
    }

    pub fn init(&self) {
        let res = session::init();
        self.manage_result(res, None);

        let res = Factory::visit(|factory| {
            Ok(self.event_bus.borrow().notify(Notification::TalkersRange(
                factory.get_categorized_talkers_label_model(),
            )))
        });
        self.manage_result(res, None);
        self.notify_new_session();
    }

    pub fn sample_rate(&self) -> usize {
        self.session.sample_rate()
    }
    pub fn set_sample_rate(&mut self, sample_rate: usize) {
        let res = self.session.set_sample_rate(sample_rate);
        self.manage_state_result(res);
    }

    pub fn find_talker(&self, talker_id: Id) -> Option<&RTalker> {
        self.session.talkers().get(&talker_id)
    }

    pub fn add_talker(&mut self, talker_model: &str) {
        let res = self.session.add_talker(talker_model);
        
        if self.manage_state_result(res) {
            self.event_bus.borrow().notify(Notification::NewTalker);
        }
    }

    pub fn duplicate_talker(&mut self, talker_id: Id) {
        match self.find_talker(talker_id) {
            Some(tkr) => self.add_talker(&tkr.model()),
            None => ()
        }
    }

    
    pub fn set_talker_data(&mut self, talker_id: Id, data: &str) {

        if self.modify_band(&Operation::SetTalkerData(talker_id, data.to_string())) {
            self.event_bus.borrow().notify(Notification::TalkerChanged);
        }
    }

    pub fn find_compatible_hum_with_voice_in_ear(
        &self,
        _: Id,    // talker_id
        _: Index, // ear_idx
        _: Id,    // voice_tkr_id
        _: Index, // voice_port
    ) -> Result<Index, failure::Error> {
        // TODO : do the job
        Ok(0)
    }

    pub fn modify_band(&mut self, operation: &Operation) -> bool {
        let res = self.session.modify_band(operation);
        self.manage_state_result(res)
    }

    pub fn set_start_tick(&mut self, t: i64) {
        let res = self.session.set_start_tick(t);
        self.manage_state_result(res);
    }

    pub fn set_end_tick(&mut self, t: i64) {
        let res = self.session.set_end_tick(t);
        self.manage_state_result(res);
    }

    fn monitor_state(session_presenter_reference: &RSessionPresenter) {
        let this = session_presenter_reference.clone();

        glib::timeout_add_seconds_local(1, move || {
            match this.borrow_mut().check_state() {
                State::Playing | State::Recording => glib::ControlFlow::Continue,
                _ => glib::ControlFlow::Break,
            }
        });
    }

    pub fn play_or_pause(&mut self, monitor: &RSessionPresenter) {

        let (res, monitor_state) = match self.session.state() {
            State::Playing => (self.session.pause(), false),
            _ => (self.session.play(), true),
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

    pub fn record(&mut self, monitor: &RSessionPresenter) {
        let res = self.session.record();
        self.manage_state_result(res);
        SessionPresenter::monitor_state(monitor);
    }

    pub fn exit(&mut self) {
        let res = self.session.exit();
        self.manage_state_result(res);
    }


    // Mixers presenters

    pub fn init_mixers_presenters<'a>(&'a mut self) {
        self.mixers_presenters.clear();

        for mixer in self.session.mixers().values() {
            self.mixers_presenters.push(MixerPresenter::new(mixer));
        }
    }

    pub fn mixers_presenters<'a>(&'a self) -> &'a Vec<MixerPresenter> {
        &self.mixers_presenters
    }

    pub fn visite_mixer<F>(&self, mixer_id: Id, mut f: F) where F: FnMut(&MixerPresenter), {
        for mixer in &self.mixers_presenters {
            if mixer.id() == mixer_id {
                f(mixer);
                break;
            }
        }
    }

    pub fn visite_mutable_mixer<F>(&mut self, mixer_id: Id, mut f: F) where F: FnMut(&mut MixerPresenter), {
        for mixer in self.mixers_presenters.iter_mut() {
            if mixer.id() == mixer_id {
                f(mixer);
                break;
            }
        }
    }

    pub fn visite_mutable_mixer_output<F>(&mut self, mixer_id: Id, output_id: Id, mut f: F) where F: FnMut(&mut OutputPresenter), {
        self.visite_mutable_mixer(mixer_id, |mixer| {
            for output in mixer.mutable_outputs().iter_mut() {
                if output.id() == output_id {
                    f(output);
                    break;
                }
            }
        });
    }

    pub fn set_mixer_output_codec(&mut self, mixer_id: Id, output_id: Id, value_index: usize) {
        let codec_name = output_presenter::CODECS_NAMES[value_index];
        let extention = output_presenter::CODEC_CONTAINERS_EXTENTIONS[value_index];

        self.visite_mutable_mixer_output(mixer_id, output_id, |o| {
            let new_file_path = util::filename_with_extention(&o.file_path(), extention);
            o.set_file_path(new_file_path.as_str());
            o.set_codec_name(codec_name);
        });
    }

    pub fn set_mixer_output_sample_rate(&mut self, mixer_id: Id, output_id: Id, value_index: usize) {
        let sample_rate = usize::from_str(output_presenter::SAMPLE_RATES[value_index]).unwrap();

        self.visite_mutable_mixer_output(mixer_id, output_id, |o| o.set_sample_rate(sample_rate));
    }

    pub fn set_mixer_output_channel_layout(&mut self, mixer_id: Id, output_id: Id, value_index: usize) {
        let channel_layout = channel::Layout::from_index(value_index);

        self.visite_mutable_mixer_output(mixer_id, output_id, |o| o.set_channel_layout(channel_layout));
    }

    pub fn set_mixer_output_file_path(&mut self, mixer_id: Id, output_id: Id, value: &str) {
        self.visite_mutable_mixer_output(mixer_id, output_id, |o| o.set_file_path(value));
    }

    pub fn default_audiofile_name(&self) -> String {
        util::filename_with_extention(self.session.filename(), output_presenter::DEFAULT_AUDIO_FILE_EXTENTION)
    }

    pub fn add_mixer_file_output(&mut self, mixer_id: Id) {
        let filepath = self.default_audiofile_name();

        self.visite_mutable_mixer(mixer_id, |mixer| {
            let output = OutputPresenter::new(Identifier::new("", "file"), 
                output_presenter::DEFAULT_CODEC,
                output_presenter::DEFAULT_SAMPLE_RATE,
                channel::DEFAULT_LAYOUT,
                filepath.as_str());
            mixer.add_output(output);
        });
    }

    pub fn remove_mixer_output(&mut self, mixer_id: Id, output_id: Id) {
        self.visite_mutable_mixer(mixer_id, |mixer| mixer.remove_output(output_id));
    }

    pub fn ratify_mixers_outputs(&mut self) {
        let mut operations = Vec::new();

        for mixer_presenter in &self.mixers_presenters {

            let mut outputs_params = Vec::new();

            for output in mixer_presenter.outputs() {
                let output_params = OutputParam::File(
                    output.codec_name().to_string(),
                    output.sample_rate(),
                    output.channel_layout().to_string(),
                    output.file_path().to_string());

                outputs_params.push(output_params);
            }
            operations.push(Operation::SetMixerOutputs(mixer_presenter.id(), outputs_params));
        }

        let mut operations_ok = true;

        for operation in &operations {
            operations_ok &= self.modify_band(operation);
        }

        if operations_ok {
            self.event_bus.borrow().notify(Notification::TalkerChanged);
        }
    }

    pub fn cancel_mixers_presenters(&mut self) {
        self.mixers_presenters.clear();
    }
}
