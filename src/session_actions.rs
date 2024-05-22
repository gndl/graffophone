use gtk::prelude::*;
use gtk::FileDialog;
use gtk::glib;
use gtk::gio::{ActionEntry, Cancellable, SimpleActionGroup};

use crate::session_presenter::RSessionPresenter;
use crate::application_view::RApplicationView;

pub fn create_actions_entries(application: &gtk::Application, window: &gtk::ApplicationWindow, view: &RApplicationView, session_presenter: &RSessionPresenter,) {

    // New session action
    let new_ctrl = session_presenter.clone();
    
    let new = ActionEntry::builder("new")
    .activate(move |_, _, _| new_ctrl.borrow_mut().new_session())
    .build();

    application.set_accels_for_action("session.new", &["<Ctrl>N"]);


    // Open session action
    let open_ctrl = session_presenter.clone();

    let open = ActionEntry::builder("open")
    .activate(glib::clone!(@weak window, @weak open_ctrl => move |_, _, _| {
        let dialog = FileDialog::builder()
            .title("Choose a Graffophone session record file")
            .accept_label("Open")
            .build();

        dialog.open(Some(&window), Cancellable::NONE, move |file| {
            if let Ok(file) = file {
                let path_buf = file.path().expect("Couldn't get file path");
                        open_ctrl.borrow_mut().open_session(&path_buf.to_string_lossy());
            }
        });
    }))
    .build();

    application.set_accels_for_action("session.open", &["<Ctrl>O"]);


    // Save session action
    let save_ctrl = session_presenter.clone();
    
    let save = ActionEntry::builder("save")
    .activate(move |_, _, _| save_ctrl.borrow_mut().save_session())
    .build();

    application.set_accels_for_action("session.save", &["<Ctrl>S"]);


    // Save session as action
    let save_as_ctrl = session_presenter.clone();

    let save_as = ActionEntry::builder("save_as")
    .activate(glib::clone!(@weak window, @weak save_as_ctrl => move |_, _, _| {
        let dialog = FileDialog::builder().title("Choose a Graffophone session record file")
        .accept_label("Open").initial_name(session::session::NEW_SESSION_FILENAME)
        .build();

        dialog.save(Some(&window), Cancellable::NONE, move |file| {
            if let Ok(file) = file {
                let path_buf = file.path().expect("Couldn't get file path");
                save_as_ctrl.borrow_mut().save_session_as(&path_buf.to_string_lossy());
            }
        });
    }))
    .build();

    application.set_accels_for_action("session.save_as", &["<Ctrl><Shift>S"]);


    // Toggle Play and pause action
    let play_ctrl = session_presenter.clone();

    let play = ActionEntry::builder("play")
    .activate(move |_: &SimpleActionGroup, _, _| play_ctrl.borrow_mut().play_or_pause(&play_ctrl))
    .build();

    application.set_accels_for_action("session.play", &["<Ctrl>P"]);

    // Stop action
    let stop_ctrl = session_presenter.clone();

    let stop = ActionEntry::builder("stop")
    .activate(move |_: &SimpleActionGroup, _, _| stop_ctrl.borrow_mut().stop())
    .build();

    application.set_accels_for_action("session.stop", &["<Ctrl>T"]);

    // Restart action
    let restart_ctrl = session_presenter.clone();

    let restart = ActionEntry::builder("restart")
    .activate(move |_: &SimpleActionGroup, _, _| {
        restart_ctrl.borrow_mut().stop();
        restart_ctrl.borrow_mut().play_or_pause(&restart_ctrl);
    })
    .build();

    application.set_accels_for_action("session.restart", &["<Ctrl>R"]);

    // Record action
    let record_ctrl = session_presenter.clone();

    let record = ActionEntry::builder("record")
    .activate(move |_: &SimpleActionGroup, _, _| record_ctrl.borrow_mut().record(&record_ctrl))
    .build();

    application.set_accels_for_action("session.record", &["<Ctrl><Shift>R"]);


    // Push talker data action
    let push_talker_data_view = view.clone();

    let push_talker_data = ActionEntry::builder("push_talker_data")
    .activate(move |_: &SimpleActionGroup, _, _| push_talker_data_view.borrow().push_talker_data())
    .build();

    application.set_accels_for_action("session.push_talker_data", &["<Ctrl>U"]);

    // Commit talker data action
    let commit_talker_data_view = view.clone();
    let commit_talker_data = ActionEntry::builder("commit_talker_data")
    .activate(move |_: &SimpleActionGroup, _, _| commit_talker_data_view.borrow().commit_talker_data())
    .build();

    application.set_accels_for_action("session.commit_talker_data", &["<Ctrl>M"]);

    // Cancel talker data action
    let cancel_talker_data_view = view.clone();
    let cancel_talker_data = ActionEntry::builder("cancel_talker_data")
    .activate(move |_: &SimpleActionGroup, _, _| cancel_talker_data_view.borrow().cancel_talker_data())
    .build();

    application.set_accels_for_action("session.cancel_talker_data", &["<Ctrl>W"]);


    let actions = SimpleActionGroup::new();
    actions.add_action_entries([new, open, save, save_as, play, stop, restart, record, push_talker_data, commit_talker_data, cancel_talker_data]);
    window.insert_action_group("session", Some(&actions));
}
