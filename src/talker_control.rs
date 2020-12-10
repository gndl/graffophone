use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use gtk::DrawingArea;

use cairo::Context;

use talker::identifier::Identifiable;
use talker::identifier::{Id, Index};
use talker::talker::{RTalker, Talker, TalkerBase};
use talker::voice::PortType;

use crate::graph_presenter::{GraphPresenter, RGraphPresenter};
use crate::session_presenter::RSessionPresenter;
use crate::style;
use crate::style::Color;
use session::event_bus::{Notification, REventBus};

pub const ADD_TAG: &str = " + ";
pub const SUP_TAG: &str = " - ";
pub const VAL_TAG: &str = " # ";

const SPACE: f64 = 4.;

const H_PADDING: f64 = 6.;
const V_PADDING: f64 = 3.;

#[derive(PartialEq, Debug, Copy, Clone)]
struct Area {
    b_x: f64,
    e_x: f64,
    b_y: f64,
    e_y: f64,
    content_b_x: f64,
    content_e_y: f64,
    selected: bool,
}
impl Area {
    pub fn new(b_x: f64, e_x: f64, b_y: f64, e_y: f64) -> Area {
        Self {
            b_x,
            e_x,
            b_y,
            e_y,
            content_b_x: b_x,
            content_e_y: e_y,
            selected: false,
        }
    }
    pub fn of_content(b_x: f64, b_y: f64, w: f64, h: f64) -> Area {
        Self {
            b_x,
            e_x: b_x + H_PADDING + w + H_PADDING + 1.,
            b_y,
            e_y: b_y + V_PADDING + h + V_PADDING + 1.,
            content_b_x: b_x + H_PADDING,
            content_e_y: b_y + V_PADDING + h,
            selected: false,
        }
    }
    pub fn copy(&self) -> Area {
        Self {
            b_x: self.b_x,
            e_x: self.e_x,
            b_y: self.b_y,
            e_y: self.e_y,
            content_b_x: self.content_b_x,
            content_e_y: self.content_e_y,
            selected: self.selected,
        }
    }

    pub fn right_align(&mut self, e_x: f64) {
        let dx = e_x - self.e_x;

        self.b_x += dx;
        self.e_x = e_x;
        self.content_b_x += dx;
    }
    pub fn center(&mut self, l: f64, r: f64) {
        let w = self.e_x - self.b_x;
        let b_x = (l + r - w) * 0.5;

        self.b_x = b_x;
        self.e_x = b_x + w;
        self.content_b_x = b_x + H_PADDING;
    }

    pub fn centered(&self, l: f64, r: f64) -> Area {
        let w = self.e_x - self.b_x;
        let b_x = (l + r - w) * 0.5;
        Self {
            b_x: b_x,
            e_x: b_x + w,
            b_y: self.b_y,
            e_y: self.e_y,
            content_b_x: b_x + H_PADDING,
            content_e_y: self.content_e_y,
            selected: self.selected,
        }
    }

    pub fn is_under(&self, x: f64, y: f64) -> bool {
        x >= self.b_x && x < self.e_x && y >= self.b_y && y < self.e_y
    }
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
    port_type: PortType,
}

struct EarControl {
    area: Area,
    talks: Vec<TalkControl>,
}

struct VoiceControl {
    tag: String,
    area: Area,
    port_type: PortType,
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
    add_dim: Dim,
    sup_dim: Dim,
    val_dim: Dim,
}

fn dim_to_area(b_x: f64, b_y: f64, dim: &Dim) -> Area {
    Area::of_content(b_x, b_y, dim.w, dim.h)
}

