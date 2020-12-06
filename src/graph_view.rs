use std::cell::RefCell;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::rc::Rc;

//use gdk::EventMask;
//use gio::prelude::*;
//use gtk::gtk_sys::GtkScrolledWindow;
use gtk::prelude::*;
use gtk::{DrawingArea, WidgetExt};

use cairo::Context;

use talker::identifier::Id;
use talker::talker::{RTalker, Talker};

use session::event_bus::{Notification, REventBus};

use crate::graph_presenter::{GraphPresenter, RGraphPresenter};
use crate::mixer_control::MixerControl;
use crate::session_presenter::RSessionPresenter;
use crate::style;
use crate::talker_control;
use crate::talker_control::{ControlSupply, RTalkerControl};

const MARGE: f64 = 10.;
const ROW_SPACING: f64 = 5.;
const COLUMN_SPACING: f64 = 50.;

struct ColumnProperty {
    start: f64, // TODO : remove
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
    control_supply: &'c ControlSupply<'c>,
    row: i32,
    column: i32,
    columns_properties: BTreeMap<i32, ColumnProperty>,
    talker_controls: HashMap<Id, RTalkerControl>,
}
impl<'c> Collector<'c> {
    pub fn new(control_supply: &'c ControlSupply, row: i32, column: i32) -> Collector<'c> {
        Self {
            control_supply,
            row,
            column,
            columns_properties: BTreeMap::new(),
            talker_controls: HashMap::new(),
        }
    }

    pub fn add_if_new(&mut self, talker: &RTalker) -> Result<bool, failure::Error> {
        let id = talker.borrow().id();

        if self.talker_controls.contains_key(&id) {
            Ok(false)
        } else {
            self.talker_controls
                .insert(id, talker_control::new_ref(talker, self.control_supply)?);
            Ok(true)
        }
    }
}

pub struct EventReceiver {
    session_presenter: RSessionPresenter,
    graph_presenter: GraphPresenter,
    talker_controls: Vec<RTalkerControl>,
}
pub type REventReceiver = Rc<RefCell<EventReceiver>>;

impl EventReceiver {
    pub fn new_ref(session_presenter: &RSessionPresenter) -> REventReceiver {
        Rc::new(RefCell::new(Self {
            session_presenter: session_presenter.clone(),
            graph_presenter: GraphPresenter::new(session_presenter),
            talker_controls: Vec::new(),
        }))
    }
    pub fn set_talker_controls(&mut self, talker_controls: &HashMap<Id, RTalkerControl>) {
        self.talker_controls.clear();

        for tkrc in talker_controls.values() {
            self.talker_controls.push(tkrc.clone());
        }
    }

    pub fn add_talker_control(&mut self, talker_control: &RTalkerControl) {
        self.talker_controls.push(talker_control.clone());
    }

    pub fn on_button_release(&mut self, ev: &gdk::EventButton) -> Inhibit {
        let (x, y) = ev.get_position();

        for tkrc in &self.talker_controls {
            match tkrc
                .borrow()
                .on_button_release(x, y, &mut self.graph_presenter)
            {
                Ok(None) => (),
                Ok(Some(notifications)) => {
                    for notification in notifications {
                        self.session_presenter.borrow().notify(notification);
                    }
                    return Inhibit(true);
                }
                Err(e) => self.session_presenter.borrow().notify_error(e),
            }
        }
        Inhibit(false)
    }
}

pub struct GraphView {
    session_presenter: RSessionPresenter,
    event_receiver: REventReceiver,
    drawing_area: DrawingArea,
    talker_controls: HashMap<Id, RTalkerControl>,
    width: f64,
    height: f64,
    build_needed: bool,
}
pub type RGraphView = Rc<RefCell<GraphView>>;

impl GraphView {
    pub fn new_ref(session_presenter: &RSessionPresenter) -> RGraphView {
        let rgv = Rc::new(RefCell::new(Self {
            session_presenter: session_presenter.clone(),
            event_receiver: EventReceiver::new_ref(session_presenter),
            drawing_area: DrawingArea::new(),
            talker_controls: HashMap::new(),
            width: 0.,
            height: 0.,
            build_needed: true,
        }));
        GraphView::connect_drawing_area(&rgv, rgv.borrow().drawing_area());
        GraphView::observe(&rgv, session_presenter.borrow().event_bus());

        rgv
    }

