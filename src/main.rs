extern crate failure;

extern crate cairo;
extern crate gio;
extern crate gtk;

extern crate session;

use crate::gtk::prelude::ApplicationExt;
use crate::gtk::prelude::ApplicationExtManual;
use gtk::gdk::Display;
use gtk::prelude::GtkApplicationExt;
use gtk::{CssProvider, STYLE_PROVIDER_PRIORITY_APPLICATION};

use session::event_bus::EventBus;

mod application_view;
mod graph_presenter;
mod graph_view;
mod mixer_control;
mod mixer_presenter;
mod output_presenter;
mod session_actions;
mod session_presenter;
mod settings;
mod style;
mod talker_control;
mod ui;
mod util;

use application_view::ApplicationView;
use session_presenter::SessionPresenter;

fn main() {
    ui::plugin_ui::init();

    let application =
        gtk::Application::new(Some("com.gitlab.gndl.graffophone"), Default::default());

    application.connect_startup(|_: &gtk::Application| {
        // The CSS "magic" happens here.
        let provider = CssProvider::new();
        provider.load_from_string(include_str!("css/style.css").as_ref());
        // We give the CssProvided to the default screen so the CSS rules we added
        // can be applied to our window.
        gtk::style_context_add_provider_for_display(
            &Display::default().expect("Could not connect to a display."),
            &provider,
            STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    });

    application.connect_activate(|app| {
        sourceview5::init();
        let event_bus = EventBus::new_ref();

        let session_presenter = SessionPresenter::new_ref(&event_bus);

        settings::create_actions_entries(app, &session_presenter);
        app.set_accels_for_action("window.close", &["<Ctrl>Q"]);

        match ApplicationView::new_ref(app, &session_presenter, &event_bus) {
            Ok(_) => session_presenter.borrow().init(),
            Err(e) => eprintln!("{}", e),
        }
    });

    application.run();

    sourceview5::finalize();
}
