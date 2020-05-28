use std::cell::RefCell;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::rc::Rc;

//use gdk::EventMask;
//use gio::prelude::*;
//use gtk::gtk_sys::GtkScrolledWindow;
use gtk::prelude::*;
use gtk::{DrawingArea, WidgetExt};

//use cairo::enums::{FontSlant, FontWeight};
use cairo::Context;

use talker::identifier::Id;
use talker::talker::{RTalker, Talker};

use session::event_bus::{Notification, REventBus};

use crate::mixer_control::MixerControl;
use crate::session_controler::RSessionControler;
use crate::talker_control::{RTalkerControl, TalkerControlBase};
//use crate::util;

const MARGE: f64 = 50.;
const PADDING: f64 = 5.;

struct ColumnProperty {
    start: f64,
    thickness: f64,
    count: i32,
}

impl ColumnProperty {
    pub fn new(start: f64, thickness: f64, count: i32) -> ColumnProperty {
        Self {
            start,
            thickness,
            count,
        }
    }
}
fn provide_column_property<'a>(
    n: i32,
    column_properties: &'a mut BTreeMap<i32, ColumnProperty>,
) -> &'a mut ColumnProperty {
    match column_properties.get(&n) {
        Some(cp) => &mut cp,
        None => {
            let mut cp = ColumnProperty::new(0., 0., 0);
            column_properties.insert(n, cp);
            &mut cp
        }
    }
}

fn provide_talker_control<'a>(
    tkr: &RTalker,
    talker_controls: &'a mut HashMap<Id, RTalkerControl>,
) -> (bool, &'a mut RTalkerControl) {
    match talker_controls.get(&tkr.borrow().id()) {
        Some(mut tkrc) => (false, &mut tkrc),
        None => {
            let tkrc = TalkerControlBase::new_ref(tkr);
            talker_controls.insert(tkrc.id(), tkrc);
            (true, &mut tkrc)
        }
    }
}
struct Collector {
    row: i32,
    column: i32,
    columns_properties: BTreeMap<i32, ColumnProperty>,
    talker_controls: HashMap<Id, RTalkerControl>,
}
impl Collector {
    pub fn new(row: i32, column: i32) -> Collector {
        Self {
            row,
            column,
            columns_properties: BTreeMap::new(),
            talker_controls: HashMap::new(),
        }
    }
}

pub struct GraphView {
    controler: RSessionControler,
    area: DrawingArea,
    talker_controls: HashMap<Id, RTalkerControl>,
}
pub type RGraphView = Rc<RefCell<GraphView>>;

impl GraphView {
    pub fn new_ref(controler: RSessionControler) -> RGraphView {
        let rgv = Rc::new(RefCell::new(Self {
            controler,
            area: DrawingArea::new(),
            talker_controls: HashMap::new(),
        }));
        GraphView::connect_area(&rgv, rgv.borrow().area());
        GraphView::observe(&rgv, controler.borrow().event_bus());

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
        let mut operated = false;

        for tkrc in self.talker_controls.values() {
            operated = tkrc.on_button_release(x, y, &self.controler);

            if operated {
                break;
            }
        }
        Inhibit(operated)
    }

