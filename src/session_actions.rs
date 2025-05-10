use gtk::prelude::*;
use gtk::FileDialog;
use gtk::glib::{self, clone};
use gtk::gio::{ActionEntry, Cancellable, SimpleActionGroup};

use crate::session_presenter::RSessionPresenter;
use crate::application_view::RApplicationView;
use crate::ui::session_opening_dialog;
use crate::ui::session_saving_dialog;

pub const NEW_SESSION_ACCEL: &str = "<Ctrl>N";
pub const OPEN_SESSION_ACCEL: &str = "<Ctrl>O";
pub const SAVE_SESSION_ACCEL: &str = "<Ctrl>S";
pub const SAVE_SESSION_AS_ACCEL: &str = "<Ctrl><Shift>S";
pub const UNDO_ACCEL: &str = "<Ctrl>Z";
pub const REDO_ACCEL: &str = "<Ctrl><Shift>Z";
pub const PLAY_ACCEL: &str = "<Ctrl>P";
pub const STOP_ACCEL: &str = "<Ctrl>T";
pub const RESTART_ACCEL: &str = "<Ctrl>R";
pub const RECORD_ACCEL: &str = "<Ctrl><Shift>R";
pub const PUSH_TALKER_DATA_ACCEL: &str = "<Ctrl>U";
pub const COMMIT_TALKER_DATA_ACCEL: &str = "<Ctrl>M";
pub const CANCEL_TALKER_DATA_ACCEL: &str = "<Ctrl>W";
pub const DUPLIATE_SELECTED_TALKERS_ACCEL: &str = "<Ctrl>D";
pub const FIND_FORWARD_ACCEL: &str = "<Ctrl>F";
pub const FIND_BACKWARD_ACCEL: &str = "<Ctrl><Shift>F";

