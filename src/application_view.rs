use std::cell::RefCell;
use std::rc::Rc;

use gtk::prelude::*;
use gtk::FileDialog;
use gtk::{glib, Widget};

use talker::data::Data;
use talker::identifier::Id;

use session;
use session::event_bus::{Notification, REventBus};
use session::state::State;
use sourceview5::traits::BufferExt;

use crate::graph_view::{GraphView, RGraphView};
use crate::session_presenter::RSessionPresenter;
use crate::ui::talker_object::TalkerObject;

pub struct ApplicationView {
    window: gtk::ApplicationWindow,
    play_or_pause_button: gtk::Button,
    play_or_pause_icon: gtk::Image,
    stop_button: gtk::Button,
    talkers_store: gio::ListStore,
    text_view: sourceview5::View,
    apply_text_button: gtk::Button,
    validate_text_button: gtk::Button,
    cancel_text_button: gtk::Button,
    graph_view: RGraphView,
    session_presenter: RSessionPresenter,
    selected_talker_id: Option<Id>,
}

pub type RApplicationView = Rc<RefCell<ApplicationView>>;

impl ApplicationView {
    fn new(
        application: &gtk::Application,
        session_presenter: &RSessionPresenter,
    ) -> Result<ApplicationView, failure::Error> {
        // header bar
        let headerbar = gtk::HeaderBar::new();

        // header bar left controls

        let new_session_button = gtk::Button::from_icon_name("document-new");
        headerbar.pack_start(&new_session_button);

        let open_session_button = gtk::Button::from_icon_name("document-open");
        headerbar.pack_start(&open_session_button);

        let save_session_button = gtk::Button::from_icon_name("document-save");
        headerbar.pack_start(&save_session_button);

        let save_session_as_button = gtk::Button::from_icon_name("document-save-as");
        headerbar.pack_start(&save_session_as_button);

        let separator = gtk::Separator::new(gtk::Orientation::Vertical);
        headerbar.pack_start(&separator);

        let talkers_list_toggle = gtk::ToggleButton::builder()
            .icon_name("view-list-tree")
            .active(true)
            .build();

        headerbar.pack_start(&talkers_list_toggle);

        let separator = gtk::Separator::new(gtk::Orientation::Vertical);
        headerbar.pack_start(&separator);

        // Apply text
        let apply_text_button = gtk::Button::from_icon_name("go-up");
        headerbar.pack_start(&apply_text_button);

        // Validate text
        let validate_text_button = gtk::Button::from_icon_name("dialog-ok");
        headerbar.pack_start(&validate_text_button);

        // Cancel text
        let cancel_text_button = gtk::Button::from_icon_name("dialog-cancel");
        headerbar.pack_start(&cancel_text_button);

        // header bar right controls
        let stop_button = gtk::Button::from_icon_name("media-playback-stop");

        headerbar.pack_end(&stop_button);

        let play_or_pause_icon = gtk::Image::from_icon_name("media-playback-start");
        let play_or_pause_button = gtk::Button::builder().child(&play_or_pause_icon).build();

        headerbar.pack_end(&play_or_pause_button);

        // Split pane
        let split_pane = gtk::Box::new(gtk::Orientation::Horizontal, 2);

        // Talkers list
        let talkers_store = gio::ListStore::new(TalkerObject::static_type());

        let talkers_item_factory = gtk::SignalListItemFactory::new();
        // talkers lit item factory setup
        talkers_item_factory.connect_setup(move |_, list_item| {
            // Create label
            let label = gtk::Label::new(None);
            let list_item = list_item
                .downcast_ref::<gtk::ListItem>()
                .expect("Needs to be ListItem");

            list_item.set_child(Some(&label));

            list_item
                .property_expression("item")
                .chain_property::<TalkerObject>("label")
                .bind(&label, "label", Widget::NONE);
        });

        let talkers_selection_model = gtk::SingleSelection::new(Some(talkers_store.clone()));
        let talkers_list = gtk::ListView::builder()
            .model(&talkers_selection_model)
            .factory(&talkers_item_factory)
            .single_click_activate(true)
            .build();

        let talkers_list_scrolledwindow = gtk::ScrolledWindow::builder()
            .min_content_width(256)
            .vexpand(true)
            .child(&talkers_list)
            .visible(false)
            .build();
        split_pane.append(&talkers_list_scrolledwindow);
        talkers_list_scrolledwindow.set_visible(true);

        // Graph view
        let graph_view = GraphView::new_ref(&session_presenter);

        let graph_view_scrolledwindow = gtk::ScrolledWindow::builder()
            .hexpand(true)
            .vexpand(true)
            .child(graph_view.borrow().area())
            .build();
        split_pane.append(&graph_view_scrolledwindow);

        // Text view
        let text_view = sourceview5::View::builder()
            .wrap_mode(gtk::WrapMode::Word)
            .vscroll_policy(gtk::ScrollablePolicy::Minimum)
            .highlight_current_line(true)
            .build();

        // Vertical box
        let v_box = gtk::Box::new(gtk::Orientation::Vertical, 2);
        v_box.append(&text_view);
        v_box.append(&split_pane);

        // ApplicationWindow
        let window = gtk::ApplicationWindow::builder()
            .application(application)
            .titlebar(&headerbar)
            .default_width(1024)
            .default_height(768)
            .child(&v_box)
            .visible(true)
            .build();

        /*
        Actions
        */
        // New session
        let new_ctrl = session_presenter.clone();
        new_session_button.connect_clicked(move |_| {
            new_ctrl.borrow_mut().new_session();
        });

        // Open session
        let open_session_ctrl = session_presenter.clone();

        open_session_button.connect_clicked(glib::clone!(@weak window, @weak open_session_ctrl => move |_| {
            let dialog = FileDialog::builder()
                .title("Choose a Graffophone session record file")
                .accept_label("Open")
                .build();

            dialog.open(Some(&window), gio::Cancellable::NONE, move |file| {
                if let Ok(file) = file {
                    let path_buf = file.path().expect("Couldn't get file path");
                            open_session_ctrl.borrow_mut().open_session(&path_buf.to_string_lossy());
                }
            });
        }));

        // Save session
        let save_session_ctrl = session_presenter.clone();
        save_session_button.connect_clicked(move |_| {
            save_session_ctrl.borrow_mut().save_session();
        });

        // Save session as
        let save_session_as_ctrl = session_presenter.clone();
        save_session_as_button.connect_clicked(glib::clone!(@weak window =>move |_| {
            let dialog = FileDialog::builder().title("Choose a Graffophone session record file")
                .accept_label("Open").initial_name(session::session::NEW_SESSION_FILENAME)
                .build();

            let save_session_as_ctrl_ctrl = save_session_as_ctrl.clone();
            dialog.save(Some(&window), gio::Cancellable::NONE, move |file| {
                if let Ok(file) = file {
                    let path_buf = file.path().expect("Couldn't get file path");
                            save_session_as_ctrl_ctrl
                                .borrow_mut()
                                .save_session_as(&path_buf.to_string_lossy());
                }
            });
        }));

        // talkers tree toggle
        talkers_list_toggle.connect_toggled(move |tb| {
            if tb.is_active() {
                talkers_list_scrolledwindow.set_visible(true);
            } else {
                talkers_list_scrolledwindow.set_visible(false);
            }
        });

        // Play
        let play_or_pause_ctrl = session_presenter.clone();
        play_or_pause_button.connect_clicked(move |_| {
            play_or_pause_ctrl
                .borrow_mut()
                .play_or_pause(&play_or_pause_ctrl);
        });

        // Stop
        let stop_ctrl = session_presenter.clone();
        stop_button.connect_clicked(move |_| {
            stop_ctrl.borrow_mut().stop();
        });

        // talkers list selection
        let session_ctrl = session_presenter.clone();

        talkers_list.connect_activate(move |list_view, position| {
            // Get `TalkerObject` from model
            let model = list_view.model().expect("The model has to exist.");
            let talker_object = model
                .item(position)
                .and_downcast::<TalkerObject>()
                .expect("The item has to be an `TalkerObject`.");

            let talker_model = talker_object.model();
            session_ctrl.borrow_mut().add_talker(talker_model.as_str());
        });

        window.present();

        Ok(Self {
            window,
            play_or_pause_button,
            play_or_pause_icon,
            stop_button,
            talkers_store,
            text_view,
            apply_text_button,
            validate_text_button,
            cancel_text_button,
            graph_view,
            session_presenter: session_presenter.clone(),
            selected_talker_id: None,
        })
    }

