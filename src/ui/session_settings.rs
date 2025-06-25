use gio::prelude::FileExt;
use gtk::{
    glib::{self, clone}, prelude::{BoxExt, ButtonExt, EditableExt, EntryBufferExtManual, EntryExt, GridExt, GtkWindowExt, WidgetExt}, DropDown, FileDialog,
};

use session::channel;
use talker::identifier::Id;

use crate::{output_presenter::{self, OutputPresenter}, session_presenter::RSessionPresenter};

const ACTION_COLUMN: i32 = 0;
const CODEC_COLUMN: i32 = 1;
const SAMPLE_RATE_COLUMN: i32 = 2;
const CHANNEL_LAYOUT_COLUMN: i32 = 3;
const FILEPATH_COLUMN: i32 = 4;


fn add_output_selectors(window: &gtk::Window,
    session_presenter: &RSessionPresenter,
    mixer_id: Id,
    output_presenter: &OutputPresenter,
    outputs_box: &gtk::Grid,
    row: i32) -> i32 {

    let output_id = output_presenter.id();

    // Delete button
    let del_button = gtk::Button::from_icon_name("list-remove-symbolic");
    del_button.set_can_focus(false);

    del_button.connect_clicked(clone!(#[weak] window, #[weak] session_presenter, #[weak] outputs_box, move |_| {
        session_presenter.borrow_mut().remove_mixer_output(mixer_id, output_id);
        update_outputs_view(&window, &session_presenter, mixer_id, &outputs_box);
    }));

    outputs_box.attach(&del_button, ACTION_COLUMN, row, 1, 1);


    // CODEC selector
    let codec_selector = DropDown::from_strings(&output_presenter::CODECS_LABELS);
    codec_selector.set_selected(output_presenter.codec_index() as u32);
    codec_selector.set_can_focus(false);

    codec_selector.connect_selected_item_notify(clone!(#[weak] window, #[weak] session_presenter, #[weak] outputs_box, move |i| {
        session_presenter.borrow_mut().set_mixer_output_codec(mixer_id, output_id, i.selected() as usize);
        update_outputs_view(&window, &session_presenter, mixer_id, &outputs_box);
    }));

    outputs_box.attach(&codec_selector, CODEC_COLUMN, row, 1, 1);

    
    // Sample rate selector
    let sample_rate_selector = DropDown::from_strings(&output_presenter::SAMPLE_RATES);
    sample_rate_selector.set_selected(output_presenter.sample_rate_index() as u32);
    sample_rate_selector.set_can_focus(false);

    sample_rate_selector.connect_selected_item_notify(clone!(#[weak] window, #[weak] session_presenter, #[weak] outputs_box, move |i| {
        session_presenter.borrow_mut().set_mixer_output_sample_rate(mixer_id, output_id, i.selected() as usize);
        update_outputs_view(&window, &session_presenter, mixer_id, &outputs_box);
    }));

    outputs_box.attach(&sample_rate_selector, SAMPLE_RATE_COLUMN, row, 1, 1);

    
    // Channel layout selector
    let channel_layout_selector = DropDown::from_strings(channel::Layout::names());
    channel_layout_selector.set_selected(output_presenter.channel_layout_index() as u32);
    channel_layout_selector.set_can_focus(false);

    channel_layout_selector.connect_selected_item_notify(clone!(#[weak] window, #[weak] session_presenter, #[weak] outputs_box, move |i| {
        session_presenter.borrow_mut().set_mixer_output_channel_layout(mixer_id, output_id, i.selected() as usize);
        update_outputs_view(&window, &session_presenter, mixer_id, &outputs_box);
    }));

    outputs_box.attach(&channel_layout_selector, CHANNEL_LAYOUT_COLUMN, row, 1, 1);

    
    // File path entry
    let filepath_entry = gtk::Entry::builder()
        .primary_icon_name("document-open-symbolic").primary_icon_sensitive(true).primary_icon_activatable(true)
        .hexpand(true).width_request(360)
        .build();
    filepath_entry.buffer().set_text(output_presenter.file_path());

    filepath_entry.connect_icon_press(clone!(#[weak] window, #[weak] session_presenter, move |_, _| {
        let dialog = FileDialog::builder()
            .title("Choose a audio output file")
            .accept_label("Open")
            .build();

        dialog.open(Some(&window), gio::Cancellable::NONE, clone!(#[weak] session_presenter, move |file| {
            if let Ok(file) = file {
                let path_buf = file.path().expect("Couldn't get file path");

                if let Some(path) = path_buf.to_str() {
                    session_presenter.borrow_mut().set_mixer_output_file_path(mixer_id, output_id, path);
                }
            }
        }));
    }));

    filepath_entry.connect_changed(clone!(#[weak] session_presenter, move |i| {
        session_presenter.borrow_mut().set_mixer_output_file_path(mixer_id, output_id, i.buffer().text().as_str());
    }));

    outputs_box.attach(&filepath_entry, FILEPATH_COLUMN, row, 1, 1);

    row + 1
}

