
use gio::{prelude::ApplicationExt, ActionEntry};
use crate::gio::prelude::ActionMapExtManual;

use crate::session_presenter::RSessionPresenter;
use crate::ui::general_settings;
use crate::ui::session_settings;

pub fn create_actions_entries(app: &gtk::Application, session_presenter: &RSessionPresenter,) {

    let toggle_feedback = ActionEntry::builder("toggle_feedback")
    .activate(|_, _, _| println!("TODO : toggle_feedback"))
    .build();

    let rssp = session_presenter.clone();
    let general_settings = ActionEntry::builder("general_settings")
    .activate(move |app, _, _| general_settings::expose(app, &rssp))
    .build();

    let ossp = session_presenter.clone();
    let session_settings = ActionEntry::builder("session_settings")
    .activate(move |app, _, _| session_settings::expose(app, &ossp))
    .build();

    let about = ActionEntry::builder("about")
    .activate(|_, _, _| println!("About was pressed"))
    .build();

    let quit = ActionEntry::builder("quit")
    .activate(|app: &gtk::Application, _, _| app.quit())
    .build();

    app.add_action_entries([toggle_feedback, general_settings, session_settings, about, quit]);
}

pub fn menu() -> gio::Menu {
    let menu = gio::Menu::new();

    // TODO : menu.append(Some("Feedback"), Some("app.toggle_feedback"));
    menu.append(Some("General settings"), Some("app.general_settings"));
    menu.append(Some("Session settings"), Some("app.session_settings"));

    menu
}

pub fn get_directory() -> std::path::PathBuf {
    session::util::configuration_path()
}
