use std::cell::RefCell;
use std::rc::Rc;

//use gdk::EventMask;
//use gio::prelude::*;
//use gtk::gtk_sys::GtkScrolledWindow;
use gtk::prelude::*;
use gtk::{DrawingArea, WidgetExt};

//use cairo::enums::{FontSlant, FontWeight};
use cairo::Context;

use crate::session_controler::RSessionControler;

struct Control {
    b_x: f64,
    e_x: f64,
    b_y: f64,
    e_y: f64,
    action: String,
}

pub struct GraphView {
    controler: RSessionControler,
    area: DrawingArea,
    controls: Vec<Control>,
}
pub type RGraphView = Rc<RefCell<GraphView>>;

impl GraphView {
    pub fn new_ref(controler: RSessionControler) -> RGraphView {
        let rgv = Rc::new(RefCell::new(Self {
            controler,
            area: DrawingArea::new(),
            controls: Vec::new(),
        }));
        GraphView::connect_area(&rgv, rgv.borrow().area());

        rgv
    }

    fn connect_area(rgraphview: &RGraphView, area: &DrawingArea) {
        area.add_events(
            // gdk::EventMask::KEY_PRESS_MASK |
            gdk::EventMask::BUTTON_PRESS_MASK | gdk::EventMask::BUTTON_RELEASE_MASK,
        );

        //            area.set_can_focus(true);
        // area.connect_key_press_event(|_, ev| {
        //     println!("Key {} pressed", ev.get_keyval());
        //     Inhibit(false)
        // });
        let rgv = rgraphview.clone();
        area.connect_button_release_event(move |w, ev| rgv.borrow_mut().on_button_release(w, ev));

        let rgv = rgraphview.clone();
        area.connect_draw(move |w, cr| rgv.borrow_mut().on_draw(w, cr));
    }

    pub fn area<'a>(&'a self) -> &'a DrawingArea {
        &self.area
    }

    pub fn draw(&self) {
        self.area.queue_draw();
    }

    fn on_button_release(&mut self, _area: &DrawingArea, ev: &gdk::EventButton) -> Inhibit {
        let (x, y) = ev.get_position();

        for c in self.controls.iter().rev() {
            if x >= c.b_x && x <= c.e_x && y >= c.b_y && y <= c.e_y {
                println!("Talker {} cliked at x = {}, y = {}!", c.action, x, y);
                return Inhibit(true);
            }
        }
        Inhibit(false)
    }

    fn on_draw(&mut self, area: &DrawingArea, cr: &Context) -> Inhibit {
        //    cr.scale(1000f64, 1000f64);

        //    cr.select_font_face("Sans", FontSlant::Normal, FontWeight::Normal);
        cr.set_font_size(12.);

        // let w = extents.width + 20.;
        // let h = extents.height + 20.;
        // area.set_size_request(w as i32, h as i32);
        //        let (w0, h0) = area.get_size_request();
        let w = 2048;
        let h = 1024;
        area.set_size_request(w, h);
        //    let (w, h) = area.get_size_request();

        self.controls.clear();
        let mut x = 10.;
        let mut y = 10.;

        for talker in self.controler.borrow().talkers() {
            let p = cr.text_extents(talker);

            x = x + 10.;
            y = y + p.height + 10.;

            cr.move_to(x, y);
            cr.show_text(talker);

            println!(
        "Talker {} :\n x_bearing {}, y_bearing {}, width {}, height {}, x_advance {}, y_advance {}", talker,
        p.x_bearing,
        p.y_bearing,
        p.width,
        p.height,
        p.x_advance,
        p.y_advance
    );

            self.controls.push(Control {
                b_x: x,
                e_x: x + p.width,
                b_y: y - p.height,
                e_y: y,
                action: talker.to_string(),
            });
        }
        Inhibit(false)
    }
    /*
    fn draw_graph(area: &DrawingArea, cr: &Context, text: &str) -> Inhibit {
        let w = 2048;
        let h = 1024;
        area.set_size_request(w, h);
        cr.set_line_width(5.);
        cr.set_source_rgb(0., 0., 0.);
        cr.rectangle(5., 5., 2038., 1014.);
        cr.stroke();
        Inhibit(false)
    }
    */
}
