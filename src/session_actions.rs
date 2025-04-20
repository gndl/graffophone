use gtk::prelude::*;
use gtk::FileDialog;
use gtk::glib;
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

pub fn create_actions_entries(
    application: &gtk::Application,
    window: &gtk::ApplicationWindow,
    view: &RApplicationView,
    session_presenter: &RSessionPresenter) {

    let mut entries = Vec::new();

    // New session action
    let new_ctrl = session_presenter.clone();
    
    let new = ActionEntry::builder("new")
    .activate(glib::clone!(@weak window, @weak new_ctrl => move |_, _, _| {
        if new_ctrl.borrow().is_modified() {
            let _ = session_saving_dialog::create(&window, &new_ctrl, move |_, ctrl| ctrl.borrow_mut().new_session() );
        }
        else {
            new_ctrl.borrow_mut().new_session()
        }
    }))
    .build();

    entries.push(new);

    application.set_accels_for_action("session.new", &[NEW_SESSION_ACCEL]);

    // Open session action
    let open_ctrl = session_presenter.clone();

    let open = ActionEntry::builder("open")
    .activate(glib::clone!(@weak window, @weak open_ctrl => move |_, _, _| {
        if open_ctrl.borrow().is_modified() {
            let _ = session_saving_dialog::create(&window, &open_ctrl, move |win, ctrl| {
                let _ = session_opening_dialog::create(win, ctrl);
            } );
        }
        else {
            let _ = session_opening_dialog::create(&window, &open_ctrl);
        }
    }))
    .build();

    entries.push(open);

    application.set_accels_for_action("session.open", &[OPEN_SESSION_ACCEL]);


    // Save session action
    let save_ctrl = session_presenter.clone();
    
    let save = ActionEntry::builder("save")
    .activate(move |_, _, _| save_ctrl.borrow_mut().save_session())
    .build();

    entries.push(save);

    application.set_accels_for_action("session.save", &[SAVE_SESSION_ACCEL]);


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

    entries.push(save_as);

    application.set_accels_for_action("session.save_as", &[SAVE_SESSION_AS_ACCEL]);


    // Undo action
    let undo_ctrl = view.clone();

    let undo = ActionEntry::builder("undo")
    .activate(move |_: &SimpleActionGroup, _, _| undo_ctrl.borrow().undo())
    .build();

    entries.push(undo);

    application.set_accels_for_action("session.undo", &[UNDO_ACCEL]);

    // Redo action
    let redo_ctrl = view.clone();

    let redo = ActionEntry::builder("redo")
    .activate(move |_: &SimpleActionGroup, _, _| redo_ctrl.borrow().redo())
    .build();

    entries.push(redo);

    application.set_accels_for_action("session.redo", &[REDO_ACCEL]);


    // Toggle Play and pause action
    let play_ctrl = session_presenter.clone();

    let play = ActionEntry::builder("play")
    .activate(move |_: &SimpleActionGroup, _, _| play_ctrl.borrow_mut().play_or_pause(&play_ctrl))
    .build();

    entries.push(play);

    application.set_accels_for_action("session.play", &[PLAY_ACCEL]);

    // Stop action
    let stop_ctrl = session_presenter.clone();

    let stop = ActionEntry::builder("stop")
    .activate(move |_: &SimpleActionGroup, _, _| stop_ctrl.borrow_mut().stop())
    .build();

    entries.push(stop);

    application.set_accels_for_action("session.stop", &[STOP_ACCEL]);

    // Restart action
    let restart_ctrl = session_presenter.clone();

    let restart = ActionEntry::builder("restart")
    .activate(move |_: &SimpleActionGroup, _, _| {
        restart_ctrl.borrow_mut().stop();
        restart_ctrl.borrow_mut().play_or_pause(&restart_ctrl);
    })
    .build();

    entries.push(restart);

    application.set_accels_for_action("session.restart", &[RESTART_ACCEL]);

    // Record action
    let record_ctrl = session_presenter.clone();

    let record = ActionEntry::builder("record")
    .activate(move |_: &SimpleActionGroup, _, _| record_ctrl.borrow_mut().record(&record_ctrl))
    .build();

    entries.push(record);

    application.set_accels_for_action("session.record", &[RECORD_ACCEL]);


    // Push talker data action
    let push_talker_data_view = view.clone();

    let push_talker_data = ActionEntry::builder("push_talker_data")
    .activate(move |_: &SimpleActionGroup, _, _| push_talker_data_view.borrow().push_talker_data())
    .build();

    entries.push(push_talker_data);

    application.set_accels_for_action("session.push_talker_data", &[PUSH_TALKER_DATA_ACCEL]);

    // Commit talker data action
    let commit_talker_data_view = view.clone();
    let commit_talker_data = ActionEntry::builder("commit_talker_data")
    .activate(move |_: &SimpleActionGroup, _, _| commit_talker_data_view.borrow().commit_talker_data())
    .build();

    entries.push(commit_talker_data);

    application.set_accels_for_action("session.commit_talker_data", &[COMMIT_TALKER_DATA_ACCEL]);

    // Cancel talker data action
    let cancel_talker_data_view = view.clone();
    let cancel_talker_data = ActionEntry::builder("cancel_talker_data")
    .activate(move |_: &SimpleActionGroup, _, _| cancel_talker_data_view.borrow().cancel_talker_data())
    .build();

    entries.push(cancel_talker_data);

    application.set_accels_for_action("session.cancel_talker_data", &[CANCEL_TALKER_DATA_ACCEL]);

    // Duplicate selected talkers action
    let duplicate_selected_talkers_view = view.clone();
    let duplicate_selected_talkers = ActionEntry::builder("duplicate_selected_talkers")
    .activate(move |_: &SimpleActionGroup, _, _| duplicate_selected_talkers_view.borrow().duplicate_selected_talkers())
    .build();

    entries.push(duplicate_selected_talkers);

    application.set_accels_for_action("session.duplicate_selected_talkers", &[DUPLIATE_SELECTED_TALKERS_ACCEL]);


    let actions = SimpleActionGroup::new();
    actions.add_action_entries(entries);
    window.insert_action_group("session", Some(&actions));


    // Ask to save session on close window
    let close_application = application.clone();
    let close_session = session_presenter.clone();

    window.connect_close_request(move |window| {

        if close_session.borrow().is_modified() {
            let app = close_application.clone();
            let _ = session_saving_dialog::create(window, &close_session, move |_, _| app.quit());

            glib::Propagation::Stop
        }
        else {
            glib::Propagation::Proceed
        }
    });
}
