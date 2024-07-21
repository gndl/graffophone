use std::cell::RefCell;
use std::rc::Rc;

use gtk::prelude::*;
use gtk::Widget;
use gtk::gio::Cancellable;

use talker::data::Data;
use talker::identifier::Id;

use session;
use session::event_bus::{Notification, REventBus};
use session::state::State;
use sourceview5::traits::BufferExt;

use crate::graph_view::{GraphView, RGraphView};
use crate::session_actions;
use crate::session_presenter::RSessionPresenter;
use crate::settings;
use crate::ui::talker_object::TalkerObject;

pub struct ApplicationView {
    window: gtk::ApplicationWindow,
    play_or_pause_button: gtk::Button,
    play_or_pause_icon: gtk::Image,
    stop_button: gtk::Button,
    record_button: gtk::Button,
    message_view_revealer: gtk::Revealer,
    message_view_label: gtk::Label,
    talker_data_view: sourceview5::View,
    talker_data_view_scrolledwindow: gtk::ScrolledWindow,
    push_talker_data_button: gtk::Button,
    commit_talker_data_button: gtk::Button,
    cancel_talker_data_button: gtk::Button,
    talkers_box: gtk::Box,
    graph_view: RGraphView,
    session_presenter: RSessionPresenter,
    event_bus: REventBus,
}

pub type RApplicationView = Rc<RefCell<ApplicationView>>;

