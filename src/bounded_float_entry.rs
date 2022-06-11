use crate::gtk::prelude::BoxExt;
use crate::gtk::prelude::ContainerExt;
use crate::gtk::prelude::DialogExt;
use crate::gtk::prelude::GtkWindowExt;
use crate::gtk::prelude::RangeExt;
use crate::gtk::prelude::ScaleExt;
use crate::gtk::prelude::WidgetExt;
use crate::gtk::prelude::WidgetExtManual;
use crate::gtk::Adjustment;
use crate::gtk::SpinButton;

pub fn create<Fcv: Fn(f64, bool) + 'static, Fend: Fn() + 'static>(
    min: f64,
    max: f64,
    current: f64,
    fcv: Fcv,
    fend: Fend,
) {
    let step = (max - min) / 200.;
    let scale = gtk::Scale::with_range(gtk::Orientation::Vertical, min, max, step);
    scale.set_size_request(64, 256);
    scale.set_inverted(true);
    scale.set_value(current);
    scale.connect_change_value(move |_, _, v| {
        fcv(v, true);
        gtk::Inhibit(false)
    });

    let window = gtk::Window::new(gtk::WindowType::Toplevel /*Popup*/);
    window.add(&scale);
    window.set_default_size(64, 256);
    window.set_position(gtk::WindowPosition::Mouse);
    window.connect_leave_notify_event(move |_, _| {
        fend();
        gtk::Inhibit(false)
    });
    window.show_all();
}

pub fn run<Fcv: Fn(f64, bool) + 'static>(
    min: f64,
    max: f64,
    def: f64,
    current: f64,
    fcv: Fcv,
) -> f64 {
    let step = f64::min((max - min) / 40000., 1.);
    let adjustment = Adjustment::new(current, min, max, step, step * 100., 0.);

    let scale = gtk::Scale::new(gtk::Orientation::Vertical, Some(&adjustment));
    scale.set_size_request(64, 360);
    scale.set_inverted(true);
    scale.set_draw_value(false);

    scale.connect_change_value(move |_, _, v| {
        fcv(v, true);
        gtk::Inhibit(false)
    });

    let entry = SpinButton::new(Some(&adjustment), 5., 4);
    entry.set_expand(false);

    // box
    let h_box = gtk::Box::new(gtk::Orientation::Horizontal, 2);
    h_box.pack_start(&scale, true, true, 0);
    h_box.pack_start(&entry, false, false, 0);

    let dialog = gtk::Dialog::new();
    dialog.add_button("Cancel", gtk::ResponseType::Cancel);
    dialog.add_button("Default", gtk::ResponseType::Other(0));
    dialog.add_button("Ok", gtk::ResponseType::Ok);
    dialog.set_default_response(gtk::ResponseType::Ok);
    dialog.content_area().add(&h_box);
    dialog.set_position(gtk::WindowPosition::Mouse);
    dialog.set_decorated(false);
    dialog.show_all();

    let mut res = current;

    match dialog.run() {
        gtk::ResponseType::Ok => {
            res = scale.value();
        }
        gtk::ResponseType::Other(0) => {
            res = def;
        }
        _ => (), // Cancel
    }
    unsafe {
        dialog.destroy();
    }
    res
}
