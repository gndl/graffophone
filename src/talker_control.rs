use std::boxed::Box;
//use std::cell::Cell;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

//use gdk::EventMask;
//use gio::prelude::*;
//use gtk::gtk_sys::GtkScrolledWindow;
//use gtk::prelude::*;
use gtk::DrawingArea;

//use cairo::enums::{FontSlant, FontWeight};
use cairo::Context;

use talker::identifier::Identifiable;
use talker::identifier::{Id, Index};
use talker::talker::{RTalker, TalkerBase};

use crate::session_controler::RSessionControler;

struct Area {
    b_x: f64,
    e_x: f64,
    b_y: f64,
    e_y: f64,
    selected: bool,
}
impl Area {
    pub fn new(b_x: f64, e_x: f64, b_y: f64, e_y: f64) -> Area {
        Self {
            b_x,
            e_x,
            b_y,
            e_y,
            selected: false,
        }
    }
    pub fn is_under(&self, x: f64, y: f64) -> bool {
        x >= self.b_x && x <= self.e_x && y >= self.b_y && y <= self.e_y
    }
}

struct Control {
    area: Area,
    action: String,
}

pub enum EarType {
    Value(),
    Talk(),
    Add,
}

struct Ear {
    area: Area,
    value_area: Area,
    add_area: Area,
    sup_area: Area,
    y: f64,
    ear_type: EarType,
    root_index: i32,
}

struct Voice {
    area: Area,
    y: f64,
    color: i64,
}

pub struct TalkerControlBase {
    id: Id,
    area: Area,
    pub row: i32,
    pub column: i32,
    dependent_row: i32,
    width: f64,
    height: f64,
    box_top: f64,
    model_area: Option<Area>,
    name_area: Option<Area>,
    main_value_area: Option<Area>,
    box_area: Option<Area>,
    ears: Vec<Ear>,
    voices: Vec<Voice>,
}

pub type RTalkerControlBase = Rc<RefCell<TalkerControlBase>>;

impl TalkerControlBase {
    pub fn new(talker_base: &TalkerBase) -> TalkerControlBase {
        Self {
            id: talker_base.id(),
            area: Area::new(0., 20., 0., 40.),
            row: -1,
            column: -1,
            dependent_row: -1,
            width: 0.,
            height: 0.,
            box_top: 0.,
            model_area: None,
            name_area: None,
            main_value_area: None,
            box_area: None,
            ears: Vec::new(),
            voices: Vec::new(),
        }
    }
    pub fn new_ref(talker: &RTalker) -> RTalkerControl {
        Box::new(TalkerControlBase::new(talker.borrow().base()))
    }
    /*                           MODEL
                    _______________________________
                   |              NAME             |
                   |            [VALUE]            |
                   |TAG_INPUT_1 [1]  [TAG_OUTPUT_1]|
                   |TAG_INPUT_2 [2]                |
                   |TAG_INPUT_3 [3]  [TAG_OUTPUT_2]|
                   |_______________________________|
    */
    fn draw_header(
        &self,
        drawing_area: &DrawingArea,
        cr: &Context,
        talker: &RTalker,
        py: f64,
        draw_model: bool,
        draw_name: bool,
        draw_main_value: bool,
    ) {
        /*      mBoxTop <- if draw_model{
                  mKindItem <- Some(GnoCanvas.text ~text:talker#getKind ~y:pY
                                      ~props:modelProperties ~anchor: `NORTH mGroup);

                  pY +. textHeight +. boxRadius
              }
                 else{ pY +. boxRadius;}

              let mainValueY = if drawName && talker#getName <> "" then (
                  let text = formatName talker#getName in

                  let nameItem = GnoCanvas.text ~text ~y:mBoxTop
                      ~props:nameProperties ~anchor: `NORTH mGroup in

                  mWidth <- nameItem#text_width;
                  self#setNameItem nameItem;
                  mBoxTop +. textHeight
                )
                else mBoxTop in

              let mainValueText = formatValue talker#getStringOfValue in

              mHeight <- if drawMainValue && S.length mainValueText > 0 then (

                  let mainValueItem = GnoCanvas.text ~text:mainValueText
                      ~y:mainValueY ~props:valueProperties ~anchor: `NORTH mGroup in

                  mMainValueItem <- Some mainValueItem;
                  mWidth <- max mWidth mainValueItem#text_width;
                  mainValueY +. textHeight -. pY
                )
                else mainValueY -. pY;
        */
    }

    fn draw_ears_and_voices(
        &self,
        drawing_area: &DrawingArea,
        cr: &Context,
        talker: &RTalker,
        py: f64,
    ) {
    }

