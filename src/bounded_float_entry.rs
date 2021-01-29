//#[macro_use]
use std::cell::RefCell;
use std::rc::Rc;

//use gdk::EventMask;
use gio::prelude::*;
//use gtk::gtk_sys::GtkScrolledWindow;
use gtk::prelude::*;
use gtk::{
    BoxExt, ButtonExt, CellRendererText, ContainerExt, GtkWindowExt, HeaderBarExt, IconSize,
    ScrolledWindowExt, TreeStore, TreeView, TreeViewColumn, WidgetExt,
};

use session::event_bus::{Notification, REventBus};
use session::state::State;

use crate::graph_view::GraphView;
use crate::session_presenter::RSessionPresenter;

pub fn create<F: Fn(f64, bool) + 'static>(min: f64, max: f64, current: f64, f: F) {
    let step = (max - min) / 200.;
    let scale = gtk::Scale::with_range(gtk::Orientation::Vertical, min, max, step);
    scale.set_inverted(true);
    scale.set_value(current);
    scale.connect_change_value(move |_, _, v| {
        f(v, true);
        Inhibit(false)
    });

    let window = gtk::Window::new(gtk::WindowType::Toplevel /*Popup*/);
    window.add(&scale);
    window.set_default_size(64, 256);
    window.show_all();
    /*
        window.connect_leave_notify_event(move |_, _| {
            //        f(v, false);
            //        window.destroy();
            Inhibit(true)
        });
    */
}
/*
pub struct BoundedFloatEntry {
    window: gtk::Window,
}

impl BoundedFloatEntry {
    pub fn new(min: f64, max: f64) -> BoundedFloatEntry {
        let step = (max - min) / 100.;
        let scale = gtk::Scale::with_range(gtk::Orientation::Vertical, min, max, step);
        scale.set_inverted(true);
        let window = gtk::Window::new(gtk::WindowType::Popup);
        window.add(&scale);
        window.set_default_size(64, 256);
        window.show_all();

        Self { window }
    }
    // TODO : On drop Widget::destroy
}
*/
