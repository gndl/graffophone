//use std::boxed::Box;
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

use crate::session_presenter::RSessionPresenter;
use crate::style;

pub const INPUT_TAG: &str = " I ";
pub const OUTPUT_TAG: &str = " O ";
pub const ADD_TAG: &str = " + ";
pub const SUP_TAG: &str = " - ";
pub const VAL_TAG: &str = " # ";

const MARGE: f64 = 4.;
const SPACE: f64 = 4.;

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

pub enum InputType {
    Value,
    Talk,
    Add,
}

struct TalkControl {
    area: Area,
    tag: String,
    tag_area: Area,
    value: Option<String>,
    value_area: Area,
    sup_area: Option<Area>,
    input_type: InputType,
    //    root_index: i32,
}

struct EarControl {
    area: Area,
    talks: Vec<TalkControl>,
}

struct VoiceControl {
    tag: String,
    area: Area,
    //    color: i64,
}

fn format_label(s: &str, max_len: usize) -> String {
    if s.len() > max_len {
        s[0..max_len].to_string() + "..."
    } else {
        s.to_string()
    }
}

fn format_name(s: &str) -> String {
    format_label(s, 12)
}
fn format_data(s: &str) -> String {
    format_label(s, 6)
}
fn format_value(v: &f32) -> String {
    format_label(&f32::to_string(v), 6)
}

pub struct TalkerControlBase {
    id: Id,
    area: Area,
    pub row: i32,
    pub column: i32,
    dependent_row: i32,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    model_area: Area,
    name_area: Area,
    data_area: Area,
    box_area: Area,
    ears: Vec<EarControl>,
    voices: Vec<VoiceControl>,
}
pub type RTalkerControlBase = Rc<RefCell<TalkerControlBase>>;

/*                           MODEL
                _______________________________
               |              NAME             |
               |             [DATA]            |
               |TAG_INPUT_1 [1]  [TAG_OUTPUT_1]|
               |TAG_INPUT_2 [2]                |
               |TAG_INPUT_3 [3]  [TAG_OUTPUT_2]|
               |_______________________________|
*/
impl TalkerControlBase {
    pub fn new(talker_base: &TalkerBase) -> TalkerControlBase {
        Self {
            id: talker_base.id(),
            area: Area::new(0., 20., 0., 40.),
            row: -1,
            column: -1,
            dependent_row: -1,
            x: 0.,
            y: 0.,
            width: 0.,
            height: 0.,
            model_area: Area::new(0., 0., 0., 0.),
            name_area: Area::new(0., 0., 0., 0.),
            data_area: Area::new(0., 0., 0., 0.),
            box_area: Area::new(0., 0., 0., 0.),
            ears: Vec::new(),
            voices: Vec::new(),
        }
    }
    // pub fn new_base_ref(talker: &RTalker) -> RTalkerControlBase {
    //     Rc::new(RefCell::new(TalkerControlBase::new(talker.borrow().base())))
    // }
    pub fn new_ref(talker_base: &TalkerBase) -> RTalkerControlBase {
        Rc::new(RefCell::new(TalkerControlBase::new(talker_base)))
    }

    fn draw_box(
        &self,
        drawing_area: &DrawingArea,
        cc: &Context,
        talker: &RTalker,
        px: f64,
        py: f64,
    ) {
        let w = self.box_area.e_x - self.box_area.b_x;
        let h = self.box_area.e_y - self.box_area.b_y;
        style::box_background(cc);
        cc.rectangle(self.x + self.box_area.b_x, self.y + self.box_area.b_y, w, h);
        cc.fill();
        style::box_border(cc);
        cc.rectangle(self.x + self.box_area.b_x, self.y + self.box_area.b_y, w, h);
        cc.stroke();
    }

    fn draw_header(
        &self,
        drawing_area: &DrawingArea,
        cc: &Context,
        talker: &RTalker,
        py: f64,
        draw_model: bool,
        draw_name: bool,
        draw_main_data: bool,
    ) {
        style::model(cc);
        cc.move_to(self.x + self.model_area.b_x, self.y + self.model_area.e_y);
        cc.show_text(talker.borrow().model());

        style::name(cc);
        cc.move_to(self.x + self.name_area.b_x, self.y + self.name_area.e_y);
        cc.show_text(&talker.borrow().name());

        style::value(cc);
        cc.move_to(self.x + self.data_area.b_x, self.y + self.data_area.e_y);
        cc.show_text(&format_data(&talker.borrow().data_string()));
    }

