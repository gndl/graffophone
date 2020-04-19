extern crate failure;

extern crate cairo;
extern crate gdk;
extern crate gio;
extern crate glib;
extern crate gtk;

extern crate session;

use gio::prelude::*;

use std::env::args;

mod application_view;
mod graph_view;
mod session_controler;

use application_view::ApplicationView;
use session_controler::SessionControler;

fn main() {
    let application =
        gtk::Application::new(Some("com.github.gndl.graffophone"), Default::default())
            .expect("Initialization failed...");

    application.connect_activate(|app| {
        let session_controler = SessionControler::new_ref();

        match ApplicationView::new_ref(app, &session_controler) {
            Ok(_) => {
                session_controler.borrow_mut().init();
            }
            Err(e) => eprintln!("{}", e),
        }
    });

    application.run(&args().collect::<Vec<_>>());
}