    fn draw_box(
        &self,
        drawing_area: &DrawingArea,
        cr: &Context,
        talker: &RTalker,
        px: f64,
        py: f64,
    ) {
        let w = self.area.e_x - self.area.b_x;
        let h = self.area.e_y - self.area.b_y;
        cr.set_line_width(5.);
        cr.set_source_rgb(0., 0., 0.);
        cr.rectangle(self.area.b_x, self.area.b_y, w, h);
        cr.stroke();
        //    cr.select_font_face("Sans", FontSlant::Normal, FontWeight::Normal);
        cr.set_font_size(12.);
        let p = cr.text_extents(&talker.borrow().name());

        let x = self.area.b_x;
        let y = self.area.b_y;

        cr.move_to(x, y);
        cr.show_text(&talker.borrow().name());

        println!(
        "Talker {} :\n x_bearing {}, y_bearing {}, width {}, height {}, x_advance {}, y_advance {}", &talker.borrow().name(),
        p.x_bearing,
        p.y_bearing,
        p.width,
        p.height,
        p.x_advance,
        p.y_advance);
    }
    fn draw_connections(
        &self,
        drawing_area: &DrawingArea,
        cr: &Context,
        talker: &RTalker,
        talker_controls: &HashMap<Id, RTalkerControl>,
    ) { /*
               A.fold_left mGEars ~init:0
                 ~f:(fun index gEar ->
                     try match gEar.earType with
                       | GWord _ -> index + 1
                       | GTalk talk ->
                         let tkr = Ear.getTalkTalker talk in
                         let gTkr = L.assoc tkr#getId gpTalkers in

                         let port = Ear.getTalkPort talk in

                         if port < A.length gTkr#getGVoices then (

                           let voice = gTkr#getGVoices.(port)  in

                           let (x1, y1) = gTkr#getGroup#i2w ~x:gTkr#getWidth ~y:voice.voiceY in
                           let (x2, y2) = mGroup#i2w ~x:0. ~y:gEar.earY in

                           let tab = boxRadius +. marge in
                           let props = [`OUTLINE_COLOR_RGBA voice.voiceColor; `WIDTH_PIXELS 2] in

                           let bpath = GnomeCanvas.PathDef.new_path ~size:4 () in

                           GnomeCanvas.PathDef.moveto bpath x1 y1;
                           GnomeCanvas.PathDef.lineto bpath (x1 +. tab) y1;

                           if x2 >= x1 then (
                             let dx = (x2 -. x1) /. 2. in
                             GnomeCanvas.PathDef.curveto bpath
                               (x1 +. dx) y1 (x2 -. dx) y2 (x2 -. tab) y2;
                           )
                           else (
                             let dx = 10. *. tab in
                             let dy = (y2 -. y1) /. 2. in
                             GnomeCanvas.PathDef.curveto bpath
                               (x1 +. dx) (y1 +. dy) (x2 -. dx) (y2 -. dy) (x2 -. tab) y2;
                           );

                           GnomeCanvas.PathDef.lineto bpath x2 y2;

                           let line = GnoCanvas.bpath ~bpath ~props canvas#root in
                           line#lower_to_bottom();
                         );
                         index + 1
                       | GAdd -> index
                     with Not_found -> index + 1
                   ) |> ignore
         */
    }
}

pub trait TalkerControl {
    // fn to_ref(self) -> RefCell<dyn TalkerControl> {
    //     RefCell::new(self)
    // }

    fn base<'a>(&'a self) -> &'a TalkerControlBase;

    fn id(&self) -> Id {
        self.base().id()
    }

    fn row(&self) -> i32 {
        self.base().row
    }
    fn set_row(&mut self, row: i32) {
        self.base().row = row;
    }
    fn column(&self) -> i32 {
        self.base().column
    }
    fn set_column(&mut self, column: i32) {
        self.base().column = column;
    }
    fn dependent_row(&self) -> i32 {
        self.base().dependent_row
    }
    fn set_dependent_row(&mut self, row: i32) {
        self.base().dependent_row = row;
    }
    fn width(&self) -> f64 {
        self.base().width
    }
    fn height(&self) -> f64 {
        self.base().height
    }

    fn draw(
        &self,
        drawing_area: &DrawingArea,
        cr: &Context,
        talker: &RTalker,
        talker_controls: &HashMap<Id, RTalkerControl>,
    ) {
        let base = self.base();
        base.draw_connections(drawing_area, cr, talker, talker_controls);
        base.draw_header(drawing_area, cr, talker, 0., true, true, true);
        base.draw_ears_and_voices(drawing_area, cr, talker, 0.);
        base.draw_box(drawing_area, cr, talker, 0., 0.);
    }

    fn move_to(&mut self, _x: f64, _y: f64) {}

    fn on_button_release(&mut self, x: f64, y: f64, controler: &RSessionControler) -> bool {
        if self.base().area.is_under(x, y) {
            true
        } else {
            false
        }
    }

    fn select(&mut self) {
        if let Some(area) = &self.base().box_area {
            area.selected = true;
        }
    }

    fn unselect(&mut self) {
        if let Some(area) = &self.base().box_area {
            area.selected = false;
        }
    }

    fn select_ear(&mut self, index: Index) {
        self.base().ears[index].area.selected = true;
    }

    fn unselect_ear(&mut self, index: Index) {
        self.base().ears[index].area.selected = false;
    }

    fn select_voice(&mut self, index: Index) {
        self.base().voices[index].area.selected = true;
    }

    fn unselect_voice(&mut self, index: Index) {
        self.base().voices[index].area.selected = false;
    }
}

pub type RTalkerControl = Box<dyn TalkerControl>;

impl TalkerControl for TalkerControlBase {
    fn base<'a>(&'a self) -> &'a TalkerControlBase {
        &self
    }
}