fn update_outputs_view(window: &gtk::Window,
        session_presenter: &RSessionPresenter,
        mixer_id: Id,
        outputs_box: &gtk::Grid,
    ) {
    outputs_box.remove_column(4);
    outputs_box.remove_column(3);
    outputs_box.remove_column(2);
    outputs_box.remove_column(1);
    outputs_box.remove_column(0);

    let mut row = 0;
    
    session_presenter.borrow().visite_mixer(mixer_id, |mixer| {
        for output in mixer.outputs() {
            row = add_output_selectors(&window, session_presenter, mixer_id, output, &outputs_box, row);
        }
    });
        
    let add_button = gtk::Button::from_icon_name("list-add-symbolic");
    add_button.set_can_focus(false);
    
    add_button.connect_clicked(clone!(#[weak] window, #[weak] session_presenter, #[weak] outputs_box, move |_| {
        session_presenter.borrow_mut().add_mixer_file_output(mixer_id);

        update_outputs_view(&window, &session_presenter, mixer_id, &outputs_box);
    }));

    outputs_box.attach(&add_button, ACTION_COLUMN, row, 1, 1);
}


pub fn expose(app: &gtk::Application, session_presenter: &RSessionPresenter,) {

    let widget = gtk::Box::builder().orientation(gtk::Orientation::Vertical)
    .spacing(20).margin_start(20).margin_end(20).margin_top(20).margin_bottom(20)
    .build();

    let window = gtk::Window::builder()
    .application(app)
    .title("Outputs")
    .child(&widget)
    .modal(true)
    .visible(true)
    .width_request(512).build();

    session_presenter.borrow_mut().init_mixers_presenters();

    // Mixers outputs
    for mixer in session_presenter.borrow().mixers_presenters() {
        let mixer_id = mixer.id();

        let outputs_box = gtk::Grid::builder()
            .margin_start(6).margin_end(6).margin_top(6).margin_bottom(6)
            .halign(gtk::Align::Start).valign(gtk::Align::Center)
            .row_spacing(6).column_spacing(6)
            .build();

        update_outputs_view(&window, session_presenter, mixer_id, &outputs_box);

        let outputs_label = format!("{} outputs", mixer.name());
        let outputs_frame = gtk::Frame::builder().label(outputs_label).child(&outputs_box).visible(true).build();
        widget.append(&outputs_frame);
    }

    let action_separator = gtk::Separator::builder().hexpand(true).vexpand(true).orientation(gtk::Orientation::Horizontal).build();
    widget.append(&action_separator);

    let action_box = gtk::Box::builder().orientation(gtk::Orientation::Horizontal).spacing(20).build();
    widget.append(&action_box);


    // Cancel button
    let cancel_button = gtk::Button::builder().label("Cancel").hexpand(true).can_focus(false).build();

    cancel_button.connect_clicked(clone!(#[weak] window, #[weak] session_presenter, move |_| {
        session_presenter.borrow_mut().cancel_mixers_presenters();
        window.destroy()
    }));

    action_box.append(&cancel_button);


    // OK button
    let ok_button = gtk::Button::builder().label("Ok").hexpand(true).build();
    ok_button.grab_focus();

    ok_button.connect_clicked(clone!(#[weak] window, #[weak] session_presenter, move |_| {
        session_presenter.borrow_mut().ratify_mixers_outputs();
        window.destroy()
    }));

    action_box.append(&ok_button);

    window.present();
}
