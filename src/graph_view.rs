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
use crate::session_presenter::RSessionPresenter;
use crate::talker_control;
use crate::talker_control::RTalkerControl;
//use crate::talker_control:: TalkerControlBase;
use crate::style;

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

fn visit_column_property<F, P, R>(
    n: i32,
    column_properties: &mut BTreeMap<i32, ColumnProperty>,
    mut f: F,
    p: P,
) -> R
where
    F: FnMut(&mut ColumnProperty, P) -> R,
{
    match column_properties.get_mut(&n) {
        Some(cp) => f(cp, p),
        None => {
            let mut cp = ColumnProperty::new(0., 0., 0);
            let r = f(&mut cp, p);
            column_properties.insert(n, cp);
            r
        }
    }
}

struct Collector<'c> {
    builder: &'c talker_control::Builder<'c>,
    row: i32,
    column: i32,
    columns_properties: BTreeMap<i32, ColumnProperty>,
    talker_controls: HashMap<Id, RTalkerControl>,
}
impl<'c> Collector<'c> {
    pub fn new(builder: &'c talker_control::Builder, row: i32, column: i32) -> Collector<'c> {
        Self {
            builder,
            row,
            column,
            columns_properties: BTreeMap::new(),
            talker_controls: HashMap::new(),
        }
    }

    pub fn add(&mut self, talker: &RTalker) -> Result<bool, failure::Error> {
        let id = talker.borrow().id();

        if self.talker_controls.contains_key(&id) {
            Ok(false)
        } else {
            self.talker_controls.insert(id, self.builder.build(talker)?);
            Ok(true)
        }
    }
}

pub struct GraphView {
    presenter: RSessionPresenter,
    drawing_area: DrawingArea,
    talker_controls: HashMap<Id, RTalkerControl>,
    width: f64,
    height: f64,
    build_needed: bool,
}
pub type RGraphView = Rc<RefCell<GraphView>>;

impl GraphView {
    pub fn new_ref(presenter: RSessionPresenter) -> RGraphView {
        let rgv = Rc::new(RefCell::new(Self {
            presenter,
            drawing_area: DrawingArea::new(),
            talker_controls: HashMap::new(),
            width: 0.,
            height: 0.,
            build_needed: true,
        }));
        GraphView::connect_drawing_area(&rgv, rgv.borrow().drawing_area());
        GraphView::observe(&rgv, rgv.borrow().presenter.borrow().event_bus());

        rgv
    }

    fn connect_drawing_area(rgraphview: &RGraphView, drawing_area: &DrawingArea) {
        drawing_area.add_events(
            // gdk::EventMask::KEY_PRESS_MASK |
            gdk::EventMask::BUTTON_PRESS_MASK | gdk::EventMask::BUTTON_RELEASE_MASK,
        );

        let rgv = rgraphview.clone();
        drawing_area
            .connect_button_release_event(move |w, ev| rgv.borrow_mut().on_button_release(w, ev));

        let rgv = rgraphview.clone();
        drawing_area.connect_draw(move |w, cc| rgv.borrow_mut().on_draw(w, cc));
    }

