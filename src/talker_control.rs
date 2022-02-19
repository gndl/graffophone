use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use cairo::Context;

use talker::horn::PortType;
use talker::identifier::Id;
use talker::identifier::Identifiable;
use talker::talker::RTalker;

use crate::bounded_float_entry;
use crate::graph_presenter::{GraphPresenter, RGraphPresenter};
use crate::style;
use crate::style::Color;
use session::event_bus::Notification;

pub const ADD_TAG: &str = "+";
pub const SUP_TAG: &str = "-";
pub const VAL_TAG: &str = "⟵";
pub const ADD_IN_TAG: &str = "⟴";
pub const DESTROY_TAG: &str = "✖";
pub const MAXIMIZE_TAG: &str = "▮";
pub const MINIMIZE_TAG: &str = "▬";

const SPACE: f64 = 4.;

const H_PADDING: f64 = 3.;
const V_PADDING: f64 = 3.;

#[derive(PartialEq, Debug, Copy, Clone)]
struct Area {
    b_x: f64,
    e_x: f64,
    b_y: f64,
    e_y: f64,
    content_b_x: f64,
    content_e_y: f64,
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
        }
    }

    pub fn right_align(&mut self, e_x: f64) {
        let dx = e_x - self.e_x;

        self.b_x += dx;
        self.e_x = e_x;
        self.content_b_x += dx;
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
        }
    }

    pub fn is_under(&self, x: f64, y: f64) -> bool {
        x >= self.b_x && x < self.e_x && y >= self.b_y && y < self.e_y
    }
}

struct HumControl {
    area: Area,
    add_in_area: Area,
    tag: String,
    tag_area: Area,
    value: Option<String>,
    value_area: Area,
    port_type: PortType,
}

struct SetControl {
    sup_area: Option<Area>,
    hums: Vec<HumControl>,
}

