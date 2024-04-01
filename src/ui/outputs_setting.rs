use gtk::{
    glib::{self, clone}, prelude::GridExt,
};

use gtk::prelude::{BoxExt, ButtonExt, GtkWindowExt};

use talker::identifier::Identifiable;

use crate::session_presenter::RSessionPresenter;

pub fn expose(app: &gtk::Application, session_presenter: &RSessionPresenter,) {
    
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

    let mut row = 0;

    for mixer in session_presenter.borrow().session().mixers().values() {
        let mixer_name = gtk::Label::new(Some(&mixer.borrow().name()));

        grid.attach(&mixer_name, 0, row, 1, 1);

        for output in mixer.borrow().outputs() {
            let output_name = gtk::Entry::builder().text(output.borrow().name()).build();
            let sup_button = gtk::Button::from_icon_name("list-remove-symbolic");

            grid.attach(&output_name, 0, row, 1, 1);
            grid.attach(&sup_button, 1, row, 1, 1);

            row +=1;
        }
        let output_name = gtk::Entry::builder().text("Output name").build();
        let add_button = gtk::Button::from_icon_name("list-add-symbolic");

        grid.attach(&output_name, 0, row, 1, 1);
        grid.attach(&add_button, 1, row, 1, 1);
    
        row +=1;
    }

    let cancel_button = gtk::Button::builder().label("Cancel").hexpand(true).build();
    let ok_button = gtk::Button::builder().label("Ok").hexpand(true).build();

    let action_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(2)
        .build();
    action_box.append(&cancel_button);
    action_box.append(&ok_button);

    let widget = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(2)
        .build();
    widget.append(&grid);
    widget.append(&action_box);

    let window = gtk::Window::builder()
        .application(app)
        .title("Outputs")
        .child(&widget)
        .default_width(200)
        .default_height(120)
        .modal(true)
        .visible(true)
        .build();

    ok_button.connect_clicked(clone!(@weak window => move |_| {
        window.destroy();
    }));
    cancel_button.connect_clicked(clone!(@weak window => move |_| window.destroy()));

    window.present();
}
