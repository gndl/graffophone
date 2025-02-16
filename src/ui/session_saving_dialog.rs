
use gtk::gio::Cancellable;

use crate::session_presenter::RSessionPresenter;

pub fn create<OnOk: Fn(&gtk::ApplicationWindow, &RSessionPresenter) + 'static>(window: &gtk::ApplicationWindow, session_presenter: &RSessionPresenter, on_ok: OnOk) -> gtk::AlertDialog {
    let dialog = gtk::AlertDialog::builder()
    .modal(true)
    .buttons(["Cancel", "Don't save", "Save"])
    .message("Save changes?")
    .build();

    let win = window.clone();
    let session = session_presenter.clone();

    dialog.choose(Some(window), Cancellable::NONE, move |r| {
        match r {
            Ok(button_idx) => {
                if button_idx > 0 {
                    if button_idx == 2 {
                        session.borrow_mut().save_session();
                    }
                    on_ok(&win, &session);
                }
            },
            Err(e) => println!("Error {}", e),
        }
    });
    dialog
}