impl ApplicationView {
    pub fn new_ref(
        application: &gtk::Application,
        session_presenter: &RSessionPresenter,
        event_bus: &REventBus,
    ) -> Result<RApplicationView, failure::Error> {
        // header bar
        let headerbar = gtk::HeaderBar::new();

        // header bar left controls

        let new_session_button = gtk::Button::builder()
            .icon_name("document-new")
            .action_name("session.new")
            .tooltip_text(format!("New session ({})", session_actions::NEW_SESSION_ACCEL))
            .build();
        headerbar.pack_start(&new_session_button);

        let open_session_button = gtk::Button::builder()
            .icon_name("document-open")
            .action_name("session.open")
            .tooltip_text(format!("Open session ({})", session_actions::OPEN_SESSION_ACCEL))
            .build();
        headerbar.pack_start(&open_session_button);

        let save_session_button = gtk::Button::builder()
            .icon_name("document-save")
            .action_name("session.save")
            .tooltip_text(format!("Save session ({})", session_actions::SAVE_SESSION_ACCEL))
            .build();
        headerbar.pack_start(&save_session_button);

        let save_session_as_button = gtk::Button::builder()
            .icon_name("document-save-as")
            .action_name("session.save_as")
            .tooltip_text(format!("Save session as ({})", session_actions::SAVE_SESSION_AS_ACCEL))
            .build();
        headerbar.pack_start(&save_session_as_button);

        let left_separator = gtk::Separator::new(gtk::Orientation::Vertical);
        headerbar.pack_start(&left_separator);

        let talkers_list_toggle = gtk::ToggleButton::builder()
            .icon_name("view-list-tree")
            .active(true)
            .tooltip_text("Plugins list")
            .build();

        headerbar.pack_start(&talkers_list_toggle);

        let separator = gtk::Separator::new(gtk::Orientation::Vertical);
        headerbar.pack_start(&separator);

        // Push talker data
        let push_talker_data_button = gtk::Button::builder()
            .icon_name("go-up")
            .action_name("session.push_talker_data")
            .tooltip_text(format!("Push talker data ({})", session_actions::PUSH_TALKER_DATA_ACCEL))
            .build();
        headerbar.pack_start(&push_talker_data_button);

        // Commit talker data
        let commit_talker_data_button = gtk::Button::builder()
            .icon_name("dialog-ok")
            .action_name("session.commit_talker_data")
            .tooltip_text(format!("Commit talker data ({})", session_actions::COMMIT_TALKER_DATA_ACCEL))
            .build();
        headerbar.pack_start(&commit_talker_data_button);

        // Cancel talker data
        let cancel_talker_data_button = gtk::Button::builder()
            .icon_name("dialog-cancel")
            .action_name("session.cancel_talker_data")
            .tooltip_text(format!("Cancel talker data ({})", session_actions::CANCEL_TALKER_DATA_ACCEL))
            .build();
        headerbar.pack_start(&cancel_talker_data_button);

        // header bar right controls
        let settings_menu = settings::menu();
        let menu_button = gtk::MenuButton::builder()
            .icon_name("open-menu-symbolic")
            .menu_model(&settings_menu)
            .build();
        headerbar.pack_end(&menu_button);

        let right_separator = gtk::Separator::new(gtk::Orientation::Vertical);
        headerbar.pack_end(&right_separator);

        let record_button = gtk::Button::builder()
            .icon_name("media-record-symbolic")
            .action_name("session.record")
            .tooltip_text(format!("Record ({})", session_actions::RECORD_ACCEL))
            .build();
        headerbar.pack_end(&record_button);

        let stop_button = gtk::Button::builder()
            .icon_name("media-playback-stop-symbolic")
            .action_name("session.stop")
            .tooltip_text(format!("Stop ({})", session_actions::STOP_ACCEL))
            .build();
        headerbar.pack_end(&stop_button);

        let play_or_pause_icon = gtk::Image::from_icon_name("media-playback-start-symbolic");
        let play_or_pause_button = gtk::Button::builder()
            .child(&play_or_pause_icon)
            .action_name("session.play")
            .tooltip_text(format!("Play ({})", session_actions::PLAY_ACCEL))
            .build();
        headerbar.pack_end(&play_or_pause_button);


        // Message view
        let message_view_label = gtk::Label::builder().hexpand(true).lines(5).halign(gtk::Align::Center).build();
        let message_view_scrolledwindow = gtk::ScrolledWindow::builder()
            .child(&message_view_label)
            .max_content_height(128)
            .build();
        let message_view_close_button = gtk::Button::from_icon_name("window-close-symbolic");
        let message_view_box = gtk::Box::new(gtk::Orientation::Horizontal, 2);
        
        message_view_box.append(&message_view_scrolledwindow);
        message_view_box.append(&message_view_close_button);
        
        let message_view_revealer = gtk::Revealer::builder().transition_type(gtk::RevealerTransitionType::SlideDown).transition_duration(200).build();
        message_view_revealer.set_child(Some(&message_view_box));


        // Talker data view
        let talker_data_view = sourceview5::View::builder()
            .wrap_mode(gtk::WrapMode::Word)
            .vscroll_policy(gtk::ScrollablePolicy::Natural)
            .highlight_current_line(true)
            .build();

        let talker_data_view_scrolledwindow = gtk::ScrolledWindow::builder()
            .child(&talker_data_view)
            .hadjustment(&talker_data_view.hadjustment().unwrap())
            .vexpand(true)
            .max_content_height(256)
            .build();


        // Split pane
        let split_pane = gtk::Box::new(gtk::Orientation::Horizontal, 2);

        // Talkers box
        let talkers_box = gtk::Box::new(gtk::Orientation::Vertical, 2);

        let talkers_box_scrolledwindow = gtk::ScrolledWindow::builder()
            .min_content_width(256)
            .vexpand(true)
            .child(&talkers_box)
            .visible(false)
            .build();
        split_pane.append(&talkers_box_scrolledwindow);
        talkers_box_scrolledwindow.set_visible(true);


        // Graph view
        let graph_view = GraphView::new_ref(&session_presenter, event_bus);

        let graph_view_scrolledwindow = gtk::ScrolledWindow::builder()
            .hexpand(true)
            .vexpand(true)
            .child(graph_view.borrow().area())
            .build();
        split_pane.append(&graph_view_scrolledwindow);

        // Vertical box
        let v_box = gtk::Box::new(gtk::Orientation::Vertical, 2);
        v_box.append(&message_view_revealer);
        v_box.append(&talker_data_view_scrolledwindow);
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
        // talkers tree toggle
        talkers_list_toggle.connect_toggled(move |tb| {
            if tb.is_active() {
                talkers_box_scrolledwindow.set_visible(true);
            } else {
                talkers_box_scrolledwindow.set_visible(false);
            }
        });

        // Message view close
        let message_view_revealer_ctrl = message_view_revealer.clone();
        message_view_close_button.connect_clicked(move |_| {
            message_view_revealer_ctrl.set_reveal_child(false);
        });

        window.present();

        let action_window = window.clone();

        let av = Self {
            window,
            play_or_pause_button,
            play_or_pause_icon,
            stop_button,
            record_button,
            message_view_revealer,
            message_view_label,
            talker_data_view,
            talker_data_view_scrolledwindow,
            push_talker_data_button,
            commit_talker_data_button,
            cancel_talker_data_button,
            talkers_box,
            graph_view,
            session_presenter: session_presenter.clone(),
            event_bus: event_bus.clone(),
        };

        av.hide_talker_data_editor();

        let rav = Rc::new(RefCell::new(av));
        
        session_actions::create_actions_entries(application, &action_window, &rav, &session_presenter);
        ApplicationView::observe(&rav, event_bus);
        
        Ok(rav)
    }