    fn on_draw(&mut self, area: &DrawingArea, cr: &Context) -> Inhibit {
        let talkers = self.controler.borrow().session().talkers();

        for (id, tkrc) in self.talker_controls {
            match talkers.get(&id) {
                Some(talker) => {
                    tkrc.draw(area, cr, talker, &self.talker_controls);
                }
                None => (),
            }
        }
        /*
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
            }
            */
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

    fn column_layout(
        x0: f64,
        y0: f64,
        talker_controls: &mut HashMap<Id, RTalkerControl>,
        columns_properties: &mut BTreeMap<i32, ColumnProperty>,
    ) -> (f64, f64) {
        /* define columns thickness */
        for tkrc in talker_controls.values() {
            let column_property = provide_column_property(tkrc.column(), columns_properties);

            if column_property.thickness < tkrc.width() {
                column_property.thickness = tkrc.width();
            }
        }

        /* define graph width and row count */
        let mut w = 0.;
        let mut row_count = 0;

        for column_property in columns_properties.values() {
            w = w + column_property.thickness + MARGE;
            row_count = i32::max(row_count, column_property.count);
        }

        let mut prev_rows_y = vec![0.; row_count as usize];

        /* position GTalkers */
        let mut prev_x = w;
        let mut h = 0.;

        for (col_nbr, column_property) in columns_properties {
            let mut col_tkrcs: BTreeMap<i32, &RTalkerControl> = BTreeMap::new();

            for tkrc in talker_controls.values() {
                if tkrc.column() == *col_nbr {
                    col_tkrcs.insert(tkrc.row(), tkrc);
                }
            }

            let mut prev_row = -1;
            let mut prev_bottom = 0.;

            for tkrc in col_tkrcs.values() {
                let x = prev_x - (tkrc.width() + column_property.thickness) * 0.5;

                let mut y = prev_bottom;
                if tkrc.row() > prev_row + 1 {
                    y = f64::max(prev_bottom, prev_rows_y[tkrc.dependent_row() as usize]);
                }
                tkrc.move_to(x + x0, y + y0);

                prev_rows_y[tkrc.row() as usize] = y;
                prev_row = tkrc.row();
                prev_bottom = y + tkrc.height() + PADDING;
            }
            prev_x = prev_x - column_property.thickness - MARGE;
            h = f64::max(prev_bottom, h);
        }
        (w, h)
    }

    fn make_talker_controls<'a>(
        talker: &RTalker,
        collector: &'a mut Collector,
    ) -> Result<&'a mut Collector, failure::Error> {
        // create GTalkers and define there row and column
        if !talker.borrow().is_hidden() {
            //            let (row, column, columns_properties, talker_controls) = collector;

            let (is_new_talker_control, tkrc) =
                provide_talker_control(talker, &mut collector.talker_controls);

            let mut talks_count = 0;

            for ear in talker.borrow().ears() {
                talks_count = ear.fold_talks(|_, tc| Ok(tc + 1), talks_count)?;
            }

            let row = collector.row;
            let column = collector.column;

            if is_new_talker_control || (tkrc.column() < column && talks_count == 0) {
                tkrc.set_column(column);

                let column_property =
                    provide_column_property(column, &mut collector.columns_properties);
                let tkrc_row = i32::max(row, column_property.count);
                tkrc.set_row(tkrc_row);
                column_property.count = tkrc_row + 1;

                if is_new_talker_control {
                    tkrc.set_dependent_row(row);
                }

                // let dep_row = i32::max(0, tkrc_row - talks_count / 2);
                // let dep_column = column + 1;
                // let mut acc = (dep_row, dep_column, columns_properties, talker_controls);
                //        let mut dep_collector = Collector::new(i32::max(0, tkrc_row - talks_count / 2),collector.column + 1);
                collector.row = i32::max(0, tkrc_row - talks_count / 2);
                collector.column = collector.column + 1;

                for ear in talker.borrow().ears() {
                    collector = ear.fold_talkers(GraphView::make_talker_controls, collector)?;
                }
                collector.row = row;
                collector.column = column;
            }
        }
        Ok(collector)
    }

    fn create_graph(&mut self) -> Result<HashMap<Id, RTalkerControl>, failure::Error> {
        let mut collector = Collector::new(0, 1);
        //        let mut columns_properties: BTreeMap<i32, ColumnProperty> = BTreeMap::new();

        /* create graph by covering mixers */
        let mixers_column_property = ColumnProperty::new(
            0.,
            0.,
            self.controler.borrow().session().mixers().len() as i32,
        );
        collector
            .columns_properties
            .insert(0, mixers_column_property);

        for (row, (mxr_id, mixer)) in self
            .controler
            .borrow()
            .session()
            .mixers()
            .iter()
            .enumerate()
        {
            collector.row = row as i32;
            let mxrc = MixerControl::new(mixer, collector.row, 0);

            /* create GTalkers by covering talkers for each track */
            //            let mut acc = (row, 1, &columns_properties, &talker_controls);
            let mut acc = &mut collector;

            for track in mixer.borrow().tracks() {
                for ear in track.ears() {
                    acc = ear.fold_talkers(GraphView::make_talker_controls, acc)?;
                }
            }

            for ear in mixer.borrow().ears() {
                acc = ear.fold_talkers(
                    GraphView::make_talker_controls,
                    acc, //                    (row, 1, &columns_properties, &talker_controls),
                )?;
            }
            collector.talker_controls.insert(*mxr_id, Box::new(mxrc));
        }

        /* position GTalkers */
        let (w, h) = GraphView::column_layout(
            0.,
            0.,
            &mut collector.talker_controls,
            &mut collector.columns_properties,
        );

        /*********** SANDBOX ***********/
        /* create unused GTalkers */
        /*let (uW, uH) = positionUnusedTalkers MARGE (h +. MARGE) talker_controls canvas in*/

        /* list the unused talkers e.g not in the talker_controls list */
        let mut unused_talkers = Vec::new();

        for (id, tkr) in self.controler.borrow().session().talkers() {
            if !tkr.borrow().is_hidden() && !collector.talker_controls.contains_key(id) {
                unused_talkers.push(tkr);
            }
        }

        /* define the unused talkers reference count */
        let mut root_unused_talkers: BTreeMap<Id, &RTalker> = BTreeMap::new();

        for tkr in unused_talkers {
            root_unused_talkers.insert(tkr.borrow().id(), tkr);
        }

        for tkr in unused_talkers {
            for ear in tkr.borrow().ears() {
                let _ = ear.fold_talkers(
                    |dep_tkr, _| {
                        let _ = root_unused_talkers.remove(&dep_tkr.borrow().id());
                        Ok(())
                    },
                    (),
                );
            }
        }

        // let mut unused_talker_controls: HashMap<Id, RTalkerControl> = HashMap::new();
        // let mut sandbox_columns_properties: BTreeMap<i32, ColumnProperty> = BTreeMap::new();

        /* sort the root unused talkers in decreasing order
        in order to have the newest talker at the top of the sandbox */
        //        let mut acc = (0, 0, &sandbox_columns_properties, &unused_talker_controls);
        let mut sandbox_collector = Collector::new(0, 0);
        let mut acc = &mut sandbox_collector;

        for tkr in root_unused_talkers.values() {
            acc = GraphView::make_talker_controls(tkr, acc)?;
        }

        /* position unused GTalkers under used GTalkers e.g the sandbox zone */
        let (sandbox_w, sandbox_h) = GraphView::column_layout(
            MARGE,
            h + MARGE,
            &mut sandbox_collector.talker_controls,
            &mut sandbox_collector.columns_properties,
        );

        let mut talker_controls: HashMap<Id, RTalkerControl> = HashMap::new();
        talker_controls.extend(collector.talker_controls);
        talker_controls.extend(sandbox_collector.talker_controls);

        /* add GTracks in GTalkers list for connection and action */
        /*
                for mxrc in mxrcs {
                    for trkc in mxrc.tracks() {
                        talker_controls.insert(trkc.talker.borrow().id(), tkrc);
                    }
                }
        */
        /* draw connections */
        // for tkrc in talker_controls.values() {
        //     tkrc.draw_connections(&talker_controls);
        // }

        // canvas#set_scroll_region ~x1:0. ~y1:(-.PADDING)
        //   ~x2:((max w sandbox_w) +. PADDING) ~y2:(h +. MARGE +. sandbox_h +. PADDING);

        Ok(talker_controls)
    }

    fn build(&mut self) {
        match self.create_graph() {
            Ok(talker_controls) => {
                /*
                                for tkrc in talker_controls.values() {
                                    tkrc.set_actions(&self.controler.borrow());
                                }
                */
                self.talker_controls = talker_controls;
            }
            Err(e) => eprintln!("{}", e),
        }
    }

    fn observe(observer: &RGraphView, bus: &REventBus) {
        let obs = observer.clone();

        bus.borrow_mut()
            .add_observer(Box::new(move |notification| match notification {
                Notification::State(state) => match state {},
                Notification::Session => obs.borrow_mut().build(),
                Notification::TalkerChanged | Notification::TalkerRenamed(_) => {
                    obs.borrow_mut().build()
                }
                Notification::TalkerSelected(tkr_id) => {
                    if let Some(tkrc) = &obs.borrow_mut().talker_controls.get(tkr_id) {
                        tkrc.select()
                    }
                }
                Notification::TalkerUnselected(tkr_id) => {
                    if let Some(tkrc) = &obs.borrow_mut().talker_controls.get(tkr_id) {
                        tkrc.unselect()
                    }
                }
                Notification::EarSelected(tkr_id, idx) => {
                    if let Some(tkrc) = &obs.borrow_mut().talker_controls.get(tkr_id) {
                        tkrc.select_ear(*idx)
                    }
                }
                Notification::EarUnselected(tkr_id, idx) => {
                    if let Some(tkrc) = &obs.borrow_mut().talker_controls.get(tkr_id) {
                        tkrc.unselect_ear(*idx)
                    }
                }
                Notification::VoiceSelected(tkr_id, idx) => {
                    if let Some(tkrc) = &obs.borrow_mut().talker_controls.get(tkr_id) {
                        tkrc.select_voice(*idx)
                    }
                }
                Notification::VoiceUnselected(tkr_id, idx) => {
                    if let Some(tkrc) = &obs.borrow_mut().talker_controls.get(tkr_id) {
                        tkrc.unselect_voice(*idx)
                    }
                }
                Notification::NewTalker => obs.borrow_mut().build(),
                _ => (),
            }))
    }
}
