extern crate failure;

extern crate cairo;
extern crate gdk;
extern crate gio;
extern crate glib;
extern crate gtk;

extern crate gramotor;

use gio::prelude::*;

use std::env::args;

mod graph_controler;
mod graph_view;
mod gui;

use graph_controler::GraphControler;

fn main() {
    let application =
        gtk::Application::new(Some("com.github.gndl.graffophone"), Default::default())
            .expect("Initialization failed...");

    application.connect_activate(|app| {
        let graph_controler = GraphControler::new_ref();
        gui::build(app, graph_controler);
    });

    application.run(&args().collect::<Vec<_>>());
}