pub fn create_actions_entries(
    application: &gtk::Application,
    window: &gtk::ApplicationWindow,
    view: &RApplicationView,
    session_presenter: &RSessionPresenter) {

    let mut entries = Vec::new();

    // New session action
    let new = ActionEntry::builder("new")
    .activate(clone!(#[weak] window, #[weak] session_presenter, move |_, _, _| {
        if session_presenter.borrow().is_modified() {
            let _ = session_saving_dialog::create(&window, &session_presenter, move |_, ctrl| ctrl.borrow_mut().new_session() );
        }
        else {
            session_presenter.borrow_mut().new_session()
        }
    }))
    .build();

    entries.push(new);

    application.set_accels_for_action("session.new", &[NEW_SESSION_ACCEL]);

    // Open session action
    let open = ActionEntry::builder("open")
    .activate(clone!(#[weak] window, #[weak] session_presenter, move |_, _, _| {
        if session_presenter.borrow().is_modified() {
            let _ = session_saving_dialog::create(&window, &session_presenter, move |win, ctrl| {
                let _ = session_opening_dialog::create(win, ctrl);
            } );
        }
        else {
            let _ = session_opening_dialog::create(&window, &session_presenter);
        }
    }))
    .build();

    entries.push(open);

    application.set_accels_for_action("session.open", &[OPEN_SESSION_ACCEL]);


    // Save session action
    let save = ActionEntry::builder("save")
    .activate(clone!(#[strong] session_presenter, move |_, _, _| session_presenter.borrow_mut().save_session()))
    .build();

    entries.push(save);

    application.set_accels_for_action("session.save", &[SAVE_SESSION_ACCEL]);


    // Save session as action
    let save_as = ActionEntry::builder("save_as")
    .activate(clone!(#[weak] window, #[weak] session_presenter, move |_, _, _| {
        let dialog = FileDialog::builder().title("Choose a Graffophone session record file")
        .accept_label("Open").initial_name(session::session::NEW_SESSION_FILENAME)
        .build();

        dialog.save(Some(&window), Cancellable::NONE, move |file| {
            if let Ok(file) = file {
                let path_buf = file.path().expect("Couldn't get file path");
                session_presenter.borrow_mut().save_session_as(&path_buf.to_string_lossy());
            }
        });
    }))
    .build();

    entries.push(save_as);

    application.set_accels_for_action("session.save_as", &[SAVE_SESSION_AS_ACCEL]);


    // Undo action
    let undo = ActionEntry::builder("undo")
    .activate(clone!(#[strong] view, move |_: &SimpleActionGroup, _, _| view.borrow().undo()))
    .build();

    entries.push(undo);

    application.set_accels_for_action("session.undo", &[UNDO_ACCEL]);

    // Redo action
    let redo = ActionEntry::builder("redo")
    .activate(clone!(#[strong] view, move |_: &SimpleActionGroup, _, _| view.borrow().redo()))
    .build();

    entries.push(redo);

    application.set_accels_for_action("session.redo", &[REDO_ACCEL]);


    // Toggle Play and pause action
    let play = ActionEntry::builder("play")
    .activate(clone!(#[strong] session_presenter, move |_: &SimpleActionGroup, _, _| session_presenter.borrow_mut().play_or_pause(&session_presenter)))
    .build();

    entries.push(play);

    application.set_accels_for_action("session.play", &[PLAY_ACCEL]);

    // Stop action
    let stop = ActionEntry::builder("stop")
    .activate(clone!(#[strong] session_presenter, move |_: &SimpleActionGroup, _, _| session_presenter.borrow_mut().stop()))
    .build();

    entries.push(stop);

    application.set_accels_for_action("session.stop", &[STOP_ACCEL]);

    // Restart action
    let restart = ActionEntry::builder("restart")
    .activate(clone!(#[strong] session_presenter, move |_: &SimpleActionGroup, _, _| {
        session_presenter.borrow_mut().stop();
        session_presenter.borrow_mut().play_or_pause(&session_presenter);
    }))
    .build();

    entries.push(restart);

    application.set_accels_for_action("session.restart", &[RESTART_ACCEL]);

    // Record action
    let record = ActionEntry::builder("record")
    .activate(clone!(#[strong] session_presenter, move |_: &SimpleActionGroup, _, _| session_presenter.borrow_mut().record(&session_presenter)))
    .build();

    entries.push(record);

    application.set_accels_for_action("session.record", &[RECORD_ACCEL]);


    // Push talker data action
    let push_talker_data = ActionEntry::builder("push_talker_data")
    .activate(clone!(#[strong] view, move |_: &SimpleActionGroup, _, _| view.borrow().push_talker_data()))
    .build();

    entries.push(push_talker_data);

    application.set_accels_for_action("session.push_talker_data", &[PUSH_TALKER_DATA_ACCEL]);

    // Commit talker data action
    let commit_talker_data = ActionEntry::builder("commit_talker_data")
    .activate(clone!(#[strong] view, move |_: &SimpleActionGroup, _, _| view.borrow().commit_talker_data()))
    .build();

    entries.push(commit_talker_data);

    application.set_accels_for_action("session.commit_talker_data", &[COMMIT_TALKER_DATA_ACCEL]);

    // Cancel talker data action
    let cancel_talker_data = ActionEntry::builder("cancel_talker_data")
    .activate(clone!(#[strong] view, move |_: &SimpleActionGroup, _, _| view.borrow().cancel_talker_data()))
    .build();

    entries.push(cancel_talker_data);

    application.set_accels_for_action("session.cancel_talker_data", &[CANCEL_TALKER_DATA_ACCEL]);

    // Duplicate selected talkers action
    let duplicate_selected_talkers = ActionEntry::builder("duplicate_selected_talkers")
    .activate(clone!(#[strong] view, move |_: &SimpleActionGroup, _, _| view.borrow().duplicate_selected_talkers()))
    .build();

    entries.push(duplicate_selected_talkers);

    application.set_accels_for_action("session.duplicate_selected_talkers", &[DUPLIATE_SELECTED_TALKERS_ACCEL]);

    // Find actions
    // Find forward action
    let find_forward = ActionEntry::builder("find_forward")
    .activate(clone!(#[strong] view, move |_: &SimpleActionGroup, _, _| view.borrow_mut().find_next(false)))
    .build();

    entries.push(find_forward);

    application.set_accels_for_action("session.find_forward", &[FIND_FORWARD_ACCEL]);

    // Find backward action
    let find_backward = ActionEntry::builder("find_backward")
    .activate(clone!(#[strong] view, move |_: &SimpleActionGroup, _, _| view.borrow_mut().find_next(true)))
    .build();

    entries.push(find_backward);

    application.set_accels_for_action("session.find_backward", &[FIND_BACKWARD_ACCEL]);


    let actions = SimpleActionGroup::new();
    actions.add_action_entries(entries);
    window.insert_action_group("session", Some(&actions));


    // Ask to save session on close window
    window.connect_close_request(clone!(#[strong] application, #[strong] session_presenter, move |window| {

        if session_presenter.borrow().is_modified() {
            let app = application.clone();
            let _ = session_saving_dialog::create(window, &session_presenter, move |_, _| app.quit());

            glib::Propagation::Stop
        }
        else {
            glib::Propagation::Proceed
        }
    }));
}