    fn connect_drawing_area(rgraphview: &RGraphView, drawing_area: &DrawingArea) {
        drawing_area.add_events(
            // gdk::EventMask::KEY_PRESS_MASK |
            gdk::EventMask::BUTTON_PRESS_MASK | gdk::EventMask::BUTTON_RELEASE_MASK,
        );

        let er = rgraphview.borrow().event_receiver.clone();
        drawing_area
            .connect_button_release_event(move |_, ev| er.borrow_mut().on_button_release(ev));

        let gv_drawer = rgraphview.clone();
        drawing_area.connect_draw(move |w, cc| gv_drawer.borrow_mut().on_draw(w, cc));
    }

    pub fn drawing_area<'a>(&'a self) -> &'a DrawingArea {
        &self.drawing_area
    }

    pub fn draw(&mut self) {
        self.build_needed = true;
        self.drawing_area.queue_draw();
    }

    pub fn refresh(&self) {
        self.drawing_area.queue_draw();
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
        let mut w = x0;
        let mut row_count = 0;

        for column_property in columns_properties.values() {
            w = w + column_property.thickness + COLUMN_SPACING;
            row_count = i32::max(row_count, column_property.count);
        }

        let mut prev_rows_y = vec![0.; row_count as usize];

        /* position GTalkers */
        let mut prev_x = w;
        let mut h = 0.;

        for (col_nbr, column_property) in columns_properties {
            let mut col_tkrcs: BTreeMap<i32, &RTalkerControl> = BTreeMap::new();

            for rtkrc in talker_controls.values() {
                let tkrc = rtkrc.borrow();

                if tkrc.column() == *col_nbr {
                    col_tkrcs.insert(tkrc.row(), rtkrc);
                }
            }

            let mut prev_row = -1;
            let mut prev_bottom = y0;

            for rtkrc in col_tkrcs.values_mut() {
                let mut tkrc = rtkrc.borrow_mut();

                let x = prev_x - ((tkrc.width() + column_property.thickness) * 0.5);

                let mut y = prev_bottom;
                if tkrc.row() > prev_row + 1 {
                    y = f64::max(prev_bottom, prev_rows_y[tkrc.dependent_row() as usize]);
                }
                tkrc.move_to(x, y);

                prev_rows_y[tkrc.row() as usize] = y;
                prev_row = tkrc.row();
                prev_bottom = y + tkrc.height() + ROW_SPACING;
            }
            prev_x = prev_x - column_property.thickness - COLUMN_SPACING;
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
            let is_new_talker_control = collector.add_if_new(talker)?;
            let id = talker.borrow().id();

            if let Some(rtkrc) = &collector.talker_controls.get_mut(&id) {
                let mut talks_count = 0;

                for ear in talker.borrow().ears() {
                    talks_count = ear.fold_talks(|_, tc| Ok(tc + 1), talks_count)?;
                }

                let row = collector.row;
                let column = collector.column;

                if is_new_talker_control || (rtkrc.borrow().column() < column && talks_count == 0) {
                    {
                        let mut tkrc = rtkrc.borrow_mut();
                        tkrc.set_column(column);

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
                        tkrc.set_row(tkrc_row);

                        if is_new_talker_control {
                            tkrc.set_dependent_row(row);
                        }

                        collector.row = i32::max(0, tkrc_row - talks_count / 2);
                        collector.column = collector.column + 1;
                    }
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
        control_supply: &ControlSupply,
    ) -> Result<HashMap<Id, RTalkerControl>, failure::Error> {
        let session_presenter = self.session_presenter.borrow();
        let session = session_presenter.session();
        {
            let mut collector = Collector::new(control_supply, 0, 1);

            /* create graph by covering mixers */
            let mixers_column_property = ColumnProperty::new(0., 0., session.mixers().len() as i32);
            collector
                .columns_properties
                .insert(0, mixers_column_property);

            for (row, (mxr_id, mixer)) in session.mixers().iter().enumerate() {
                collector.row = row as i32;
                let mxrc = MixerControl::new_ref(mixer, control_supply)?;

                mxrc.borrow_mut().set_row(collector.row);
                mxrc.borrow_mut().set_column(0);

                /* create GTalkers by covering talkers for each track */
                for track in mixer.borrow().tracks() {
                    for ear in track.borrow().ears() {
                        ear.iter_talkers(GraphView::make_talker_controls, &mut collector)?;
                    }
                }

                for ear in mixer.borrow().ears() {
                    ear.iter_talkers(GraphView::make_talker_controls, &mut collector)?;
                }
                collector.talker_controls.insert(*mxr_id, mxrc);
            }

            /* position GTalkers */
            let (graph_e_x, graph_e_y) = GraphView::column_layout(
                MARGE,
                MARGE,
                &mut collector.talker_controls,
                &mut collector.columns_properties,
            );

            /*********** SANDBOX ***********/
            /* create unused GTalkers */
            /* list the unused talkers e.g not in the talker_controls list */
            let mut unused_talkers = Vec::new();

            for (id, tkr) in session.talkers() {
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

            /* sort the root unused talkers in decreasing order
            in order to have the newest talker at the top of the sandbox */
            let mut sandbox_collector = Collector::new(control_supply, 0, 0);

            for tkr in root_unused_talkers.values() {
                GraphView::make_talker_controls(tkr, &mut sandbox_collector)?;
            }

            /* position unused GTalkers under used GTalkers e.g the sandbox zone */
            let (sandbox_e_x, sandbox_e_y) = GraphView::column_layout(
                MARGE,
                graph_e_y + MARGE,
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
                                            talker_controls.insert(trkc.borrow().id(), tkrc);
                                        }
                                    }
            */
            /* draw connections */
            // for tkrc in talker_controls.values() {
            //     tkrc.borrow().draw_connections(&talker_controls);
            // }

            self.width = f64::max(f64::max(graph_e_x, sandbox_e_x) + MARGE, 1024.);
            self.height = f64::max(sandbox_e_y, 768.);

            Ok(talker_controls)
        }
    }

    fn build(&mut self, drawing_area: &DrawingArea, cc: &Context) {
        let control_supply = ControlSupply::new(cc);

        match self.create_graph(drawing_area, &control_supply) {
            Ok(talker_controls) => {
                /*
                               for tkrc in talker_controls.values() {
                                   tkrc.borrow().set_actions(&self.session_presenter.borrow());
                               }
                */
                self.event_receiver
                    .borrow_mut()
                    .set_talker_controls(&talker_controls);

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

        let graph_presenter = &self.event_receiver.borrow().graph_presenter;

        for (_, tkrc) in &self.talker_controls {
            tkrc.borrow()
                .draw(cc, graph_presenter, &self.talker_controls);
        }

        Inhibit(false)
    }

    fn observe(observer: &RGraphView, bus: &REventBus) {
        let obs = observer.clone();

        bus.borrow_mut()
            .add_observer(Box::new(move |notification| match notification {
                //                Notification::State(state) => match state {},
                Notification::SelectionChanged => obs.borrow_mut().refresh(),
                Notification::Session | Notification::TalkerChanged | Notification::NewTalker => {
                    obs.borrow_mut().draw()
                }
                Notification::TalkerSelected(tkr_id) => {
                    if let Some(tkrc) = &obs.borrow().talker_controls.get(tkr_id) {
                        tkrc.borrow().select()
                    }
                }
                Notification::TalkerUnselected(tkr_id) => {
                    if let Some(tkrc) = &obs.borrow().talker_controls.get(tkr_id) {
                        tkrc.borrow().unselect()
                    }
                }
                Notification::EarSelected(tkr_id, ear_idx, talk_idx) => {
                    if let Some(tkrc) = &obs.borrow().talker_controls.get(tkr_id) {
                        tkrc.borrow().select_ear(*ear_idx, *talk_idx)
                    }
                }
                Notification::EarUnselected(tkr_id, ear_idx, talk_idx) => {
                    if let Some(tkrc) = &obs.borrow().talker_controls.get(tkr_id) {
                        tkrc.borrow().unselect_ear(*ear_idx, *talk_idx)
                    }
                }
                Notification::VoiceSelected(tkr_id, idx) => {
                    if let Some(tkrc) = &obs.borrow().talker_controls.get(tkr_id) {
                        tkrc.borrow().select_voice(*idx)
                    }
                }
                Notification::VoiceUnselected(tkr_id, idx) => {
                    if let Some(tkrc) = &obs.borrow().talker_controls.get(tkr_id) {
                        tkrc.borrow().unselect_voice(*idx)
                    }
                }
                _ => (),
            }))
    }
}
