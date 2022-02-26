use gtk::{prelude::WidgetExtManual, ContainerExt, DialogExt, GtkWindowExt, RangeExt, WidgetExt};

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
    let step = f64::min((max - min) / 10000., 1.);

    let scale = gtk::Scale::with_range(gtk::Orientation::Vertical, min, max, step);
    scale.set_size_request(64, 360);
    scale.set_inverted(true);
    scale.set_value(current);
    scale.connect_change_value(move |_, _, v| {
        fcv(v, true);
        gtk::Inhibit(false)
    });

    let dialog = gtk::Dialog::new();
    dialog.add_button("Cancel", gtk::ResponseType::Cancel);
    dialog.add_button("Default", gtk::ResponseType::Other(0));
    dialog.add_button("Ok", gtk::ResponseType::Ok);
    dialog.set_default_response(gtk::ResponseType::Ok);
    dialog.get_content_area().add(&scale);
    dialog.set_position(gtk::WindowPosition::Mouse);
    dialog.set_decorated(false);
    dialog.show_all();

    let mut res = current;

    match dialog.run() {
        gtk::ResponseType::Ok => {
            res = scale.get_value();
        }
        gtk::ResponseType::Other(0) => {
            res = def;
        }
        _ => println!("bounded_float_entry::run SUCK!"),
    }
    unsafe {
        dialog.destroy();
    }
    res
}