struct EarControl {
    area: Area,
    tag_area: Option<(String, Area)>,
    sets: Vec<SetControl>,
    add_set_area: Option<Area>,
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
fn format_tag(s: &str) -> String {
    //    s[0..1].to_uppercase() + &s[1..s.len()]
    s.to_uppercase()
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
    add_in_dim: Dim,
    maximize_dim: Dim,
    minimize_dim: Dim,
    destroy_dim: Dim,
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
        style::add(cc);
        let add_in_dim = Dim::of(cc, ADD_IN_TAG);
        style::switch(cc);
        let maximize_dim = Dim::of(cc, MAXIMIZE_TAG);
        let minimize_dim = Dim::of(cc, MINIMIZE_TAG);
        let destroy_dim = Dim::of(cc, DESTROY_TAG);
        Self {
            cc,
            add_dim,
            sup_dim,
            val_dim,
            add_in_dim,
            maximize_dim,
            minimize_dim,
            destroy_dim,
        }
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
    imize_area: Area,
    minimized: bool,
    destroy_area: Area,
}
pub type RTalkerControlBase = Rc<RefCell<TalkerControlBase>>;

/*                MODEL
     _______________________________
    |              NAME             |
    |             [DATA]            |
 ---|⊞ EAR_TAG_1 ⟵       VOICE_TAG_1|
----|⊞ EAR_TAG_2 ⟵                  |
    |                    VOICE_TAG_2|
  --|⊞ EAR_TAG_3 ⟵ -                |
    |⊞ EAR_TAG_3 ⟵ -     VOICE_TAG_3|
    |+                              |
\   |EAR_TAG_4                      |
----|⊞ HUM_TAG_1 ⟵ -                |
----|⊞ HUM_TAG_2 ⟵                  |
 /  |⊞ HUM_TAG_3 ⟵                  |
    |⊞ HUM_TAG_1 ⟵ -                |
    |⊞ HUM_TAG_2 ⟵                  |
    |⊞ HUM_TAG_3 ⟵                  |
    |+                              |
    |_______________________________|
*/
impl TalkerControlBase {
    pub fn new(
        talker: &RTalker,
        control_supply: &ControlSupply,
        draw_model: bool,
        draw_name: bool,
        draw_data: bool,
        minimized: bool,
    ) -> Result<TalkerControlBase, failure::Error> {
        let tkr = talker;

        let mut box_e_x = 0.;
        let mut box_b_y = 0.;

        let mut header_e_y = SPACE;

        let model_area = if draw_model && !minimized {
            style::model(control_supply.cc);
            let m_a = control_supply.area_of(&tkr.model(), 0., 0.);
            box_b_y = m_a.e_y;
            header_e_y += box_b_y;
            Some(m_a)
        } else {
            None
        };

        let imize_dim = if minimized {
            &control_supply.maximize_dim
        } else {
            &control_supply.minimize_dim
        };
        let imize_area = dim_to_area(0., header_e_y, imize_dim);

        let name_area = if draw_name {
            style::name(control_supply.cc);
            let n_a = control_supply.area_of(&format_name(&tkr.name()), imize_area.e_x, header_e_y);
            box_e_x = n_a.e_x;
            header_e_y = n_a.e_y;
            Some(n_a)
        } else {
            None
        };

        let data_area = if draw_data && !minimized {
            style::data(control_supply.cc);
            let d_a = control_supply.area_of(&format_data(&tkr.data_string()), 0., header_e_y);
            box_e_x = f64::max(box_e_x, d_a.e_x);
            header_e_y = d_a.e_y;
            Some(d_a)
        } else {
            None
        };

        let mut destroy_area = dim_to_area(box_e_x, imize_area.b_y, &control_supply.destroy_dim);
        box_e_x = destroy_area.e_x;

        let mut ears_e_y = header_e_y;
        let mut ears = Vec::with_capacity(tkr.ears().len());
        let mut voices_e_y = header_e_y;
        let mut voices = Vec::with_capacity(tkr.voices().len());

        if !minimized {
            let b_x = 0.;
            let mut ears_e_x = 0.;

            for ear in tkr.ears() {
                let mut sets = Vec::with_capacity(ear.sets_len());
                let ear_is_multi_set = ear.is_multi_set();
                let sup_set = ear.sets().len() > 1;
                let ear_tag = format_tag(ear.tag());
                let mut ear_e_x = 0.;
                let mut b_y = ears_e_y;

                let (ear_tag_area, hum_tag) = if ear.is_multi_hum() {
                    style::name(control_supply.cc);
                    let tag_area = control_supply.area_of(&ear_tag, b_x, ears_e_y);
                    b_y = tag_area.e_y;
                    (Some((ear_tag, tag_area)), None)
                } else {
                    (None, Some(&ear_tag))
                };
                let mut set_b_y = b_y;

                for set in ear.sets() {
                    let mut hums = Vec::with_capacity(set.hums().len());
                    let mut hums_e_x = 0.;

                    for hum in set.hums() {
                        let add_in_area = dim_to_area(b_x, b_y, &control_supply.add_in_dim);

                        let tag = if let Some(h_tag) = hum_tag {
                            h_tag.to_string()
                        } else {
                            format_tag(hum.tag())
                        };

                        let tag_area = control_supply.area_of(&tag, add_in_area.e_x, b_y);

                        let (value, value_area) = if let Some(v) = hum.value() {
                            let value = format_value(&v);
                            style::value(control_supply.cc);
                            let value_area = control_supply.area_of(&value, tag_area.e_x, b_y);

                            (Some(value), value_area)
                        } else {
                            (
                                None,
                                dim_to_area(tag_area.e_x, b_y, &control_supply.val_dim),
                            )
                        };

                        let area = Area::new(b_x, value_area.e_x, b_y, tag_area.e_y);

                        hums_e_x = f64::max(hums_e_x, value_area.e_x);
                        b_y = tag_area.e_y;

                        let hum_ctrl = HumControl {
                            area,
                            add_in_area,
                            tag,
                            tag_area,
                            value,
                            value_area,
                            port_type: hum.port_type(),
                        };
                        hums.push(hum_ctrl);
                    }

                    let sup_area = if sup_set {
                        let sup_a = dim_to_area(hums_e_x, set_b_y, &control_supply.sup_dim);
                        ear_e_x = f64::max(ear_e_x, sup_a.e_x);
                        Some(sup_a)
                    } else {
                        ear_e_x = f64::max(ear_e_x, hums_e_x);
                        None
                    };

                    let set_ctrl = SetControl { sup_area, hums };
                    sets.push(set_ctrl);
                    set_b_y = b_y;
                }
                let mut ear_e_y = b_y;
                let add_set_area = if ear_is_multi_set {
                    let add_area = dim_to_area(b_x, b_y, &control_supply.add_dim);
                    ear_e_y = add_area.e_y;
                    Some(add_area)
                } else {
                    None
                };

                let ear_ctrl = EarControl {
                    area: Area::new(b_x, ear_e_x, ears_e_y, ear_e_y),
                    tag_area: ear_tag_area,
                    sets,
                    add_set_area,
                };
                ears.push(ear_ctrl);
                ears_e_y = ear_e_y;
                ears_e_x = f64::max(ears_e_x, ear_e_x);
            }
            let voices_b_x = ears_e_x;
            let mut voices_e_x = f64::max(voices_b_x, box_e_x);

            let tkr_id = tkr.id();

            style::io(control_supply.cc);

            for (port, voice) in tkr.voices().iter().enumerate() {
                let tag = format_tag(voice.tag());
                let area = control_supply.area_of(&tag, voices_b_x, voices_e_y);

                voices_e_x = f64::max(voices_e_x, area.e_x);
                voices_e_y = area.e_y;
                let vc = VoiceControl {
                    tag,
                    area,
                    port_type: voice.port_type(),
                    color: style::make_color(tkr_id as u64, port as u64),
                };
                voices.push(vc);
            }

            for voice in &mut voices {
                voice.area.right_align(voices_e_x);
            }
            destroy_area.right_align(voices_e_x);
        }
        let width = destroy_area.e_x;
        let height = f64::max(ears_e_y, voices_e_y) + SPACE;

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
            model_area: model_area.map(|a| a.centered(0., width)),
            name_area: name_area.map(|a| a.centered(0., width)),
            data_area: data_area.map(|a| a.centered(0., width)),
            box_area: Area::new(0., width, box_b_y, height),
            ears,
            voices,
            imize_area,
            minimized,
            destroy_area,
        })
    }
    pub fn new_ref(
        talker: &RTalker,
        control_supply: &ControlSupply,
        draw_model: bool,
        draw_name: bool,
        draw_data: bool,
        minimized: bool,
    ) -> Result<RTalkerControlBase, failure::Error> {
        Ok(Rc::new(RefCell::new(TalkerControlBase::new(
            talker,
            control_supply,
            draw_model,
            draw_name,
            draw_data,
            minimized,
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
    pub fn height(&self) -> f64 {
        self.height
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
        } else {
            style::box_border(cc);
        }
        cc.rectangle(self.x + self.box_area.b_x, self.y + self.box_area.b_y, w, h);
        cc.stroke();
    }

    pub fn draw_header(&self, cc: &Context, draw_switch: bool) {
        if draw_switch {
            style::switch(cc);
            cc.move_to(
                self.x + self.imize_area.content_b_x,
                self.y + self.imize_area.content_e_y,
            );
            if self.minimized {
                cc.show_text(MAXIMIZE_TAG);
            } else {
                cc.show_text(MINIMIZE_TAG);
            }

            cc.move_to(
                self.x + self.destroy_area.content_b_x,
                self.y + self.destroy_area.content_e_y,
            );
            cc.show_text(DESTROY_TAG);
        }
        if let Some(model_area) = &self.model_area {
            style::model(cc);
            cc.move_to(
                self.x + model_area.content_b_x,
                self.y + model_area.content_e_y,
            );
            cc.show_text(&self.talker.model());
        }
        if let Some(name_area) = &self.name_area {
            style::name(cc);
            cc.move_to(
                self.x + name_area.content_b_x,
                self.y + name_area.content_e_y,
            );
            cc.show_text(&format_name(&self.talker.name()));
        }
        if let Some(data_area) = &self.data_area {
            style::data(cc);
            cc.move_to(
                self.x + data_area.content_b_x,
                self.y + data_area.content_e_y,
            );
            cc.show_text(&format_data(&self.talker.data_string()));
        }
    }

    fn draw_add_in(&self, cc: &Context, area: &Area, selected: bool) {
        if selected {
            style::selected_io_background(cc);

            cc.rectangle(
                self.x + area.b_x,
                self.y + area.b_y,
                area.e_x - area.b_x,
                area.e_y - area.b_y,
            );
            cc.fill();
        }
        style::add(cc);
        cc.move_to(self.x + area.content_b_x, self.y + area.content_e_y);
        cc.show_text(ADD_IN_TAG);
    }

    fn draw_io(
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
        if !self.minimized {
            for (ear_idx, ear) in self.ears.iter().enumerate() {
                for (set_idx, set) in ear.sets.iter().enumerate() {
                    for (hum_idx, hum) in set.hums.iter().enumerate() {
                        self.draw_add_in(
                            cc,
                            &hum.add_in_area,
                            graph_presenter
                                .ear_hum_add_in_selected(self.id, ear_idx, set_idx, hum_idx),
                        );

                        self.draw_io(
                            cc,
                            &hum.tag_area,
                            &hum.tag,
                            hum.port_type,
                            graph_presenter.ear_hum_selected(self.id, ear_idx, set_idx, hum_idx),
                        );

                        style::value(cc);
                        cc.move_to(
                            self.x + hum.value_area.content_b_x,
                            self.y + hum.value_area.content_e_y,
                        );
                        if let Some(v) = &hum.value {
                            cc.show_text(&v);
                        } else {
                            cc.show_text(VAL_TAG);
                        }
                    }
                    if let Some(sa) = &set.sup_area {
                        style::sup(cc);
                        cc.move_to(self.x + sa.content_b_x, self.y + sa.content_e_y);
                        cc.show_text(SUP_TAG);
                    }
                }
                if let Some((tag, area)) = &ear.tag_area {
                    style::name(cc);
                    cc.move_to(self.x + area.content_b_x, self.y + area.content_e_y);
                    cc.show_text(&tag);
                }
                if let Some(add_area) = ear.add_set_area {
                    style::add(cc);
                    cc.move_to(self.x + add_area.content_b_x, self.y + add_area.content_e_y);
                    cc.show_text(ADD_TAG);
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
    }

    fn draw_connection(
        &self,
        cc: &Context,
        talk_area: &Area,
        voice_tkrcb: &TalkerControlBase,
        voice_area: &Area,
        voice_color: &Color,
    ) {
        style::connection(cc, voice_color);

        let x1 = voice_tkrcb.x + voice_area.e_x;
        let y1 = voice_tkrcb.y + (voice_area.b_y + voice_area.e_y) * 0.5;
        let x2 = self.x + talk_area.b_x;
        let y2 = self.y + (talk_area.b_y + talk_area.e_y) * 0.5;
        let tab = 2.;

        cc.move_to(x1, y1);
        cc.line_to(x1 + tab, y1);

        if x2 >= x1 {
            let dx = (x2 - x1) * 0.5;
            cc.curve_to(x1 + dx, y1, x2 - dx, y2, x2 - tab, y2);
        } else {
            let dx = 10. * tab;
            let dy = (y2 - y1) * 0.5;
            cc.curve_to(x1 + dx, y1 + dy, x2 - dx, y2 - dy, x2 - tab, y2);
        }

        cc.line_to(x2, y2);
        cc.stroke();
    }

    pub fn draw_connections(&self, cc: &Context, talker_controls: &HashMap<Id, RTalkerControl>) {
        for (ear_idx, ear) in self.talker.ears().iter().enumerate() {
            for (set_idx, set) in ear.sets().iter().enumerate() {
                for (hum_idx, hum) in set.hums().iter().enumerate() {
                    for talk in hum.talks() {
                        if let None = talk.value() {
                            let mut ohum_area: Option<&Area> = None;

                            if self.minimized {
                                ohum_area = Some(&self.imize_area);
                            } else {
                                if let Some(ear_ctrl) = self.ears.get(ear_idx) {
                                    if let Some(set_ctrl) = ear_ctrl.sets.get(set_idx) {
                                        if let Some(hum_ctrl) = set_ctrl.hums.get(hum_idx) {
                                            ohum_area = Some(&hum_ctrl.area);
                                        }
                                    }
                                }
                            }

                            if let Some(hum_area) = ohum_area {
                                if let Some(voice_rtkrc) = &talker_controls.get(&talk.talker().id())
                                {
                                    let voice_tkrc = voice_rtkrc.borrow();
                                    let voice_tkrcb = voice_tkrc.base().borrow();

                                    if voice_tkrcb.minimized {
                                        self.draw_connection(
                                            cc,
                                            hum_area,
                                            &voice_tkrcb,
                                            &voice_tkrcb.destroy_area,
                                            &style::WHITE_COLOR,
                                        );
                                    } else {
                                        if let Some(voice) = voice_tkrcb.voices.get(talk.port()) {
                                            self.draw_connection(
                                                cc,
                                                hum_area,
                                                &voice_tkrcb,
                                                &voice.area,
                                                &voice.color,
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn on_hum_clicked(
        &self,
        rx: f64,
        ry: f64,
        ear_idx: usize,
        set_idx: usize,
        hum_idx: usize,
        hum: &HumControl,
        graph_presenter: &RGraphPresenter,
    ) -> Result<Option<Vec<Notification>>, failure::Error> {
        if hum.add_in_area.is_under(rx, ry) {
            let notifications = graph_presenter
                .borrow_mut()
                .select_ear_hum_add_in(self.id, ear_idx, set_idx, hum_idx)?;
            return Ok(Some(notifications));
        }

        if hum.tag_area.is_under(rx, ry) {
            let notifications = graph_presenter
                .borrow_mut()
                .select_ear_hum(self.id, ear_idx, set_idx, hum_idx)?;
            return Ok(Some(notifications));
        }

        if hum.value_area.is_under(rx, ry) {
            let ear_setter = graph_presenter.clone();
            let notifier = graph_presenter.clone();
            let talker_id = self.id;
            let (min, max, def) = self.talker.ear(ear_idx).hum_range(hum_idx);
            let cur = self
                .talker
                .ear(ear_idx)
                .talk_value_or_default(set_idx, hum_idx);
            println!(
                "hum_range : min {}, max {}, def {}, cur {}",
                min, max, def, cur
            );

            let value = bounded_float_entry::run(
                min.into(),
                max.into(),
                def.into(),
                cur.into(),
                move |v, fly| {
                    let _ = ear_setter.borrow_mut().set_talker_ear_talk_value(
                        talker_id, ear_idx, set_idx, hum_idx, 0, v as f32, fly,
                    );
                },
            );
            println!(
                "talker_control::on_hum_clicked : set_talker_ear_talk_value {}",
                value
            );
            let notifications = notifier.borrow_mut().set_talker_ear_talk_value(
                talker_id,
                ear_idx,
                set_idx,
                hum_idx,
                0,
                value as f32,
                false,
            )?;
            /*
            bounded_float_entry::create(
                min.into(),
                max.into(),
                cur.into(),
                move |v, fly| {
                    let _ = ear_setter
                        .borrow_mut()
                        .set_talker_ear_talk_value(
                            talker_id, ear_idx, talk_idx, v as f32, fly,
                        );
                },
                move || {
                    notifier.borrow().notify_talker_changed();
                },
            );
             */
            return Ok(Some(notifications));
        }
        return Ok(None);
    }

    pub fn on_button_release(
        &self,
        x: f64,
        y: f64,
        graph_presenter: &RGraphPresenter,
    ) -> Result<Option<Vec<Notification>>, failure::Error> {
        let rx = x - self.x;
        let ry = y - self.y;

        if self.area.is_under(rx, ry) {
            for (ear_idx, ear) in self.ears.iter().enumerate() {
                if ear.area.is_under(rx, ry) {
                    for (set_idx, set) in ear.sets.iter().enumerate() {
                        for (hum_idx, hum) in set.hums.iter().enumerate() {
                            if hum.area.is_under(rx, ry) {
                                match self.on_hum_clicked(
                                    rx,
                                    ry,
                                    ear_idx,
                                    set_idx,
                                    hum_idx,
                                    hum,
                                    graph_presenter,
                                )? {
                                    Some(notifications) => return Ok(Some(notifications)),
                                    None => break,
                                }
                            }
                        }

                        if let Some(sup_set_area) = &set.sup_area {
                            if sup_set_area.is_under(rx, ry) {
                                let notifications = graph_presenter
                                    .borrow()
                                    .sup_ear_set(self.id, ear_idx, set_idx)?;
                                return Ok(Some(notifications));
                            }
                        }
                    }
                    if let Some(add_set_area) = &ear.add_set_area {
                        if add_set_area.is_under(rx, ry) {
                            let notifications =
                                graph_presenter.borrow_mut().add_ear_set(self.id, ear_idx)?;
                            return Ok(Some(notifications));
                        }
                    }
                }
            }
            for (port, voice) in self.voices.iter().enumerate() {
                if voice.area.is_under(rx, ry) {
                    let notifications = graph_presenter.borrow_mut().select_voice(self.id, port)?;
                    return Ok(Some(notifications));
                }
            }
            // TODO : edit talker name and data

            if self.imize_area.is_under(rx, ry) {
                let notifications = graph_presenter.borrow_mut().minimize_talker(self.id)?;
                return Ok(Some(notifications));
            }

            if self.destroy_area.is_under(rx, ry) {
                let notifications = graph_presenter.borrow_mut().sup_talker(self.id)?;
                return Ok(Some(notifications));
            }

            let notifications = graph_presenter.borrow_mut().select_talker(self.id)?;
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
    fn height(&self) -> f64 {
        self.base().borrow().height()
    }
    fn draw_connections(&self, cc: &Context, talker_controls: &HashMap<Id, RTalkerControl>) {
        self.base().borrow().draw_connections(cc, talker_controls);
    }

    fn draw(&self, cc: &Context, graph_presenter: &GraphPresenter) {
        let base = self.base().borrow();

        base.draw_box(cc, graph_presenter);
        base.draw_header(cc, true);
        base.draw_ears_and_voices(cc, graph_presenter);
    }

    fn move_to(&mut self, x: f64, y: f64) {
        self.base().borrow_mut().move_to(x, y);
    }

    fn on_button_release(
        &self,
        x: f64,
        y: f64,
        graph_presenter: &RGraphPresenter,
    ) -> Result<Option<Vec<Notification>>, failure::Error> {
        self.base()
            .borrow()
            .on_button_release(x, y, graph_presenter)
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
        minimized: bool,
    ) -> Result<TalkerControlImpl, failure::Error> {
        Ok(Self {
            base: TalkerControlBase::new_ref(talker, control_supply, false, true, true, minimized)?,
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
    minimized: bool,
) -> Result<RTalkerControl, failure::Error> {
    Ok(Rc::new(RefCell::new(TalkerControlImpl::new(
        talker,
        control_supply,
        minimized,
    )?)))
}