    pub fn new_ref(
        application: &gtk::Application,
        session_presenter: &RSessionPresenter,
    ) -> Result<RApplicationView, failure::Error> {
        let av = ApplicationView::new(application, session_presenter)?;
        let rav = Rc::new(RefCell::new(av));
        ApplicationView::create_text_editor(&rav);
        ApplicationView::observe(&rav, session_presenter.borrow().event_bus());
        Ok(rav)
    }

    fn fill_talkers_list(
        &self,
        categorized_talkers_label_model: &Vec<(String, Vec<(String, String)>)>,
    ) {
        //        let mut talkers_store = self.talkers_list.model().unwrap().model().unwrap();
        //   let talkers_store = self.talkers_store;

        for (category, talkers) in categorized_talkers_label_model {
            self.talkers_store.append(&TalkerObject::new(category, ""));

            for (label, model) in talkers {
                self.talkers_store.append(&TalkerObject::new(label, model));
            }
        }
    }

    fn unselect_talker(&self) {
        if let Some(talker_id) = self.selected_talker_id {
            let res = self.graph_view.borrow().unselect_talker(talker_id);
            self.session_presenter
                .borrow()
                .notify_notifications_result(res);
        }
    }
    fn create_text_editor(application_view: &RApplicationView) {
        // Apply text
        let apply_text_view = application_view.clone();
        application_view
            .borrow()
            .apply_text_button
            .connect_clicked(move |_| {
                apply_text_view.borrow().set_talker_data();
            });

        // Validate text
        let validate_text_view = application_view.clone();
        application_view
            .borrow()
            .validate_text_button
            .connect_clicked(move |_| {
                validate_text_view.borrow().set_talker_data();
                validate_text_view.borrow().unselect_talker();
                validate_text_view.borrow_mut().disable_text_editor();
            });

        // Cancel text
        let cancel_text_view = application_view.clone();
        application_view
            .borrow()
            .cancel_text_button
            .connect_clicked(move |_| {
                cancel_text_view.borrow().unselect_talker();
                cancel_text_view.borrow_mut().disable_text_editor();
            });
        application_view.borrow_mut().disable_text_editor();
    }