    // Plugins list
    fn fill_talkers_list(
        &self,
        categorized_talkers_label_model: &Vec<(String, Vec<(String, String)>)>,
    ) {
        for (category, talkers) in categorized_talkers_label_model {
            let talkers_store = gio::ListStore::new::<TalkerObject>();

            let talkers_item_factory = gtk::SignalListItemFactory::new();
            // talkers list item factory setup
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

            // talkers list selection
            let session_ctrl = self.session_presenter.clone();
            talkers_list.connect_activate(move |list_view, position| {
                let model = list_view.model().expect("The model has to exist.");
                let talker_object = model
                    .item(position)
                    .and_downcast::<TalkerObject>()
                    .expect("The item has to be an `TalkerObject`.");

                let talker_model = talker_object.model();

                if talker_model.is_empty() {
                } else {
                    session_ctrl.borrow_mut().add_talker(talker_model.as_str());
                }
            });

            for (label, model) in talkers {
                talkers_store.append(&TalkerObject::new(label, model));
            }
            let expander = gtk::Expander::builder()
                .label(category)
                .child(&talkers_list)
                .vexpand(false)
                .build();
            self.talkers_box.append(&expander);
        }
        self.talkers_box.append(&gtk::Label::new(None));
    }

    // Talker data editor
    fn edit_talker_data(&self, talker_id: Id) {
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
                    self.talker_data_view.set_buffer(Some(&text_buffer));
                    self.talker_data_view_scrolledwindow.set_visible(true);
                    self.push_talker_data_button.set_visible(true);
                    self.commit_talker_data_button.set_visible(true);
                    self.cancel_talker_data_button.set_visible(true);
                    self.talker_data_view.grab_focus();
                }
                Data::File(_) => {
                    let selected_data_talker = self.graph_view.borrow().selected_data_talker();

                    if let Some(talker_id) = selected_data_talker {
                        let open_ctrl = self.session_presenter.clone();

                        let dialog = gtk::FileDialog::builder()
                        .title("Choose a file")
                        .accept_label("Open")
                        .build();

                        dialog.open(Some(&self.window), Cancellable::NONE, move |file| {
                            if let Ok(file) = file {
                                let path_buf = file.path().expect("Couldn't get file path");

                                open_ctrl.borrow_mut().set_talker_data(talker_id,&path_buf.to_string_lossy());
                            }
                        });
                    }
                },
                _ => (),
            }
        }
    }

    pub fn push_talker_data(&self) {
        let selected_data_talker = self.graph_view.borrow().selected_data_talker();

        if let Some(talker_id) = selected_data_talker {
            let text_buffer = self.talker_data_view.buffer();

            self.session_presenter.borrow_mut().set_talker_data(
                talker_id,
                &text_buffer.text(&text_buffer.start_iter(), &text_buffer.end_iter(), false),
            );
        }
    }

    pub fn commit_talker_data(&self) {
        self.push_talker_data();
        self.close_talker_data_editor();
    }

    pub fn cancel_talker_data(&self) {
        self.close_talker_data_editor();
    }

    fn close_talker_data_editor(&self) {
        self.hide_talker_data_editor();

        let res = self.graph_view.borrow().unselect_data_talker();
        self.event_bus.borrow().notify_notifications_result(res);
    }

    fn hide_talker_data_editor(&self) {
        //        self.text_view.set_buffer(None::<&impl IsA<TextBuffer>>);
        self.talker_data_view_scrolledwindow.set_visible(false);
        self.push_talker_data_button.set_visible(false);
        self.commit_talker_data_button.set_visible(false);
        self.cancel_talker_data_button.set_visible(false);
    }

    // Messages view
    fn display_message(&self, message: &String, tags: &str) {
        let msg = gtk::glib::markup_escape_text(&message.replace("\\n", "\n"));
        let markup_msg = format!("<span {}>{}</span>", tags, msg);
        self.message_view_label.set_markup(&markup_msg);
        self.message_view_revealer.set_reveal_child(true);
    }

    fn display_info_message(&self, message: &String) {
        self.display_message(message, "color=\"#8080FF\"");
    }

    fn display_warning_message(&self, message: &String) {
        self.display_message(message, "color=\"#80FFFF\"");
    }

    fn display_error_message(&self, message: &String) {
        self.display_message(message, "color=\"#FF8080\" weight=\"bold\"");
    }

    fn hide_message(&self) {
        self.message_view_revealer.set_reveal_child(false);
    }

    fn observe(observer: &RApplicationView, bus: &REventBus) {
        let obs = observer.clone();

        bus.borrow_mut()
            .add_observer(Box::new(move |notification| match notification {
                Notification::State(state) => {
                    obs.borrow().hide_message();

                    match state {
                        State::Playing => {
                            obs.borrow()
                                .play_or_pause_icon
                                .set_from_icon_name(Some("media-playback-pause"));
                            obs.borrow().record_button.set_sensitive(false);
                            obs.borrow().stop_button.set_sensitive(true);
                        }
                        State::Recording => {
                            obs.borrow().play_or_pause_button.set_sensitive(false);
                            obs.borrow().record_button.set_sensitive(false);
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
                            obs.borrow().record_button.set_sensitive(true);
                            obs.borrow().stop_button.set_sensitive(false);
                        }
                        State::Exited => {
                            obs.borrow()
                                .play_or_pause_icon
                                .set_from_icon_name(Some("media-playback-start"));
                            obs.borrow().play_or_pause_button.set_sensitive(true);
                            obs.borrow().record_button.set_sensitive(true);
                            obs.borrow().stop_button.set_sensitive(false);
                        }
                    }
                },
                Notification::NewSession(name) => {
                    obs.borrow().hide_message();
                    obs.borrow().hide_talker_data_editor();
                    obs.borrow().window.set_title(Some(&name));
                }
                Notification::SessionSaved => obs.borrow().display_info_message(&"Session saved.".to_string()),
                Notification::SessionSavedAs(name) => {
                    obs.borrow().hide_message();
                    obs.borrow().window.set_title(Some(&name));
                }
                Notification::Tick(tick) => println!("Todo : Applicationview.set_tick {}", tick),
                Notification::TimeRange(st, et) => {
                    println!("Todo : Applicationview.set_time_range {} <-> {}", st, et)
                }
                Notification::TalkersRange(talkers) => obs.borrow().fill_talkers_list(&talkers),
                Notification::EditTalkerData(talker_id) => {
                    obs.borrow().edit_talker_data(*talker_id)
                }
                Notification::CurveAdded => println!("Todo : Applicationview.CurveAdded"),
                Notification::CurveRemoved => println!("Todo : Applicationview.CurveRemoved"),
                Notification::Info(msg) => obs.borrow().display_info_message(msg),
                Notification::Warning(msg) => obs.borrow().display_warning_message(msg),
                Notification::Error(msg) => obs.borrow().display_error_message(msg),
                Notification::TalkerChanged => obs.borrow().hide_message(),
                _ => (),
            }))
    }
}
