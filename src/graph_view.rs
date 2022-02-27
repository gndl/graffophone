use std::cell::RefCell;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::collections::HashSet;
use std::rc::Rc;

use gtk::prelude::*;
use gtk::{DrawingArea, WidgetExt};

use cairo::Context;

use talker::identifier::Id;
use talker::identifier::Identifiable;
use talker::talker::RTalker;
//use talker::talker::Talker;

use session::event_bus::{Notification, REventBus};
use session::mixer::Mixer;

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
    graph_presenter: RGraphPresenter,
    row: i32,
    column: i32,
    columns_properties: BTreeMap<i32, ColumnProperty>,
    talker_controls: HashMap<Id, RTalkerControl>,
    exclude_talkers_ids: Option<HashSet<Id>>,
}
impl<'c> Collector<'c> {
    pub fn new(
        control_supply: &'c ControlSupply,
        graph_presenter: &RGraphPresenter,
        row: i32,
        column: i32,
        exclude_talkers_ids: Option<HashSet<Id>>,
    ) -> Collector<'c> {
        Self {
            control_supply,
            graph_presenter: graph_presenter.clone(),
            row,
            column,
            columns_properties: BTreeMap::new(),
            talker_controls: HashMap::new(),
            exclude_talkers_ids,
        }
    }

    pub fn add_if_new(&mut self, talker: &RTalker) -> Result<bool, failure::Error> {
        let id = talker.id();

        if self.talker_controls.contains_key(&id) {
            Ok(false)
        } else {
            let minimized = self.graph_presenter.borrow().talker_minimized(id);
            self.talker_controls.insert(
                id,
                talker_control::new_ref(talker, self.control_supply, minimized)?,
            );
            Ok(true)
        }
    }
}

pub struct EventReceiver {
    session_presenter: RSessionPresenter,
    graph_presenter: RGraphPresenter,
    talker_controls: Vec<RTalkerControl>,
}
pub type REventReceiver = Rc<RefCell<EventReceiver>>;

impl EventReceiver {
    pub fn new_ref(
        session_presenter: &RSessionPresenter,
        graph_presenter: &RGraphPresenter,
    ) -> REventReceiver {
        Rc::new(RefCell::new(Self {
            session_presenter: session_presenter.clone(),
            graph_presenter: graph_presenter.clone(),
            talker_controls: Vec::new(),
        }))
    }
    pub fn set_talker_controls(&mut self, talker_controls: &HashMap<Id, RTalkerControl>) {
        self.talker_controls.clear();

        for tkrc in talker_controls.values() {
            self.talker_controls.push(tkrc.clone());
        }
    }

    pub fn on_button_release(&mut self, ev: &gdk::EventButton) -> Inhibit {
        let (x, y) = ev.get_position();

        for tkrc in &self.talker_controls {
            match tkrc.borrow().on_button_release(x, y, &self.graph_presenter) {
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
    graph_presenter: RGraphPresenter,
    drawing_area: DrawingArea,
    talker_controls: HashMap<Id, RTalkerControl>,
    width: f64,
    height: f64,
    build_needed: bool,
}
pub type RGraphView = Rc<RefCell<GraphView>>;

impl GraphView {
    pub fn new_ref(session_presenter: &RSessionPresenter) -> RGraphView {
        let graph_presenter = GraphPresenter::new_ref(session_presenter);

        let rgv = Rc::new(RefCell::new(Self {
            session_presenter: session_presenter.clone(),
            event_receiver: EventReceiver::new_ref(session_presenter, &graph_presenter),
            graph_presenter,
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
        if talker.is_hidden() {
            return Ok(());
        } else if let Some(exclude_talkers_ids) = &collector.exclude_talkers_ids {
            if exclude_talkers_ids.contains(&talker.id()) {
                return Ok(());
            }
        }

        let is_new_talker_control = collector.add_if_new(talker)?;
        let id = talker.id();

        if let Some(rtkrc) = &collector.talker_controls.get_mut(&id) {
            let mut talks_count = 0;

            for ear in talker.ears() {
                talks_count = ear.fold_talks(
                    |_, _, _, tlk, tc| {
                        if tlk.talker().is_hidden() {
                            Ok(tc)
                        } else {
                            Ok(tc + 1)
                        }
                    },
                    talks_count,
                )?;
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
                for ear in talker.ears() {
                    ear.iter_talkers(GraphView::make_talker_controls, collector)?;
                }
                collector.row = row;
                collector.column = column;
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
            let mut collector = Collector::new(control_supply, &self.graph_presenter, 0, 1, None);

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
                /*
                                for track in mixer.borrow().tracks() {
                                    for ear in track.borrow().ears() {
                                        ear.iter_talkers(GraphView::make_talker_controls, &mut collector)?;
                                    }
                                }
                */
                for ear in mixer.borrow().talker().ears() {
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
            let mut used_talkers: HashSet<Id> =
                HashSet::with_capacity(collector.talker_controls.len());

            for (id, tkr) in session.talkers() {
                if !tkr.is_hidden() && tkr.model() != Mixer::kind() {
                    if collector.talker_controls.contains_key(id) {
                        used_talkers.insert(*id);
                    } else {
                        unused_talkers.push(tkr);
                    }
                }
            }

            /* define the unused talkers reference count */
            let mut root_unused_talkers: BTreeMap<Id, &RTalker> = BTreeMap::new();

            for tkr in &unused_talkers {
                root_unused_talkers.insert(tkr.id(), tkr);
            }

            for tkr in &unused_talkers {
                for ear in tkr.ears() {
                    ear.iter_talkers(
                        |dep_tkr, _| {
                            let _ = root_unused_talkers.remove(&dep_tkr.id());
                            Ok(())
                        },
                        &mut (),
                    )?;
                }
            }

            /* sort the root unused talkers in decreasing order
            in order to have the newest talker at the top of the sandbox */
            let mut sandbox_collector = Collector::new(
                control_supply,
                &self.graph_presenter,
                0,
                0,
                Some(used_talkers),
            );

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

            self.width = f64::max(f64::max(graph_e_x, sandbox_e_x) + MARGE, 1024.);
            self.height = f64::max(sandbox_e_y, 768.);

            Ok(talker_controls)
        }
    }

    fn build(&mut self, drawing_area: &DrawingArea, cc: &Context) {
        let control_supply = ControlSupply::new(cc);

        match self.create_graph(drawing_area, &control_supply) {
            Ok(talker_controls) => {
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
            self.build(drawing_area, cc);
        }
        style::background(cc);
        cc.rectangle(0., 0., self.width, self.height);
        cc.fill();

        //        let graph_presenter = &self.event_receiver.borrow().graph_presenter;

        for (_, tkrc) in &self.talker_controls {
            tkrc.borrow().draw_connections(cc, &self.talker_controls);
        }

        for (_, tkrc) in &self.talker_controls {
            tkrc.borrow().draw(cc, &self.graph_presenter.borrow());
        }

        Inhibit(false)
    }

    fn observe(observer: &RGraphView, bus: &REventBus) {
        let obs = observer.clone();

        bus.borrow_mut()
            .add_observer(Box::new(move |notification| match notification {
                Notification::SelectionChanged => obs.borrow_mut().refresh(),
                Notification::Session | Notification::TalkerChanged | Notification::NewTalker => {
                    obs.borrow_mut().draw()
                }
                _ => (),
            }))
    }
}