    fn draw_ears_and_voices(
        &self,
        drawing_area: &DrawingArea,
        cc: &Context,
        talker: &RTalker,
        py: f64,
    ) {
        for ear in &self.ears {
            for talk in &ear.talks {
                match talk.input_type {
                    InputType::Add => {
                        style::add(cc);
                        cc.move_to(self.x + talk.area.b_x, self.y + talk.area.e_y);
                        cc.show_text(ADD_TAG);
                    }
                    _ => {
                        if talk.tag_area.selected {
                            style::selected_ear(cc);
                        } else {
                            style::ear(cc);
                        }
                        cc.move_to(self.x + talk.tag_area.b_x, self.y + talk.tag_area.e_y);
                        cc.show_text(&talk.tag);

                        style::value(cc);
                        cc.move_to(self.x + talk.value_area.b_x, self.y + talk.value_area.e_y);
                        if let Some(v) = &talk.value {
                            cc.show_text(&v);
                        } else {
                            cc.show_text(VAL_TAG);
                        }

                        if let Some(sa) = &talk.sup_area {
                            style::sup(cc);
                            cc.move_to(self.x + sa.b_x, self.y + sa.e_y);
                            cc.show_text(SUP_TAG);
                        }
                    }
                }
            }
        }

        for voice in &self.voices {
            if voice.area.selected {
                style::selected_voice(cc);
            } else {
                style::voice(cc);
            }
            cc.move_to(self.x + voice.area.b_x, self.y + voice.area.e_y);
            cc.show_text(&voice.tag);
        }
    }

    fn draw_connections(
        &self,
        drawing_area: &DrawingArea,
        cc: &Context,
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

    fn base<'a>(&'a self) -> &'a RTalkerControlBase;

    fn id(&self) -> Id {
        self.base().borrow().id
    }

    fn row(&self) -> i32 {
        self.base().borrow().row
    }
    fn set_row(&mut self, row: i32) {
        self.base().borrow_mut().row = row;
    }
    fn column(&self) -> i32 {
        self.base().borrow().column
    }
    fn set_column(&mut self, column: i32) {
        self.base().borrow_mut().column = column;
    }
    fn dependent_row(&self) -> i32 {
        self.base().borrow().dependent_row
    }
    fn set_dependent_row(&mut self, row: i32) {
        self.base().borrow_mut().dependent_row = row;
    }
    fn width(&self) -> f64 {
        self.base().borrow().width
    }
    fn height(&self) -> f64 {
        self.base().borrow().height
    }

    fn draw(
        &self,
        drawing_area: &DrawingArea,
        cc: &Context,
        talker: &RTalker,
        talker_controls: &HashMap<Id, RTalkerControl>,
    ) {
        let base = self.base().borrow_mut();
        base.draw_connections(drawing_area, cc, talker, talker_controls);
        base.draw_box(drawing_area, cc, talker, 0., 0.);
        base.draw_header(drawing_area, cc, talker, 0., true, true, true);
        base.draw_ears_and_voices(drawing_area, cc, talker, 0.);
    }

    fn move_to(&mut self, x: f64, y: f64) {
        self.base().borrow_mut().x = x;
        self.base().borrow_mut().y = y;
    }

    fn on_button_release(&mut self, x: f64, y: f64, presenter: &RSessionPresenter) -> bool {
        let rx = x - self.base().borrow().x;
        let ry = y - self.base().borrow().y;

        if self.base().borrow().area.is_under(rx, ry) {
            true
        } else {
            false
        }
    }

    fn select(&mut self) {
        self.base().borrow_mut().box_area.selected = true;
    }

    fn unselect(&mut self) {
        self.base().borrow_mut().box_area.selected = false;
    }

    fn select_ear(&mut self, index: Index) {
        self.base().borrow_mut().ears[index].area.selected = true;
    }