impl<'a> ControlSupply<'a> {
    pub fn new(cc: &'a Context) -> ControlSupply<'a> {
        style::add(cc);
        let add_dim = Dim::of(cc, ADD_TAG);
        style::sup(cc);
        let sup_dim = Dim::of(cc, SUP_TAG);
        style::value(cc);
        let val_dim = Dim::of(cc, VAL_TAG);
        Self {
            cc,
            add_dim,
            sup_dim,
            val_dim,
        }
    }
    fn dim_of(&self, txt: &str) -> Dim {
        Dim::of(self.cc, txt)
    }
    fn area_of(&self, txt: &str, b_x: f64, b_y: f64) -> Area {
        let te = self.cc.text_extents(txt);
        Area::of_content(b_x, b_y, te.x_advance, te.height)
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

        let mut box_e_x = 0.;
        let mut box_b_y = 0.;

        let mut header_e_y = SPACE;

        let model_area = if draw_model {
            style::model(control_supply.cc);
            let m_a = control_supply.area_of(tkr.model(), 0., 0.);
            box_b_y = m_a.e_y;
            header_e_y += box_b_y;
            Some(m_a)
        } else {
            None
        };

        let mut name_area = if draw_name {
            style::name(control_supply.cc);
            let n_a = control_supply.area_of(&format_name(&tkr.name()), 0., header_e_y);
            box_e_x = n_a.e_x;
            header_e_y = n_a.e_y;
            Some(n_a)
        } else {
            None
        };

        let data_area = if draw_data {
            style::data(control_supply.cc);
            let d_a = control_supply.area_of(&format_data(&tkr.data_string()), 0., header_e_y);
            box_e_x = f64::max(box_e_x, d_a.e_x);
            header_e_y = d_a.e_y;
            Some(d_a)
        } else {
            None
        };

        let mut ears_e_y = header_e_y;
        let mut ears_e_x = 0.;
        let mut ears = Vec::new();

        for ear in tkr.ears() {
            let mut talks = Vec::new();
            let ear_is_multi_talk = ear.is_multi_talk();

            let (b_x, e_y, ear_e_x) = ear.fold_talks(
                |talk, (b_x, b_y, ear_e_x)| {
                    style::io(control_supply.cc);
                    let tag_area = control_supply.area_of(talk.tag(), b_x, b_y);

                    let (input_type, value, value_area) = match talk.value() {
                        Some(v) => {
                            let value = format_value(&v);
                            style::value(control_supply.cc);
                            let value_area = control_supply.area_of(&value, tag_area.e_x, b_y);

                            (InputType::Value, Some(value), value_area)
                        }
                        None => (
                            InputType::Talk,
                            None,
                            dim_to_area(tag_area.e_x, b_y, &control_supply.val_dim),
                        ),
                    };

                    let mut e_x = value_area.e_x;
                    let e_y = tag_area.e_y;

                    let sup_area = if ear_is_multi_talk {
                        let sup_a = dim_to_area(value_area.e_x, b_y, &control_supply.sup_dim);
                        e_x = sup_a.e_x;
                        Some(sup_a)
                    } else {
                        None
                    };

                    let talk_ctrl = TalkControl {
                        area: Area::new(b_x, e_x, b_y, e_y),
                        tag: talk.tag().to_string(),
                        tag_area,
                        value,
                        value_area,
                        sup_area,
                        input_type,
                        port_type: talk.port_type(),
                    };
                    talks.push(talk_ctrl);
                    Ok((b_x, e_y, f64::max(ear_e_x, e_x)))
                },
                (0., ears_e_y, 0.),
            )?;
            let mut ear_e_y = e_y;

            if ear_is_multi_talk {
                let add_area = dim_to_area(b_x, e_y, &control_supply.add_dim);
                ear_e_y = add_area.e_y;

                let add_ctrl = TalkControl {
                    area: add_area,
                    tag: ADD_TAG.to_string(),
                    tag_area: add_area.copy(),
                    value: None,
                    value_area: Area::new(0., -1., 0., -1.),
                    sup_area: None,
                    input_type: InputType::Add,
                    port_type: PortType::Control,
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

        let mut voices = Vec::new();
        let voices_b_x = ears_e_x;
        let mut voices_e_x = f64::max(voices_b_x, box_e_x);
        let mut voices_e_y = header_e_y;

        let tkr_id = tkr.id();

        style::io(control_supply.cc);

        for (port, voice) in tkr.voices().iter().enumerate() {
            let tag = voice.borrow().tag().to_string();
            let area = control_supply.area_of(&tag, voices_b_x, voices_e_y);

            voices_e_x = f64::max(voices_e_x, area.e_x);
            voices_e_y = area.e_y;
            let vc = VoiceControl {
                tag,
                area,
                port_type: voice.borrow().port_type(),
                color: style::make_color(tkr_id as u64, port as u64),
            };
            voices.push(vc);
        }

        for voice in &mut voices {
            voice.area.right_align(voices_e_x);
        }

        let width = voices_e_x;
        let height = f64::max(ears_e_y, voices_e_y) + SPACE;

        Ok(Self {
            id: tkr_id,
            talker: talker.clone(),
            area: Area::new(0., width, 0., height),
            row: -1,
            column: -1,
            dependent_row: -1,
            x: 0.,
            y: 0.,
            width,
            height,
            model_area: model_area.map(|a| a.centered(0., width)),
            name_area: name_area.map(|a| a.centered(0., width)),
            data_area: data_area.map(|a| a.centered(0., width)),
            box_area: Area::new(0., width, box_b_y, height),
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

    pub fn draw_box(&self, cc: &Context, graph_presenter: &GraphPresenter) {
        let w = self.box_area.e_x - self.box_area.b_x; //self.width;
        let h = self.box_area.e_y - self.box_area.b_y;

        style::box_background(cc);
        cc.rectangle(self.x + self.box_area.b_x, self.y + self.box_area.b_y, w, h);
        cc.fill();

        if graph_presenter.talker_selected(self.id) {
            style::selected(cc);
            cc.rectangle(
                self.x + self.box_area.b_x - 2.,
                self.y + self.box_area.b_y - 2.,
                w + 4.,
                h + 4.,
            );
        } else {
            style::box_border(cc);
            cc.rectangle(self.x + self.box_area.b_x, self.y + self.box_area.b_y, w, h);
        }
        cc.stroke();
    }

    pub fn draw_header(&self, cc: &Context) {
        if let Some(model_area) = &self.model_area {
            style::model(cc);
            cc.move_to(
                self.x + model_area.content_b_x,
                self.y + model_area.content_e_y,
            );
            cc.show_text(self.talker.borrow().model());
        }
        if let Some(name_area) = &self.name_area {
            style::name(cc);
            cc.move_to(
                self.x + name_area.content_b_x,
                self.y + name_area.content_e_y,
            );
            cc.show_text(&format_name(&self.talker.borrow().name()));
        }
        if let Some(data_area) = &self.data_area {
            style::data(cc);
            cc.move_to(
                self.x + data_area.content_b_x,
                self.y + data_area.content_e_y,
            );
            cc.show_text(&format_data(&self.talker.borrow().data_string()));
        }
    }

    pub fn draw_io(
        &self,
        cc: &Context,
        area: &Area,
        txt: &String,
        port_type: PortType,
        selected: bool,
    ) {
        if selected {
            style::selected_io_background(cc);

            cc.rectangle(
                self.x + area.b_x,
                self.y + area.b_y,
                area.e_x - area.b_x,
                area.e_y - area.b_y,
            );
            cc.fill();

            match port_type {
                PortType::Audio => style::selected_audio(cc),
                PortType::Control => style::selected_control(cc),
                PortType::Cv => style::selected_cv(cc),
            }
        } else {
            match port_type {
                PortType::Audio => style::audio(cc),
                PortType::Control => style::control(cc),
                PortType::Cv => style::cv(cc),
            }
        }
        cc.move_to(self.x + area.content_b_x, self.y + area.content_e_y);
        cc.show_text(txt);
    }

    pub fn draw_ears_and_voices(&self, cc: &Context, graph_presenter: &GraphPresenter) {
        for (ear_idx, ear) in self.ears.iter().enumerate() {
            for (talk_idx, talk) in ear.talks.iter().enumerate() {
                match talk.input_type {
                    InputType::Add => {
                        style::add(cc);
                        cc.move_to(
                            self.x + talk.tag_area.content_b_x,
                            self.y + talk.tag_area.content_e_y,
                        );
                        cc.show_text(ADD_TAG);
                    }
                    _ => {
                        self.draw_io(
                            cc,
                            &talk.tag_area,
                            &talk.tag,
                            talk.port_type,
                            graph_presenter.ear_talk_selected(self.id, ear_idx, talk_idx),
                        );

                        style::value(cc);
                        cc.move_to(
                            self.x + talk.value_area.content_b_x,
                            self.y + talk.value_area.content_e_y,
                        );
                        if let Some(v) = &talk.value {
                            cc.show_text(&v);
                        } else {
                            cc.show_text(VAL_TAG);
                        }

                        if let Some(sa) = &talk.sup_area {
                            style::sup(cc);
                            cc.move_to(self.x + sa.content_b_x, self.y + sa.content_e_y);
                            cc.show_text(SUP_TAG);
                        }
                    }
                }
            }
        }

        for (voice_idx, voice) in self.voices.iter().enumerate() {
            self.draw_io(
                cc,
                &voice.area,
                &voice.tag,
                voice.port_type,
                graph_presenter.voice_selected(self.id, voice_idx),
            );
        }
    }

    pub fn draw_connections(&self, cc: &Context, talker_controls: &HashMap<Id, RTalkerControl>) {
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
                                        let tab = SPACE;

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

    pub fn on_button_release(
        &self,
        x: f64,
        y: f64,
        graph_presenter: &mut GraphPresenter,
    ) -> Result<Option<Vec<Notification>>, failure::Error> {
        let rx = x - self.x;
        let ry = y - self.y;

        if self.area.is_under(rx, ry) {
            for (ear_idx, ear) in self.ears.iter().enumerate() {
                if ear.area.is_under(rx, ry) {
                    for (talk_idx, talk) in ear.talks.iter().enumerate() {
                        if talk.area.is_under(rx, ry) {
                            if talk.tag_area.is_under(rx, ry) {
                                let notifications = match &talk.input_type {
                                    InputType::Add => {
                                        graph_presenter.add_ear_talk(&self.talker, ear_idx)?
                                    }
                                    _ => graph_presenter.select_ear_talk(
                                        &self.talker,
                                        ear_idx,
                                        talk_idx,
                                    )?,
                                };
                                return Ok(Some(notifications));
                            }

                            if talk.value_area.is_under(rx, ry) {
                                // TODO : display float delector
                                let notifications = graph_presenter
                                    .set_talker_ear_talk_value_by_index(
                                        &self.talker,
                                        ear_idx,
                                        talk_idx,
                                        100.,
                                        false,
                                    )?;
                                return Ok(Some(notifications));
                            }

                            if let Some(sup_area) = &talk.sup_area {
                                if sup_area.is_under(rx, ry) {
                                    let notifications = graph_presenter.sup_ear_talk(
                                        &self.talker,
                                        ear_idx,
                                        talk_idx,
                                    )?;
                                    return Ok(Some(notifications));
                                }
                            }
                        }
                    }
                }
            }
            for (port, voice) in self.voices.iter().enumerate() {
                if voice.area.is_under(rx, ry) {
                    let notifications = graph_presenter.select_voice(&self.talker, port)?;
                    return Ok(Some(notifications));
                }
            }
            // TODO : edit talker name and data
            let notifications = graph_presenter.select_talker(&self.talker)?;
            Ok(Some(notifications))
        } else {
            Ok(None)
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

    fn draw(
        &self,
        cc: &Context,
        graph_presenter: &GraphPresenter,
        talker_controls: &HashMap<Id, RTalkerControl>,
    ) {
        let base = self.base().borrow();
        base.draw_connections(cc, talker_controls);
        base.draw_box(cc, graph_presenter);
        base.draw_header(cc);
        base.draw_ears_and_voices(cc, graph_presenter);
    }

    fn move_to(&mut self, x: f64, y: f64) {
        self.base().borrow_mut().move_to(x, y);
    }

    fn on_button_release(
        &self,
        x: f64,
        y: f64,
        graph_presenter: &mut GraphPresenter,
    ) -> Result<Option<Vec<Notification>>, failure::Error> {
        self.base()
            .borrow()
            .on_button_release(x, y, graph_presenter)
    }

    fn select(&self) {
        self.base().borrow_mut().box_area.selected = true;
    }
    fn unselect(&self) {
        self.base().borrow_mut().box_area.selected = false;
    }
    fn select_ear(&self, ear_idx: Index, talk_idx: Index) {
        self.base().borrow_mut().ears[ear_idx].talks[talk_idx]
            .tag_area
            .selected = true;
        println!(
            "Talker {} ear {} talk {} selected",
            self.base().borrow().talker.borrow().id(),
            ear_idx,
            talk_idx
        );
    }
    fn unselect_ear(&self, ear_idx: Index, talk_idx: Index) {
        self.base().borrow_mut().ears[ear_idx].talks[talk_idx]
            .tag_area
            .selected = false;
        println!(
            "Talker {} ear {} talk {} unselected",
            self.base().borrow().talker.borrow().id(),
            ear_idx,
            talk_idx
        );
    }
    fn select_voice(&self, index: Index) {
        self.base().borrow_mut().voices[index].area.selected = true;
    }
    fn unselect_voice(&self, index: Index) {
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
