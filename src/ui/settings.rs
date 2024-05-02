
use gio::{prelude::ApplicationExt, ActionEntry};

use crate::session_presenter::RSessionPresenter;
use crate::ui::general_settings;
use crate::ui::session_settings;

pub fn action_entries(app: &gtk::Application, session_presenter: &RSessionPresenter,) -> impl IntoIterator<Item = ActionEntry<gtk::Application>> {

    let toggle_feedback = gio::ActionEntry::builder("toggle_feedback")
    .activate(|_, _, _| println!("TODO : toggle_feedback"))
    .build();

    let rssp = session_presenter.clone();
    let general_settings = gio::ActionEntry::builder("general_settings")
    .activate(move |app, _, _| general_settings::expose(app, &rssp))
    .build();

    let ossp = session_presenter.clone();
    let session_settings = gio::ActionEntry::builder("session_settings")
    .activate(move |app, _, _| session_settings::expose(app, &ossp))
    .build();

    let about = gio::ActionEntry::builder("about")
    .activate(|_, _, _| println!("About was pressed"))
    .build();

    let quit = gio::ActionEntry::builder("quit")
    .activate(|app: &gtk::Application, _, _| app.quit())
    .build();

    [toggle_feedback, general_settings, session_settings, about, quit]
}

pub fn menu() -> gio::Menu {
    let menu = gio::Menu::new();

    // TODO : menu.append(Some("Feedback"), Some("app.toggle_feedback"));
    menu.append(Some("General settings"), Some("app.general_settings"));
    menu.append(Some("Session settings"), Some("app.session_settings"));
    menu.append(Some("Quit"), Some("app.quit"));

    menu
}