    fn unselect_ear(&mut self, index: Index) {
        self.base().borrow_mut().ears[index].area.selected = false;
    }

    fn select_voice(&mut self, index: Index) {
        self.base().borrow_mut().voices[index].area.selected = true;
    }

    fn unselect_voice(&mut self, index: Index) {
        self.base().borrow_mut().voices[index].area.selected = false;
    }
}

pub type RTalkerControl = Rc<RefCell<dyn TalkerControl>>;

pub struct TalkerControlImpl {
    base: RTalkerControlBase,
}

impl TalkerControlImpl {
    pub fn new(talker: &RTalker) -> TalkerControlImpl {
        Self {
            base: TalkerControlBase::new_ref(talker.borrow().base()),
        }
    }
}

impl TalkerControl for TalkerControlImpl {
    fn base<'a>(&'a self) -> &'a RTalkerControlBase {
        &self.base
    }
}

pub fn new_ref(talker: &RTalker) -> RTalkerControl {
    Rc::new(RefCell::new(TalkerControlImpl::new(talker)))
}

#[derive(Debug, Copy, Clone)]
struct Dim {
    w: f64,
    h: f64,
}
impl Dim {
    pub fn new(w: f64, h: f64) -> Dim {
        Self { w, h }
    }
    pub fn of(cc: &Context, txt: &str) -> Dim {
        let te = cc.text_extents(txt);
        Dim::new(te.x_advance, te.height)
    }
}

pub struct Builder<'a> {
    cc: &'a Context,
    input_dim: Dim,
    output_dim: Dim,
    add_dim: Dim,
    sup_dim: Dim,
    val_dim: Dim,
}

fn text_extents_to_dim(te: &cairo::TextExtents) -> Dim {
    Dim::new(te.x_advance, te.height)
}

fn dim_to_area(dim: &Dim, x: f64, y: f64) -> Area {
    Area::new(x, x + dim.w, y, y + dim.h)
}

