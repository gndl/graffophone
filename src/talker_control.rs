use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

//use gdk::EventMask;
//use gio::prelude::*;
//use gtk::gtk_sys::GtkScrolledWindow;
//use gtk::prelude::*;
use gtk::DrawingArea;

use cairo::Context;

use talker::identifier::Identifiable;
use talker::identifier::{Id, Index};
use talker::talker::{RTalker, Talker, TalkerBase};

use crate::session_presenter::RSessionPresenter;
use crate::style;
use crate::style::Color;

pub const INPUT_TAG: &str = " I ";
pub const OUTPUT_TAG: &str = " O ";
pub const ADD_TAG: &str = " + ";
pub const SUP_TAG: &str = " - ";
pub const VAL_TAG: &str = " # ";

const MARGE: f64 = 3.;
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
    color: Color,
}

fn format_label(s: &str, max_len: usize) -> String {
    if s.len() > max_len {
        s[0..max_len].to_string() + "..."
    } else {
        s.to_string()
    }
}

fn format_name(s: &str) -> String {
    format_label(s, 24)
}
fn format_data(s: &str) -> String {
    format_label(s, 6)
}
fn format_value(v: &f32) -> String {
    format_label(&f32::to_string(v), 6)
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

pub struct ControlSupply<'a> {
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

impl<'a> ControlSupply<'a> {
    pub fn new(cc: &'a Context) -> ControlSupply<'a> {
        style::ear(cc);
        let input_dim = Dim::of(cc, INPUT_TAG);
        style::voice(cc);
        let output_dim = Dim::of(cc, OUTPUT_TAG);
        style::add(cc);
        let add_dim = Dim::of(cc, ADD_TAG);
        style::sup(cc);
        let sup_dim = Dim::of(cc, SUP_TAG);
        style::value(cc);
        let val_dim = Dim::of(cc, VAL_TAG);
        Self {
            cc,
            input_dim,
            output_dim,
            add_dim,
            sup_dim,
            val_dim,
        }
    }
    fn dim_of(&self, txt: &str) -> Dim {
        Dim::of(self.cc, txt)
    }
}

pub struct TalkerControlBase {
    id: Id,
    talker: RTalker,
    area: Area,
    pub row: i32,
    pub column: i32,
    dependent_row: i32,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    model_area: Option<Area>,
    name_area: Option<Area>,
    data_area: Option<Area>,
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
    pub fn new(
        talker: &RTalker,
        control_supply: &ControlSupply,
        draw_model: bool,
        draw_name: bool,
        draw_data: bool,
    ) -> Result<TalkerControlBase, failure::Error> {
        let tkr = talker.borrow();

        let mut model_w = 0.;
        let mut model_h = 0.;

        let mut box_b_y = 0.;

        let mut header_e_y = SPACE;

        if draw_model {
            style::model(control_supply.cc);
            let model_dim = control_supply.dim_of(tkr.model());
            model_w = model_dim.w;
            model_h = model_dim.h;
            box_b_y = model_h + MARGE;
            header_e_y += box_b_y;
        }

        let mut name_w = 0.;
        let mut name_h = 0.;
        let name_b_y = header_e_y;

        if draw_name {
            style::name(control_supply.cc);
            let name_dim = control_supply.dim_of(&format_name(&tkr.name()));
            name_w = name_dim.w;
            name_h = name_dim.h;
            header_e_y += name_h + SPACE;
        }
        let name_e_y = name_b_y + name_h;

        let mut data_w = 0.;
        let mut data_h = 0.;
        let data_b_y = header_e_y;

        if draw_data {
            style::data(control_supply.cc);
            let data_dim = control_supply.dim_of(&format_data(&tkr.data_string()));
            data_w = data_dim.w;
            data_h = data_dim.h;
            header_e_y += data_h + SPACE;
        }
        let data_e_y = data_b_y + data_h;

        let mut ears_e_y = header_e_y;
        let mut ears_e_x = 0.;
        let mut ears = Vec::new();

        for ear in tkr.ears() {
            let mut talks = Vec::new();
            let ear_is_multi_talk = ear.is_multi_talk();

            let (b_x, e_y, ear_e_x) = ear.fold_talks(
                |talk, (b_x, b_y, ear_e_x)| {
                    style::ear(control_supply.cc);
                    let tag_dim = control_supply.dim_of(talk.tag());
                    let (input_type, value, value_dim) = match talk.value() {
                        Some(v) => {
                            let value = format_value(&v);
                            style::value(control_supply.cc);
                            let value_dim = control_supply.dim_of(&value);
                            (InputType::Value, Some(value), value_dim)
                        }
                        None => (InputType::Talk, None, control_supply.val_dim),
                    };
                    let mut e_x = b_x + tag_dim.w + MARGE + value_dim.w;
                    let e_y = b_y + tag_dim.h;
                    let tag_e_x = b_x + tag_dim.w;
                    let value_b_x = tag_e_x + MARGE;
                    let value_e_x = value_b_x + value_dim.w;

                    let sup_area = if ear_is_multi_talk {
                        e_x += control_supply.sup_dim.w;
                        Some(Area::new(
                            value_e_x,
                            value_e_x + control_supply.sup_dim.w,
                            b_y,
                            e_y,
                        ))
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
            let mut ear_e_y = e_y + SPACE;

            if ear_is_multi_talk {
                ear_e_y += control_supply.add_dim.h;

                let add_ctrl = TalkControl {
                    area: Area::new(b_x, ear_e_x, e_y, ear_e_y),
                    tag: ADD_TAG.to_string(),
                    tag_area: Area::new(b_x, ear_e_x, e_y, ear_e_y),
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
        let mut voices_e_x = f64::max(voices_b_x, MARGE + f64::max(name_w, data_w));
        let mut voices_e_y = header_e_y + SPACE;

        let tkr_id = tkr.id();

        style::voice(control_supply.cc);

        for (port, voice) in tkr.voices().iter().enumerate() {
            let tag = voice.borrow().tag().to_string();
            let tag_dim = control_supply.dim_of(&tag);
            let e_x = voices_b_x + tag_dim.w;
            let e_y = voices_e_y + tag_dim.h;

            let vc = VoiceControl {
                tag,
                area: Area::new(voices_b_x, e_x, voices_e_y, e_y),
                color: style::make_color(tkr_id as u64, port as u64),
            };
            tmp_voices.push(vc);
            voices_e_x = f64::max(voices_e_x, e_x);
            voices_e_y = e_y + SPACE + SPACE;
        }

        let mut voices = Vec::new();

        for voice in tmp_voices {
            let b_x = voices_b_x + voices_e_x - voice.area.e_x;
            let vc = VoiceControl {
                tag: voice.tag,
                area: Area::new(b_x, voices_e_x, voice.area.b_y, voice.area.e_y),
                color: voice.color,
            };
            voices.push(vc);
        }

        let width = voices_e_x + MARGE;
        let height = f64::max(ears_e_y, voices_e_y) + MARGE;

        let model_area = if draw_model {
            let model_b_x = (width - model_w) / 2.;
            Some(Area::new(model_b_x, model_b_x + model_w, 0., model_h))
        } else {
            None
        };

        let name_area = if draw_name {
            let name_b_x = (width - name_w) / 2.;
            Some(Area::new(name_b_x, name_b_x + name_w, name_b_y, name_e_y))
        } else {
            None
        };

        let data_area = if draw_data {
            let data_b_x = (width - data_w) / 2.;
            Some(Area::new(data_b_x, data_b_x + data_w, data_b_y, data_e_y))
        } else {
            None
        };

        let box_area = Area::new(0., width, box_b_y, height);

        Ok(Self {
            id: tkr.id(),
            talker: talker.clone(),
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
        })
    }
    pub fn new_ref(
        talker: &RTalker,
        control_supply: &ControlSupply,
        draw_model: bool,
        draw_name: bool,
        draw_data: bool,
    ) -> Result<RTalkerControlBase, failure::Error> {
        Ok(Rc::new(RefCell::new(TalkerControlBase::new(
            talker,
            control_supply,
            draw_model,
            draw_name,
            draw_data,
        )?)))
    }

    pub fn id(&self) -> Id {
        self.id
    }
    pub fn row(&self) -> i32 {
        self.row
    }
    pub fn set_row(&mut self, row: i32) {
        self.row = row;
    }
    pub fn column(&self) -> i32 {
        self.column
    }
    pub fn set_column(&mut self, column: i32) {
        self.column = column;
    }
    pub fn dependent_row(&self) -> i32 {
        self.dependent_row
    }
    pub fn set_dependent_row(&mut self, row: i32) {
        self.dependent_row = row;
    }
    pub fn width(&self) -> f64 {
        self.width
    }
    pub fn set_width(&mut self, width: f64) {
        self.box_area.e_x = width;
        self.width = width;
    }
    pub fn height(&self) -> f64 {
        self.height
    }
    pub fn set_height(&mut self, height: f64) {
        self.box_area.e_y = height;
        self.height = height;
    }

    pub fn move_to(&mut self, x: f64, y: f64) {
        self.x = x;
        self.y = y;
    }

    pub fn draw_box(&self, cc: &Context, talker: &RTalker, px: f64, py: f64) {
        let w = self.width; //self.box_area.e_x - self.box_area.b_x;
        let h = self.box_area.e_y - self.box_area.b_y;

        style::box_background(cc);
        cc.rectangle(self.x + self.box_area.b_x, self.y + self.box_area.b_y, w, h);
        cc.fill();

        style::box_border(cc);
        cc.rectangle(self.x + self.box_area.b_x, self.y + self.box_area.b_y, w, h);
        cc.stroke();
    }

    pub fn draw_header(&self, cc: &Context, talker: &RTalker, py: f64) {
        if let Some(model_area) = &self.model_area {
            style::model(cc);
            cc.move_to(self.x + model_area.b_x, self.y + model_area.e_y);
            cc.show_text(talker.borrow().model());
        }
        if let Some(name_area) = &self.name_area {
            style::name(cc);
            cc.move_to(self.x + name_area.b_x, self.y + name_area.e_y);
            cc.show_text(&format_name(&talker.borrow().name()));
        }
        if let Some(data_area) = &self.data_area {
            style::data(cc);
            cc.move_to(self.x + data_area.b_x, self.y + data_area.e_y);
            cc.show_text(&format_data(&talker.borrow().data_string()));
        }
    }

    pub fn draw_ears_and_voices(&self, cc: &Context, talker: &RTalker, py: f64) {
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

    pub fn draw_connections(
        &self,
        cc: &Context,
        talker: &RTalker,
        talker_controls: &HashMap<Id, RTalkerControl>,
    ) {
        for (ear_idx, ear) in self.talker.borrow().ears().iter().enumerate() {
            if let Some(ear_ctrl) = self.ears.get(ear_idx) {
                let _ = ear.fold_talks(
                    |talk, talk_idx| {
                        if let Some(talk_ctrl) = ear_ctrl.talks.get(talk_idx) {
                            if let None = talk.value() {
                                if let Some(voice_rtkrc) =
                                    &talker_controls.get(&talk.talker().borrow().id())
                                {
                                    let voice_tkrc = voice_rtkrc.borrow();
                                    let voice_tkrcb = voice_tkrc.base().borrow();

                                    if let Some(voice) = voice_tkrcb.voices.get(talk.port()) {
                                        style::connection(cc, voice.color);

                                        let x1 = voice_tkrcb.x + voice.area.e_x;
                                        let y1 =
                                            voice_tkrcb.y + (voice.area.b_y + voice.area.e_y) * 0.5;
                                        let x2 = self.x + talk_ctrl.area.b_x;
                                        let y2 = self.y
                                            + (talk_ctrl.area.b_y + talk_ctrl.area.e_y) * 0.5;
                                        let tab = MARGE + SPACE;

                                        cc.move_to(x1, y1);
                                        cc.line_to(x1 + tab, y1);

                                        if x2 >= x1 {
                                            let dx = (x2 - x1) * 0.5;
                                            cc.curve_to(x1 + dx, y1, x2 - dx, y2, x2 - tab, y2);
                                        } else {
                                            let dx = 10. * tab;
                                            let dy = (y2 - y1) * 0.5;
                                            cc.curve_to(
                                                x1 + dx,
                                                y1 + dy,
                                                x2 - dx,
                                                y2 - dy,
                                                x2 - tab,
                                                y2,
                                            );
                                        }

                                        cc.line_to(x2, y2);
                                        cc.stroke();
                                    }
                                }
                            }
                        }
                        Ok(talk_idx + 1)
                    },
                    0,
                );
            }
        }
    }
}

pub trait TalkerControl {
    fn base<'a>(&'a self) -> &'a RTalkerControlBase;

    fn id(&self) -> Id {
        self.base().borrow().id
    }
    fn row(&self) -> i32 {
        self.base().borrow().row()
    }
    fn set_row(&mut self, row: i32) {
        self.base().borrow_mut().set_row(row);
    }
    fn column(&self) -> i32 {
        self.base().borrow().column()
    }
    fn set_column(&mut self, column: i32) {
        self.base().borrow_mut().set_column(column);
    }
    fn dependent_row(&self) -> i32 {
        self.base().borrow().dependent_row()
    }
    fn set_dependent_row(&mut self, row: i32) {
        self.base().borrow_mut().set_dependent_row(row);
    }
    fn width(&self) -> f64 {
        self.base().borrow().width()
    }
    fn set_width(&mut self, width: f64) {
        self.base().borrow_mut().set_width(width);
    }
    fn height(&self) -> f64 {
        self.base().borrow().height()
    }
    fn set_height(&mut self, height: f64) {
        self.base().borrow_mut().set_height(height);
    }

    fn draw(&self, cc: &Context, talker: &RTalker, talker_controls: &HashMap<Id, RTalkerControl>) {
        let base = self.base().borrow();
        base.draw_connections(cc, talker, talker_controls);
        base.draw_box(cc, talker, 0., 0.);
        base.draw_header(cc, talker, 0.);
        base.draw_ears_and_voices(cc, talker, 0.);
    }

    fn move_to(&mut self, x: f64, y: f64) {
        self.base().borrow_mut().move_to(x, y);
    }

    fn on_button_release(&mut self, x: f64, y: f64, presenter: &RSessionPresenter) -> bool {
        let base = self.base().borrow();

        let rx = x - base.x;
        let ry = y - base.y;

        if base.area.is_under(rx, ry) {
            for (ear_idx, ear) in base.ears.iter().enumerate() {
                if ear.area.is_under(rx, ry) {
                    for (talk_idx, talk) in ear.talks.iter().enumerate() {
                        if talk.tag_area.is_under(rx, ry) {
                            //                            presenter.select_ear();
                            return true;
                        }
                    }
                }
            }
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
    pub fn new(
        talker: &RTalker,
        control_supply: &ControlSupply,
    ) -> Result<TalkerControlImpl, failure::Error> {
        Ok(Self {
            base: TalkerControlBase::new_ref(talker, control_supply, false, true, true)?,
        })
    }
}

impl TalkerControl for TalkerControlImpl {
    fn base<'a>(&'a self) -> &'a RTalkerControlBase {
        &self.base
    }
}

pub fn new_ref(
    talker: &RTalker,
    control_supply: &ControlSupply,
) -> Result<RTalkerControl, failure::Error> {
    Ok(Rc::new(RefCell::new(TalkerControlImpl::new(
        talker,
        control_supply,
    )?)))
}
