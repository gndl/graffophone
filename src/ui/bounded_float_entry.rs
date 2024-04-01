use crate::gtk::Adjustment;
use crate::gtk::SpinButton;
use gtk::glib::IsA;
use gtk::traits::{AdjustmentExt, BoxExt, ButtonExt};

pub fn create<
    OnValueChanged: Fn(f64) + 'static,
    OnOk: Fn(f64) + 'static,
    OnCancel: Fn(f64) + 'static,
    OnDefault: Fn(f64) + 'static,
>(
    min: f64,
    max: f64,
    def: f64,
    current: f64,
    on_value_changed: OnValueChanged,
    on_ok: OnOk,
    on_cancel: OnCancel,
    on_default: OnDefault,
) -> impl IsA<gtk::Widget> {
    let step = f64::min((max - min) / 40000., 1.);
    let adjustment = Adjustment::new(current, min, max, step, step * 100., 0.);

    let scale = gtk::Scale::builder()
        .adjustment(&adjustment)
        .orientation(gtk::Orientation::Vertical)
        .width_request(64)
        .height_request(360)
        .inverted(true)
        .draw_value(false)
        .build();

    let entry = SpinButton::builder()
        .adjustment(&adjustment)
        .climb_rate(5.)
        .digits(4)
        .hexpand(false)
        .build();

    let cancel_button = gtk::Button::builder().label("Cancel").hexpand(true).build();

    let default_button = gtk::Button::builder()
        .label("Default")
        .hexpand(true)
        .build();

    let ok_button = gtk::Button::builder().label("Ok").hexpand(true).build();

    // box
    let value_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(2)
        .build();
    value_box.append(&scale);
    value_box.append(&entry);

    let action_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(2)
        .build();
    action_box.append(&cancel_button);
    action_box.append(&default_button);
    action_box.append(&ok_button);

    let widget = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(2)
        .build();
    widget.append(&value_box);
    widget.append(&action_box);

    adjustment.connect_value_changed(move |adj| on_value_changed(adj.value()));

    cancel_button.connect_clicked(move |_| on_cancel(current));

    default_button.connect_clicked(move |_| on_default(def));

    ok_button.connect_clicked(move |_| on_ok(adjustment.value()));

    return widget;
}
