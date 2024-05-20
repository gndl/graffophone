
use std::str::FromStr;

use gtk::{
    glib::{self, clone}, prelude::{BoxExt, ButtonExt, GtkWindowExt, GridExt}, DropDown,
};

use crate::session_presenter::RSessionPresenter;


pub fn expose(app: &gtk::Application, session_presenter: &RSessionPresenter,) {
    let sample_rates_strs = ["8000", "11025", "16000", "22050", "32000", "44100", "48000", "88200", "96000"];
    let sample_rates = sample_rates_strs.map(|s| usize::from_str(s).unwrap());

    let sample_rate = session_presenter.borrow().sample_rate();

    let grid = gtk::Grid::builder()
        .margin_start(6)
        .margin_end(6)
        .margin_top(6)
        .margin_bottom(6)
        .halign(gtk::Align::Center)
        .valign(gtk::Align::Center)
        .row_spacing(6)
        .column_spacing(6)
        .build();

    let row = 0;

    let sample_rate_label = gtk::Label::new(Some("Sample rate : "));
    let sample_rate_selector = DropDown::from_strings(&sample_rates_strs);

    match sample_rates.binary_search(&sample_rate) {
        Result::Ok(idx) => sample_rate_selector.set_selected(idx as u32),
        Result::Err(e) => eprintln!("Sample rate {} unknown ({}).", sample_rate, e),
    }

    grid.attach(&sample_rate_label, 0, row, 1, 1);
    grid.attach(&sample_rate_selector, 1, row, 1, 1);

    let action_separator = gtk::Separator::builder().hexpand(true).vexpand(true).orientation(gtk::Orientation::Horizontal).build();

    let cancel_button = gtk::Button::builder().label("Cancel").hexpand(true).build();
    let ok_button = gtk::Button::builder().label("Ok").hexpand(true).build();

    let action_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(20)
        .build();
    action_box.append(&cancel_button);
    action_box.append(&ok_button);

    let widget = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(20).margin_start(20).margin_end(20).margin_top(20).margin_bottom(20)
        .build();
    widget.append(&grid);
    widget.append(&action_separator);
    widget.append(&action_box);

    let window = gtk::Window::builder()
        .application(app)
        .title("Runtime")
        .child(&widget)
        .modal(true)
        .visible(true)
        .build();

    let rssp = session_presenter.clone();
    ok_button.connect_clicked(clone!(@weak window => move |_| {
        let new_sample_rate = sample_rates[sample_rate_selector.selected() as usize];

        if new_sample_rate != sample_rate {
            rssp.borrow_mut().set_sample_rate(new_sample_rate);
        }

        window.destroy();
    }));
    cancel_button.connect_clicked(clone!(@weak window => move |_| window.destroy()));

    window.present();
}
