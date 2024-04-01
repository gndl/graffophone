
use gio::{prelude::ApplicationExt, ActionEntry};

use crate::session_presenter::RSessionPresenter;
use crate::ui::outputs_setting;

pub fn action_entries(app: &gtk::Application, session_presenter: &RSessionPresenter,) -> impl IntoIterator<Item = ActionEntry<gtk::Application>> {

    let toggle_feedback = gio::ActionEntry::builder("toggle_feedback")
    .activate(|_, _, _| println!("toggle_feedback was pressed"))
    .build();

    let cosp = session_presenter.clone();
    let configure_outputs = gio::ActionEntry::builder("configure_outputs")
    .activate(move |app, _, _| outputs_setting::expose(app, &cosp))
    .build();

    let about = gio::ActionEntry::builder("about")
    .activate(|_, _, _| println!("About was pressed"))
    .build();

    let quit = gio::ActionEntry::builder("quit")
    .activate(|app: &gtk::Application, _, _| app.quit())
    .build();

    [toggle_feedback, configure_outputs, about, quit]
}

pub fn menu() -> gio::Menu {
    let menu = gio::Menu::new();

    menu.append(Some("Feedback"), Some("app.toggle_feedback"));
    menu.append(Some("Configure outputs"), Some("app.configure_outputs"));
    menu.append(Some("Quit"), Some("app.quit"));

    menu
}
