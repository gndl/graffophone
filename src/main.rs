extern crate failure;
extern crate gio;
extern crate glib;
extern crate gramotor;
extern crate gtk;

use gio::prelude::*;

use std::env::args;

mod gui;

fn main() {
    let application =
        gtk::Application::new(Some("com.github.gndl.graffophone"), Default::default())
            .expect("Initialization failed...");

    application.connect_activate(|app| {
        gui::build(app);
    });

    application.run(&args().collect::<Vec<_>>());
}