    pub fn drawing_area<'a>(&'a self) -> &'a DrawingArea {
        &self.drawing_area
    }

    pub fn draw(&mut self) {
        self.build_needed = true;
        self.drawing_area.queue_draw();
    }

    fn on_button_release(&mut self, _drawing_area: &DrawingArea, ev: &gdk::EventButton) -> Inhibit {
        let (x, y) = ev.get_position();
        let mut operated = false;

        for tkrc in self.talker_controls.values_mut() {
            operated = tkrc.borrow_mut().on_button_release(x, y, &self.presenter);

            if operated {
                break;
            }
        }
        Inhibit(operated)
    }

    fn column_layout(
        x0: f64,
        y0: f64,
        talker_controls: &mut HashMap<Id, RTalkerControl>,
        columns_properties: &mut BTreeMap<i32, ColumnProperty>,
    ) -> (f64, f64) {
        /* define columns thickness */
        for tkrc in talker_controls.values() {
            visit_column_property(
                tkrc.borrow().column(),
                columns_properties,
                |column_property, width| {
                    if column_property.thickness < width {
                        column_property.thickness = width;
                    }
                    ()
                },
                tkrc.borrow().width(),
            );
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
                if tkrc.borrow().column() == *col_nbr {
                    col_tkrcs.insert(tkrc.borrow().row(), tkrc);
                }
            }

            let mut prev_row = -1;
            let mut prev_bottom = 0.;

            for tkrc in col_tkrcs.values_mut() {
                let x = prev_x - (tkrc.borrow().width() + column_property.thickness) * 0.5;

                let mut y = prev_bottom;
                if tkrc.borrow().row() > prev_row + 1 {
                    y = f64::max(
                        prev_bottom,
                        prev_rows_y[tkrc.borrow().dependent_row() as usize],
                    );
                }
                tkrc.borrow_mut().move_to(x + x0, y + y0);

                prev_rows_y[tkrc.borrow().row() as usize] = y;
                prev_row = tkrc.borrow().row();
                prev_bottom = y + tkrc.borrow().height() + PADDING;
            }
            prev_x = prev_x - column_property.thickness - MARGE;
            h = f64::max(prev_bottom, h);
        }
        (w, h)
    }

    fn make_talker_controls(
        talker: &RTalker,
        collector: &mut Collector,
    ) -> Result<(), failure::Error> {
        // create GTalkers and define there row and column
        if !talker.borrow().is_hidden() {
            let is_new_talker_control = collector.add(talker)?;
            let id = talker.borrow().id();

            if let Some(tkrc) = &collector.talker_controls.get_mut(&id) {
                let mut talks_count = 0;

                for ear in talker.borrow().ears() {
                    talks_count = ear.fold_talks(|_, tc| Ok(tc + 1), talks_count)?;
                }

                let row = collector.row;
                let column = collector.column;

                if is_new_talker_control || (tkrc.borrow().column() < column && talks_count == 0) {
                    tkrc.borrow_mut().set_column(column);

                    let tkrc_row = visit_column_property(
                        column,
                        &mut collector.columns_properties,
                        |column_property, row| {
                            let tkrc_row = i32::max(row, column_property.count);
                            column_property.count = tkrc_row + 1;
                            tkrc_row
                        },
                        row,
                    );
                    tkrc.borrow_mut().set_row(tkrc_row);

                    if is_new_talker_control {
                        tkrc.borrow_mut().set_dependent_row(row);
                    }

                    // let dep_row = i32::max(0, tkrc_row - talks_count / 2);
                    // let dep_column = column + 1;
                    // let mut acc = (dep_row, dep_column, columns_properties, talker_controls);
                    // let mut dep_collector = Collector::new(i32::max(0, tkrc_row - talks_count / 2),collector.column + 1);
                    collector.row = i32::max(0, tkrc_row - talks_count / 2);
                    collector.column = collector.column + 1;

                    for ear in talker.borrow().ears() {
                        ear.iter_talkers(GraphView::make_talker_controls, collector)?;
                    }
                    collector.row = row;
                    collector.column = column;
                }
            }
        }
        Ok(())
    }

    fn create_graph(
        &mut self,
        drawing_area: &DrawingArea,
        builder: &talker_control::Builder,
    ) -> Result<HashMap<Id, RTalkerControl>, failure::Error> {
        let presenter = self.presenter.borrow();

        let mut collector = Collector::new(builder, 0, 1);
        // let mut columns_properties: BTreeMap<i32, ColumnProperty> = BTreeMap::new();

        /* create graph by covering mixers */
        let mixers_column_property =
            ColumnProperty::new(0., 0., presenter.session().mixers().len() as i32);
        collector
            .columns_properties
            .insert(0, mixers_column_property);

        for (row, (mxr_id, mixer)) in self
            .presenter
            .borrow()
            .session()
            .mixers()
            .iter()
            .enumerate()
        {
            collector.row = row as i32;
            let mxrc = MixerControl::new_ref(mixer, collector.row, 0);

            /* create GTalkers by covering talkers for each track */
            //            let mut acc = (row, 1, &columns_properties, &talker_controls);
            //            let mut acc = &mut collector;

            for track in mixer.borrow().tracks() {
                for ear in track.ears() {
                    //                    acc =
                    ear.iter_talkers(GraphView::make_talker_controls, &mut collector)?;
                }
            }

            for ear in mixer.borrow().ears() {
                //                acc =
                ear.iter_talkers(
                    GraphView::make_talker_controls,
                    &mut collector, //                    (row, 1, &columns_properties, &talker_controls),
                )?;
            }
            collector.talker_controls.insert(*mxr_id, mxrc);
        }

        /* position GTalkers */
        let (graph_w, graph_h) = GraphView::column_layout(
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

        for (id, tkr) in presenter.session().talkers() {
            if !tkr.borrow().is_hidden() && !collector.talker_controls.contains_key(id) {
                unused_talkers.push(tkr);
            }
        }

        /* define the unused talkers reference count */
        let mut root_unused_talkers: BTreeMap<Id, &RTalker> = BTreeMap::new();

        for tkr in &unused_talkers {
            root_unused_talkers.insert(tkr.borrow().id(), tkr);
        }

        for tkr in &unused_talkers {
            for ear in tkr.borrow().ears() {
                ear.iter_talkers(
                    |dep_tkr, _| {
                        let _ = root_unused_talkers.remove(&dep_tkr.borrow().id());
                        Ok(())
                    },
                    &mut (),
                )?;
            }
        }

        // let mut unused_talker_controls: HashMap<Id, RTalkerControl> = HashMap::new();
        // let mut sandbox_columns_properties: BTreeMap<i32, ColumnProperty> = BTreeMap::new();

        /* sort the root unused talkers in decreasing order
        in order to have the newest talker at the top of the sandbox */
        //        let mut acc = (0, 0, &sandbox_columns_properties, &unused_talker_controls);
        let mut sandbox_collector = Collector::new(builder, 0, 0);
        //        let mut acc = &mut sandbox_collector;

        for tkr in root_unused_talkers.values() {
            //            acc =
            GraphView::make_talker_controls(tkr, &mut sandbox_collector)?;
        }

        /* position unused GTalkers under used GTalkers e.g the sandbox zone */
        let (sandbox_w, sandbox_h) = GraphView::column_layout(
            MARGE,
            graph_h + MARGE,
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
        //     tkrc.borrow().draw_connections(&talker_controls);
        // }

        // canvas#set_scroll_region ~x1:0. ~y1:(-.PADDING)
        //   ~x2:((max graph_w sandbox_w) +. PADDING) ~y2:(graph_h +. MARGE +. sandbox_h +. PADDING);

        self.width = MARGE + f64::max(graph_w, sandbox_w) + MARGE;
        self.height = graph_h + MARGE + sandbox_h + PADDING;

        Ok(talker_controls)
    }

    fn build(&mut self, drawing_area: &DrawingArea, cc: &Context) {
        let builder = talker_control::Builder::new(cc);

        match self.create_graph(drawing_area, &builder) {
            Ok(talker_controls) => {
                /*
                                for tkrc in talker_controls.values() {
                                    tkrc.borrow().set_actions(&self.presenter.borrow());
                                }
                */
                self.talker_controls = talker_controls;
                drawing_area.set_size_request(self.width as i32, self.height as i32);
                self.build_needed = false;
            }
            Err(e) => eprintln!("{}", e),
        }
    }

    fn on_draw(&mut self, drawing_area: &DrawingArea, cc: &Context) -> Inhibit {
        if self.build_needed {
            println!("GraphView::on_draw with build needed");

            self.build(drawing_area, cc);
        } else {
            println!("GraphView::on_draw without build needed");
        }
        style::background(cc);
        cc.rectangle(0., 0., self.width, self.height);
        cc.fill();

        let presenter = self.presenter.borrow();
        let talkers = presenter.session().talkers();

        for (id, tkrc) in &self.talker_controls {
            match talkers.get(&id) {
                Some(talker) => {
                    tkrc.borrow()
                        .draw(drawing_area, cc, talker, &self.talker_controls);
                }
                None => (),
            }
        }
        /*
            //    cc.scale(1000f64, 1000f64);
            //    cc.select_font_face("Sans", FontSlant::Normal, FontWeight::Normal);
            cc.set_font_size(12.);

            // let w = extents.width + 20.;
            // let h = extents.height + 20.;
            // drawing_area.set_size_request(w as i32, h as i32);
            //        let (w0, h0) = drawing_area.get_size_request();
            let w = 2048;
            let h = 1024;
            drawing_area.set_size_request(w, h);
            //    let (w, h) = drawing_area.get_size_request();

            let mut x = 10.;
            let mut y = 10.;

            for talker in self.presenter.borrow().talkers() {
                let p = cc.text_extents(talker);

                x = x + 10.;
                y = y + p.height + 10.;

                cc.move_to(x, y);
                cc.show_text(talker);

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

    fn observe(observer: &RGraphView, bus: &REventBus) {
        let obs = observer.clone();

        bus.borrow_mut()
            .add_observer(Box::new(move |notification| match notification {
                //                Notification::State(state) => match state {},
                Notification::Session => obs.borrow_mut().draw(),
                Notification::TalkerChanged | Notification::TalkerRenamed(_) => {
                    obs.borrow_mut().draw()
                }
                Notification::TalkerSelected(tkr_id) => {
                    if let Some(tkrc) = &obs.borrow_mut().talker_controls.get(tkr_id) {
                        tkrc.borrow_mut().select()
                    }
                }
                Notification::TalkerUnselected(tkr_id) => {
                    if let Some(tkrc) = &obs.borrow_mut().talker_controls.get(tkr_id) {
                        tkrc.borrow_mut().unselect()
                    }
                }
                Notification::EarSelected(tkr_id, idx) => {
                    if let Some(tkrc) = &obs.borrow_mut().talker_controls.get(tkr_id) {
                        tkrc.borrow_mut().select_ear(*idx)
                    }
                }
                Notification::EarUnselected(tkr_id, idx) => {
                    if let Some(tkrc) = &obs.borrow_mut().talker_controls.get(tkr_id) {
                        tkrc.borrow_mut().unselect_ear(*idx)
                    }
                }
                Notification::VoiceSelected(tkr_id, idx) => {
                    if let Some(tkrc) = &obs.borrow_mut().talker_controls.get(tkr_id) {
                        tkrc.borrow_mut().select_voice(*idx)
                    }
                }
                Notification::VoiceUnselected(tkr_id, idx) => {
                    if let Some(tkrc) = &obs.borrow_mut().talker_controls.get(tkr_id) {
                        tkrc.borrow_mut().unselect_voice(*idx)
                    }
                }
                Notification::NewTalker => obs.borrow_mut().draw(),
                _ => (),
            }))
    }
}
