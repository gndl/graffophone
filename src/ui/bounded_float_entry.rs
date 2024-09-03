use std::str::FromStr;

use crate::gtk::Adjustment;
use gtk::glib::IsA;
use gtk::prelude::EditableExt;
use gtk::prelude::WidgetExt;
use gtk::traits::{AdjustmentExt, BoxExt, ButtonExt};

fn key_is_numeric(c: u32) -> bool {
    (c >= 10 && c <= 19) || (c >= 79 && c <= 91) || c == 59
}

pub fn create<
    OnValueChanged: Fn(f32) + 'static,
    OnOk: Fn(f32) + 'static,
    OnCancel: Fn(f32) + 'static,
    OnDefault: Fn(f32) + 'static,
>(
    min: f32,
    max: f32,
    def: f32,
    current: f32,
    on_value_changed: OnValueChanged,
    on_ok: OnOk,
    on_cancel: OnCancel,
    on_default: OnDefault,
) -> impl IsA<gtk::Widget> {
    let step = f32::min((max - min) / 40000., 1.);
    let adjustment = Adjustment::new(
        current as f64,
        min as f64,
        max as f64,
        step as f64,
        step as f64 * 100.,
        0.);

    let scale = gtk::Scale::builder()
        .adjustment(&adjustment)
        .orientation(gtk::Orientation::Vertical)
        .width_request(64)
        .height_request(360)
        .inverted(true)
        .draw_value(false)
        .build();
    
    let entry = gtk::Entry::builder().can_focus(false)
        .input_purpose(gtk::InputPurpose::Number)
        .css_classes(["bounded_float_entry"]).build();
    entry.set_text(&current.to_string());
    entry.select_region(0, -1);

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


    // Key pressed event
    let key_pressed_adjustment = adjustment.clone();
    let key_pressed_entry = entry.clone();
    let key_pressed_ok_button = ok_button.clone();

    let key_pressed_event_controller = gtk::EventControllerKey::builder().build();
    key_pressed_event_controller.connect_key_pressed(move |_, key, key_code, _| {

        let entry_value = key_pressed_entry.text();

        if key == gtk::gdk::Key::space || key == gtk::gdk::Key::Return || key == gtk::gdk::Key::KP_Enter {
            match f32::from_str(&entry_value) {
                Ok(v) => key_pressed_adjustment.set_value(v as f64),
                Err(_) => key_pressed_entry.set_text(&(key_pressed_adjustment.value() as f32).to_string()),
            }
            if key != gtk::gdk::Key::space {
                key_pressed_ok_button.emit_clicked();
            }
        }
        else if key == gtk::gdk::Key::BackSpace {
            if entry_value.len() > 0 {
                key_pressed_entry.set_text(entry_value.get(..entry_value.len() - 1).unwrap());
            }
        }
        else if key == gtk::gdk::Key::Delete {
            key_pressed_entry.set_text("");
        }
        else if key_is_numeric(key_code) {
            if let Some(car) = key.to_unicode() {

                let new_value = match key_pressed_entry.selection_bounds() {
                    Some((sel_start, sel_end)) => format!("{}{}{}",
                        entry_value.get(..sel_start as usize).unwrap(),
                        car,
                        entry_value.get(sel_end as usize..).unwrap()),
                    None => format!("{}{}", entry_value, car),
                };
                if new_value.matches(".").count() < 2 {
                    key_pressed_entry.set_text(&new_value);
                }
            }
        }

        return gtk::glib::signal::Propagation::Proceed;
    });

    widget.add_controller(key_pressed_event_controller);

    let adjustment_entry = entry.clone();

    adjustment.connect_value_changed(move |adj| {
        let v = adj.value() as f32;
        on_value_changed(v);
        adjustment_entry.set_text(&v.to_string());
        adjustment_entry.select_region(0, -1);
    });

    cancel_button.connect_clicked(move |_| on_cancel(current));

    default_button.connect_clicked(move |_| on_default(def));

    ok_button.connect_clicked(move |_| on_ok(adjustment.value() as f32));

    return widget;
}
