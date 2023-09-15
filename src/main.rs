extern crate failure;

extern crate cairo;
extern crate gio;
extern crate gtk;

extern crate session;

use crate::gtk::prelude::ApplicationExt;
use crate::gtk::prelude::ApplicationExtManual;
use gtk::gdk::Display;
use gtk::{CssProvider, STYLE_PROVIDER_PRIORITY_APPLICATION};

mod application_view;
mod bounded_float_entry;
mod graph_presenter;
mod graph_view;
mod mixer_control;
mod session_presenter;
mod style;
mod talker_control;
mod ui;
mod util;

use application_view::ApplicationView;
use session_presenter::SessionPresenter;

fn main() {
    let application =
        gtk::Application::new(Some("com.gitlab.gndl.graffophone"), Default::default());

    application.connect_startup(|_| {
        // The CSS "magic" happens here.
        let provider = CssProvider::new();
        provider.load_from_data(include_str!("css/style.css").as_ref());
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

        let session_presenter = SessionPresenter::new_ref();

        match ApplicationView::new_ref(app, &session_presenter) {
            Ok(_) => session_presenter.borrow_mut().init(),
            Err(e) => eprintln!("{}", e),
        }
    });

    application.run();

    sourceview5::finalize();
}
