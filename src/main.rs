extern crate failure;

extern crate cairo;
extern crate gdk;
extern crate gio;
extern crate glib;
extern crate gtk;

extern crate session;

use crate::gtk::prelude::ApplicationExt;
use crate::gtk::prelude::ApplicationExtManual;

mod application_view;
mod bounded_float_entry;
mod graph_presenter;
mod graph_view;
mod mixer_control;
mod session_presenter;
mod style;
mod talker_control;
mod util;

use application_view::ApplicationView;
use session_presenter::SessionPresenter;

fn main() {
    let application =
        gtk::Application::new(Some("com.github.gndl.graffophone"), Default::default());

    application.connect_activate(|app| {
        let session_presenter = SessionPresenter::new_ref();

        match ApplicationView::new_ref(app, &session_presenter) {
            Ok(_) => {
                session_presenter.borrow_mut().init();
            }
            Err(e) => eprintln!("{}", e),
        }
    });

    application.run();
}
