use gtk::prelude::*;
use gtk::FileDialog;
use gtk::gio::Cancellable;


use crate::session_presenter::RSessionPresenter;

pub fn create(window: &gtk::ApplicationWindow, session_presenter: &RSessionPresenter) -> gtk::FileDialog {
    let filters = gio::ListStore::new::<gtk::FileFilter>();

    let gsr_filter = gtk::FileFilter::new();
    gsr_filter.add_pattern("*.gsr");
    filters.append(&gsr_filter);

    let no_filter = gtk::FileFilter::new();
    no_filter.add_pattern("*");
    filters.append(&no_filter);

    let dialog = FileDialog::builder()
        .title("Choose a Graffophone session record file")
        .accept_label("Open")
        .filters(&filters)
        .build();

        let session = session_presenter.clone();

    dialog.open(Some(window), Cancellable::NONE, move |file| {
        if let Ok(file) = file {
            let path_buf = file.path().expect("Couldn't get file path");
            session.borrow_mut().open_session(&path_buf.to_string_lossy());
        }
    });
    dialog
}
