//#[macro_use]
use std::cell::RefCell;
use std::rc::Rc;

//use gdk::EventMask;
use gio::prelude::*;
//use gtk::gtk_sys::GtkScrolledWindow;
use gtk::prelude::*;
use gtk::{
    BoxExt, ButtonExt, CellRendererText, ContainerExt, GtkWindowExt, HeaderBarExt, IconSize,
    ScrolledWindowExt, TreeStore, TreeView, TreeViewColumn, WidgetExt,
};

//use cairo::enums::{FontSlant, FontWeight};
//use cairo::Context;

use session::event_bus::{Notification, REventBus};
//use session::factory::Factory;
use session::state::State;

use crate::graph_view::GraphView;
use crate::session_presenter::RSessionPresenter;

pub struct ApplicationView {
    talkers_tree: gtk::TreeView,
    play_or_pause_button: gtk::Button,
    stop_button: gtk::Button,
    play_icon: gtk::Image,
    pause_icon: gtk::Image,
}

pub type RApplicationView = Rc<RefCell<ApplicationView>>;

impl ApplicationView {
    fn new(
        application: &gtk::Application,
        session_presenter: &RSessionPresenter,
    ) -> Result<ApplicationView, failure::Error> {
        let selected_talker_label = gtk::Label::new(Some("Selected talkers :"));

        // header bar
        let headerbar = gtk::HeaderBar::new();
        headerbar.set_title(Some("Graffophone"));
        headerbar.set_show_close_button(true);

        // header bar left controls

        let new_session_button =
            gtk::Button::from_icon_name(Some("gtk-new"), IconSize::SmallToolbar);
        headerbar.pack_start(&new_session_button);

        let open_session_button = gtk::MenuButton::new();
        open_session_button.set_image(Some(&gtk::Image::from_icon_name(
            Some("gtk-open"),
            IconSize::SmallToolbar,
        )));
        headerbar.pack_start(&open_session_button);

        let save_session_button =
            gtk::Button::from_icon_name(Some("gtk-save"), IconSize::SmallToolbar);
        headerbar.pack_start(&save_session_button);

        let save_as_session_button =
            gtk::Button::from_icon_name(Some("gtk-save-as"), IconSize::SmallToolbar);
        headerbar.pack_start(&new_session_button);

        let separator = gtk::Separator::new(gtk::Orientation::Vertical);
        headerbar.pack_start(&separator);

        let talkers_tree_toggle = gtk::ToggleButton::new();
        talkers_tree_toggle.set_image(Some(&gtk::Image::from_icon_name(
            Some("gtk-index"),
            IconSize::SmallToolbar,
        )));
        talkers_tree_toggle.set_active(true);

        headerbar.pack_start(&talkers_tree_toggle);

        // header bar right controls
        let stop_button =
            gtk::Button::from_icon_name(Some("gtk-media-stop"), IconSize::SmallToolbar);

        headerbar.pack_end(&stop_button);

        let play_or_pause_button =
            gtk::Button::from_icon_name(Some("gtk-media-play"), IconSize::SmallToolbar);

        let play_icon = gtk::Image::from_icon_name(Some("gtk-media-play"), IconSize::SmallToolbar);

        let pause_icon =
            gtk::Image::from_icon_name(Some("gtk-media-pause"), IconSize::SmallToolbar);

        headerbar.pack_end(&play_or_pause_button);

        // Split pane
        let split_pane = gtk::Box::new(gtk::Orientation::Horizontal, 10);

        //            split_pane.set_size_request(-1, -1);

        // Talkers tree
        let talkers_tree = TreeView::new();
        talkers_tree.set_headers_visible(false);

        let column = TreeViewColumn::new();
        let cell = CellRendererText::new();

        column.pack_start(&cell, true);
        column.add_attribute(&cell, "text", 0);
        talkers_tree.append_column(&column);

        let talkers_tree_scrolledwindow =
            gtk::ScrolledWindow::new(None::<&gtk::Adjustment>, None::<&gtk::Adjustment>);
        talkers_tree_scrolledwindow.set_min_content_width(256);
        //            talkers_tree_scrolledwindow.set_vexpand(true);
        talkers_tree_scrolledwindow.add(&talkers_tree);
        talkers_tree_scrolledwindow.hide();
        split_pane.pack_start(&talkers_tree_scrolledwindow, false, true, 0);

        // Graph view
        let graph_view = GraphView::new_ref(&session_presenter);

        //            let graph_area = DrawingArea::new();

        let graph_view_scrolledwindow =
            gtk::ScrolledWindow::new(None::<&gtk::Adjustment>, None::<&gtk::Adjustment>);
        graph_view_scrolledwindow.add(graph_view.borrow().drawing_area());
        split_pane.pack_start(&graph_view_scrolledwindow, true, true, 0);

        // Vertical box
        let v_box = gtk::Box::new(gtk::Orientation::Vertical, 10);
        v_box.pack_start(&split_pane, true, true, 0);
        v_box.pack_start(&selected_talker_label, false, false, 0);

        // Actions
        // talkers tree toggle
        talkers_tree_toggle.connect_toggled(move |tb| {
            if tb.get_active() {
                talkers_tree_scrolledwindow.show();
            } else {
                talkers_tree_scrolledwindow.hide();
            }
        });

        // New session
        let new_ctrl = session_presenter.clone();
        new_session_button.connect_clicked(move |_| {
            new_ctrl.borrow_mut().new_session();
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

        // talkers tree selection
        let session_ctrl = session_presenter.clone();
        talkers_tree.connect_cursor_changed(move |tree_view| {
            let selection = tree_view.get_selection();

            if let Some((model, iter)) = selection.get_selected() {
                match model.get_value(&iter, 1).get::<String>() {
                    Ok(otalker_model) => match otalker_model {
                        Some(talker_model) => {
                            session_ctrl.borrow_mut().add_talker(&talker_model);
                            graph_view.borrow_mut().draw();
                        }
                        None => {
                            if let Some(path) = model.get_path(&iter) {
                                if tree_view.row_expanded(&path) {
                                    tree_view.collapse_row(&path);
                                } else {
                                    tree_view.expand_row(&path, true);
                                }
                            }
                        }
                    },
                    Err(e) => eprintln!("{}", e),
                };
            }
        });

        // ApplicationWindow
        let window = gtk::ApplicationWindow::new(application);

        window.set_titlebar(Some(&headerbar));
        window.set_border_width(5);
        window.set_position(gtk::WindowPosition::Center);
        window.set_default_size(1024, 768);

        window.add(&v_box);
        window.show_all();

        Ok(Self {
            talkers_tree,
            play_or_pause_button,
            stop_button,
            play_icon,
            pause_icon,
        })
    }

    pub fn new_ref(
        application: &gtk::Application,
        session_presenter: &RSessionPresenter,
    ) -> Result<RApplicationView, failure::Error> {
        let av = ApplicationView::new(application, session_presenter)?;
        let rav = Rc::new(RefCell::new(av));
        ApplicationView::observe(&rav, session_presenter.borrow().event_bus());
        Ok(rav)
    }

    fn fill_talkers_tree(
        &self,
        categorized_talkers_label_model: &Vec<(String, Vec<(String, String)>)>,
    ) {
        let talkers_store = TreeStore::new(&[String::static_type(), String::static_type()]);

        for (category, talkers) in categorized_talkers_label_model {
            let category_iter = talkers_store.insert_with_values(None, None, &[0], &[&category]);

            for (label, model) in talkers {
                talkers_store.insert_with_values(
                    Some(&category_iter),
                    None,
                    &[0, 1],
                    &[&label, &model],
                );
            }
        }
        self.talkers_tree.set_model(Some(&talkers_store));
    }

    /*
    fn build_talkers_menu(application: &gtk::Application, factory: &Factory, msgbox: &gtk::Label) {
    let categories_menu_bar = gio::Menu::new();

    for (category, talkers) in factory.get_categorized_talkers_label_model() {
    let category_menu = gio::Menu::new();

    for (label, model) in talkers {
    let action_name = label.replace(" ", "_");

    category_menu.append(Some(&label), Some(&action_name));

    let model_action = gio::SimpleAction::new(&action_name, None);
    model_action.connect_activate(clone!(@weak msgbox => move |_, _| {
    msgbox.set_text(&model);
                }));

                application.add_action(&model_action);
            }
            categories_menu_bar.append_submenu(Some(&category), &category_menu);
        }

        application.set_app_menu(Some(&categories_menu_bar));
    }
    */

    fn observe(observer: &RApplicationView, bus: &REventBus) {
        let obs = observer.clone();

        bus.borrow_mut()
            .add_observer(Box::new(move |notification| match notification {
                Notification::State(state) => match state {
                    State::Playing => {
                        obs.borrow()
                            .play_or_pause_button
                            .set_image(Some(&obs.borrow().pause_icon));
                        obs.borrow().stop_button.set_sensitive(true);
                    }
                    State::Paused => {
                        obs.borrow()
                            .play_or_pause_button
                            .set_image(Some(&obs.borrow().play_icon));
                        obs.borrow().stop_button.set_sensitive(true);
                    }
                    State::Stopped => {
                        obs.borrow()
                            .play_or_pause_button
                            .set_image(Some(&obs.borrow().play_icon));
                        obs.borrow().play_or_pause_button.set_sensitive(true);
                        obs.borrow().stop_button.set_sensitive(false);
                    }
                    State::Exited => {
                        obs.borrow()
                            .play_or_pause_button
                            .set_image(Some(&obs.borrow().play_icon));
                        obs.borrow().play_or_pause_button.set_sensitive(false);
                        obs.borrow().stop_button.set_sensitive(false);
                    }
                },
                Notification::Tick(tick) => println!("Todo : Applicationview.set_tick {}", tick),
                Notification::TimeRange(st, et) => {
                    println!("Todo : Applicationview.set_time_range {} <-> {}", st, et)
                }
                Notification::TalkersRange(talkers) => obs.borrow().fill_talkers_tree(&talkers),
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