    fn edit_talker_data(&mut self, talker_id: Id) {
        if let Some(talker) = self.session_presenter.borrow().find_talker(talker_id) {
            match &*talker.data().borrow() {
                Data::Int(_) => println!("Todo : Applicationview.edit_talker_data Data::Int"),
                Data::Float(_) => println!("Todo : Applicationview.edit_talker_data Data::Float"),
                Data::String(_) => println!("Todo : Applicationview.edit_talker_data Data::String"),
                Data::Text(data) => {
                    let text_buffer = sourceview5::Buffer::builder()
                        .enable_undo(true)
                        .highlight_matching_brackets(true)
                        .highlight_syntax(true)
                        .build();

                    if let Some(scheme) =
                        sourceview5::StyleSchemeManager::default().scheme("classic-dark")
                    {
                        text_buffer.set_style_scheme(Some(&scheme));
                    }
                    text_buffer.set_text(&data);
                    self.text_view.set_buffer(Some(&text_buffer));
                    self.text_view.set_visible(true);
                    self.apply_text_button.set_visible(true);
                    self.validate_text_button.set_visible(true);
                    self.cancel_text_button.set_visible(true);
                    self.selected_talker_id = Some(talker_id);
                }
                Data::File(_) => println!("Todo : Applicationview.edit_talker_data Data::File"),
                _ => (),
            }
        }
    }

    fn set_talker_data(&self) {
        if let Some(talker_id) = self.selected_talker_id {
            let text_buffer = self.text_view.buffer();
            self.session_presenter.borrow_mut().set_talker_data(
                talker_id,
                &text_buffer.text(&text_buffer.start_iter(), &text_buffer.end_iter(), false),
            );
        }
    }

    fn disable_text_editor(&mut self) {
        //        self.text_view.set_buffer(None::<&impl IsA<TextBuffer>>);
        self.text_view.set_visible(false);
        self.apply_text_button.set_visible(false);
        self.validate_text_button.set_visible(false);
        self.cancel_text_button.set_visible(false);
        self.selected_talker_id = None;
    }

    fn observe(observer: &RApplicationView, bus: &REventBus) {
        let obs = observer.clone();

        bus.borrow_mut()
            .add_observer(Box::new(move |notification| match notification {
                Notification::State(state) => match state {
                    State::Playing => {
                        obs.borrow()
                            .play_or_pause_icon
                            .set_from_icon_name(Some("media-playback-pause"));
                        obs.borrow().stop_button.set_sensitive(true);
                    }
                    State::Paused => {
                        obs.borrow()
                            .play_or_pause_icon
                            .set_from_icon_name(Some("media-playback-start"));
                        obs.borrow().stop_button.set_sensitive(true);
                    }
                    State::Stopped => {
                        obs.borrow()
                            .play_or_pause_icon
                            .set_from_icon_name(Some("media-playback-start"));
                        obs.borrow().play_or_pause_button.set_sensitive(true);
                        obs.borrow().stop_button.set_sensitive(false);
                    }
                    State::Exited => {
                        obs.borrow()
                            .play_or_pause_icon
                            .set_from_icon_name(Some("media-playback-start"));
                        obs.borrow().play_or_pause_button.set_sensitive(false);
                        obs.borrow().stop_button.set_sensitive(false);
                    }
                },
                Notification::NewSession(name) => {
                    obs.borrow_mut().disable_text_editor();
                    obs.borrow().window.set_title(Some(&name));
                }
                Notification::NewSessionName(name) => {
                    obs.borrow().window.set_title(Some(&name));
                }
                Notification::Tick(tick) => println!("Todo : Applicationview.set_tick {}", tick),
                Notification::TimeRange(st, et) => {
                    println!("Todo : Applicationview.set_time_range {} <-> {}", st, et)
                }
                Notification::TalkersRange(talkers) => obs.borrow().fill_talkers_list(&talkers),
                Notification::EditTalkerData(talker_id) => {
                    obs.borrow_mut().edit_talker_data(*talker_id)
                }
                Notification::CurveAdded => println!("Todo : Applicationview.CurveAdded"),
                Notification::CurveRemoved => println!("Todo : Applicationview.CurveRemoved"),
                Notification::Info(msg) => println!("Todo : Applicationview.display_info {}", msg),
                Notification::Warning(msg) => {
                    println!("Todo : Applicationview.display_warning {}", msg)
                }
                Notification::Error(msg) => {
                    println!("Todo : Applicationview.display_error {}", msg)
                }
                _ => (),
            }))
    }
}