impl<'a> Builder<'a> {
    pub fn new(cc: &'a Context) -> Builder<'a> {
        //    cc.select_font_face("Sans", FontSlant::Normal, FontWeight::Normal);
        cc.set_font_size(12.);
        Self {
            cc,
            input_dim: Dim::of(cc, INPUT_TAG),
            output_dim: Dim::of(cc, OUTPUT_TAG),
            add_dim: Dim::of(cc, ADD_TAG),
            sup_dim: Dim::of(cc, SUP_TAG),
            val_dim: Dim::of(cc, VAL_TAG),
        }
    }

    pub fn build(&self, rtalker: &RTalker) -> Result<RTalkerControl, failure::Error> {
        let talker = rtalker.borrow();
        let model_dim = Dim::of(self.cc, talker.model());
        let name = talker.name();
        let name_dim = Dim::of(self.cc, &name);
        let data = format_data(&talker.data_string());
        let data_dim = Dim::of(self.cc, &data);

        let header_e_y = model_dim.h + MARGE + MARGE + name_dim.h + SPACE + data_dim.h + SPACE;
        let mut ears_e_y = header_e_y;

        let mut ears_e_x = 0.;
        let mut ears = Vec::new();

        for ear in talker.ears() {
            let mut talks = Vec::new();
            let ear_is_multi_talk = ear.is_multi_talk();

            let (b_x, e_y, ear_e_x) = ear.fold_talks(
                |talk, (b_x, b_y, ear_e_x)| {
                    let tag_dim = Dim::of(self.cc, talk.tag());
                    let (input_type, value, value_dim) = match talk.value() {
                        Some(v) => {
                            let value = format_value(&v);
                            let value_dim = Dim::of(self.cc, &value);
                            (InputType::Value, Some(value), value_dim)
                        }
                        None => (InputType::Talk, None, self.val_dim),
                    };
                    let mut e_x = b_x + tag_dim.w + MARGE + value_dim.w;
                    let e_y = b_y + tag_dim.h;
                    let tag_e_x = b_x + tag_dim.w;
                    let value_b_x = tag_e_x + MARGE;
                    let value_e_x = value_b_x + value_dim.w;

                    let sup_area = if ear_is_multi_talk {
                        e_x += self.sup_dim.w;
                        Some(Area::new(value_e_x, value_e_x + self.sup_dim.w, b_y, e_y))
                    } else {
                        None
                    };

                    let talk_ctrl = TalkControl {
                        area: Area::new(b_x, e_x, b_y, e_y),
                        tag: talk.tag().to_string(),
                        tag_area: Area::new(b_x, tag_e_x, b_y, e_y),
                        value,
                        value_area: Area::new(value_b_x, value_e_x, b_y, e_y),
                        sup_area,
                        //    b_y,
                        input_type,
                        //    root_index: i32,
                    };
                    talks.push(talk_ctrl);
                    Ok((b_x, e_y + SPACE, f64::max(ear_e_x, e_x)))
                },
                (MARGE, ears_e_y, 0.),
            )?;
            let mut ear_e_y = e_y;

            if ear_is_multi_talk {
                ear_e_y += self.add_dim.h;

                let add_ctrl = TalkControl {
                    area: Area::new(b_x, ear_e_x, e_y, ear_e_y),
                    tag: ADD_TAG.to_string(),
                    tag_area: Area::new(0., -1., 0., -1.),
                    value: None,
                    value_area: Area::new(0., -1., 0., -1.),
                    sup_area: None,
                    //  b_y: e_y,
                    input_type: InputType::Add,
                    //    root_index: i32,
                };
                talks.push(add_ctrl);
            }
            let ear_ctrl = EarControl {
                area: Area::new(b_x, ear_e_x, ears_e_y, ear_e_y),
                talks,
            };
            ears.push(ear_ctrl);
            ears_e_y = ear_e_y;
            ears_e_x = f64::max(ears_e_x, ear_e_x);
        }

        let mut tmp_voices = Vec::new();
        let voices_b_x = ears_e_x + MARGE;
        let mut voices_e_x = MARGE + f64::max(name_dim.w, data_dim.w);
        let mut voices_e_y = header_e_y;

        for voice in talker.voices() {
            let tag = voice.borrow().tag().to_string();
            let tag_dim = Dim::of(self.cc, &tag);
            let e_x = voices_b_x + tag_dim.w;
            let e_y = voices_e_y + tag_dim.h;

            let vc = VoiceControl {
                tag,
                area: Area::new(voices_b_x, e_x, voices_e_y, e_y),
            };
            tmp_voices.push(vc);
            voices_e_x = f64::max(voices_e_x, e_x);
            voices_e_y = e_y + SPACE;
        }

        let mut voices = Vec::new();

        for voice in tmp_voices {
            let b_x = voices_b_x + voices_e_x - voice.area.e_x;
            let vc = VoiceControl {
                tag: voice.tag,
                area: Area::new(b_x, voices_e_x, voice.area.b_y, voice.area.e_y),
            };
            voices.push(vc);
        }

        let width = voices_e_x + MARGE;
        let height = f64::max(ears_e_y, voices_e_y) + MARGE;

        let model_b_x = (width - model_dim.w) / 2.;
        let model_area = Area::new(model_b_x, model_b_x + model_dim.w, 0., model_dim.h);

        let name_b_x = (width - name_dim.w) / 2.;
        let name_b_y = model_dim.h + MARGE;
        let name_e_y = name_b_y + name_dim.h;
        let name_area = Area::new(name_b_x, name_b_x + name_dim.w, name_b_y, name_e_y);

        let data_b_x = (width - data_dim.w) / 2.;
        let data_e_y = name_e_y + SPACE;
        let data_area = Area::new(
            data_b_x,
            data_b_x + data_dim.w,
            data_e_y,
            data_e_y + data_dim.h,
        );

        let box_area = Area::new(0., width, model_dim.h + MARGE, height);

        let base = TalkerControlBase {
            id: talker.id(),
            area: Area::new(0., width, 0., height),
            row: -1,
            column: -1,
            dependent_row: -1,
            x: 0.,
            y: 0.,
            width,
            height,
            model_area,
            name_area,
            data_area,
            box_area,
            ears,
            voices,
        };
        Ok(Rc::new(RefCell::new(TalkerControlImpl {
            base: Rc::new(RefCell::new(base)),
        })))
    }
}
