use std::cell::RefCell;
use std::collections::HashMap;
use std::f64::consts::PI;
use std::rc::Rc;

use cairo::Context;

use talker::data::RData;
use talker::horn::PortType;
use talker::identifier::{Id, Identifiable};
use talker::talker::RTalker;

use crate::graph_presenter::{GraphPresenter, RGraphPresenter};
use crate::util;
use session::event_bus::Notification;
use crate::ui::{self, control::{Area, ControlSupply}, style::Color};

struct HumControl {
    area: Area,
    add_in_area: Area,
    tag: String,
    tag_area: Area,
    value: Option<String>,
    value_area: Option<Area>,
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
    let mut label = s.trim_start();

    if let Some(eol_pos) = label.find("\n") {
        label = &label[0..eol_pos];
    }

    if label.is_empty() {
        "...".to_string()
    } else if label.len() > max_len {
        label[0..max_len].to_string() + "..."
    } else {
        label.to_string()
    }
}

fn format_name(s: &str) -> String {
    format_label(s, 24)
}
fn format_data(data: &RData) -> String {
    let data = data.borrow();

    if let Some(s) = data.to_string() {
        format_label(&s, 15)
    }
    else if data.is_ui() {
        "[-O-]".to_string()
    }
    else {
        String::default()
    }
}
fn format_tag(s: &str) -> String {
    //    s[0..1].to_uppercase() + &s[1..s.len()]
    s.to_uppercase()
}
fn format_value(v: &f32) -> String {
    format_label(&f32::to_string(v), 6)
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
    pub fn new<F>(
        talker: &RTalker,
        control_supply: &ControlSupply,
        draw_model: bool,
        draw_name: bool,
        draw_data: bool,
        minimized: bool,
        mut customize_ear_set: F,
    ) -> Result<TalkerControlBase, failure::Error>
    where
        F: FnMut(usize, usize, f64, f64) -> (f64, f64),
    {
        let tkr = talker;

        let mut box_e_x = 0.;
        let mut box_b_y = 0.;

        let mut header_e_y = ui::control::SPACE;

        let model_area = if draw_model && !minimized {
            ui::style::model(control_supply.cc);
            let m_a = control_supply.area_of(&tkr.model(), 0., 0.)?;
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
        let imize_area = ui::control::dim_to_area(0., header_e_y, imize_dim);

        let name_area = if draw_name {
            ui::style::name(control_supply.cc);
            let n_a =
                control_supply.area_of(&format_name(&tkr.name()), imize_area.e_x, header_e_y)?;
            box_e_x = n_a.e_x;
            header_e_y = n_a.e_y;
            Some(n_a)
        } else {
            None
        };

        let data_area = if draw_data && !minimized {
            ui::style::data(control_supply.cc);
            let d_a = control_supply.area_of(&format_data(tkr.data()), 0., header_e_y)?;
            box_e_x = box_e_x.max(d_a.e_x);
            header_e_y = d_a.e_y;
            Some(d_a)
        } else {
            None
        };

        let mut destroy_area = ui::control::dim_to_area(box_e_x, imize_area.b_y, &control_supply.destroy_dim);
        box_e_x = destroy_area.e_x;

        let mut ears_e_y = header_e_y;
        let mut ears = Vec::with_capacity(tkr.ears().len());
        let mut voices_e_y = header_e_y;
        let mut voices = Vec::with_capacity(tkr.voices().len());

        if !minimized {
            let b_x = 0.;
            let mut ears_e_x: f64 = 0.;

            for ear in tkr.ears() {
                let mut sets = Vec::with_capacity(ear.sets_len());
                let ear_is_multi_set = ear.is_multi_set();
                let set_padding = if ear_is_multi_set { ui::control::V_PADDING } else { 0. };
                let sup_set = ear.sets().len() > 1;
                let ear_tag = format_tag(ear.tag());
                let mut ear_e_x: f64 = 0.;
                let mut b_y = ears_e_y;

                let (ear_tag_area, hum_tag) = if ear.is_multi_hum() {
                    ui::style::name(control_supply.cc);
                    let tag_area = control_supply.area_of(&ear_tag, b_x, ears_e_y)?;
                    b_y = tag_area.e_y;
                    (Some((ear_tag, tag_area)), None)
                } else {
                    (None, Some(&ear_tag))
                };
                let mut set_b_y = b_y;

                for set in ear.sets() {
                    let mut hums = Vec::with_capacity(set.hums().len());
                    let mut hums_e_x: f64 = 0.;

                    for hum in set.hums() {
                        let add_in_area = ui::control::dim_to_area(b_x, b_y, &control_supply.add_in_dim);

                        let tag = if let Some(h_tag) = hum_tag {
                            h_tag.to_string()
                        } else {
                            format_tag(hum.tag())
                        };

                        let tag_area = control_supply.area_of(&tag, add_in_area.e_x, b_y)?;

                        let (value, value_area, hum_area) = if hum.can_have_a_value() {
                            let (value, value_area) = if let Some(v) = hum.value() {
                                let value = format_value(&v);
                                ui::style::value(control_supply.cc);
                                let value_area =
                                    control_supply.area_of(&value, tag_area.e_x, b_y)?;

                                (Some(value), value_area)
                            } else {
                                (
                                    None,
                                    ui::control::dim_to_area(tag_area.e_x, b_y, &control_supply.val_dim),
                                )
                            };
                            let hum_area = Area::new(b_x, value_area.e_x, b_y, tag_area.e_y);
                            (value, Some(value_area), hum_area)
                        } else {
                            (None, None, Area::new(b_x, tag_area.e_x, b_y, tag_area.e_y))
                        };

                        hums_e_x = hums_e_x.max(hum_area.e_x);
                        b_y = tag_area.e_y;

                        let hum_ctrl = HumControl {
                            area: hum_area,
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
                        let sup_a = ui::control::dim_to_area(hums_e_x, set_b_y, &control_supply.sup_dim);
                        ear_e_x = ear_e_x.max(sup_a.e_x);
                        Some(sup_a)
                    } else {
                        ear_e_x = ear_e_x.max(hums_e_x);
                        None
                    };

                    let set_ctrl = SetControl { sup_area, hums };
                    sets.push(set_ctrl);
                    b_y += set_padding;
                    set_b_y = b_y;
                }
                let mut ear_e_y = b_y;
                let add_set_area = if ear_is_multi_set {
                    let add_area = ui::control::dim_to_area(b_x, b_y, &control_supply.add_dim);
                    ear_e_x = ear_e_x.max(add_area.e_x);
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
                ears_e_x = ears_e_x.max(ear_e_x);
            }

            // Custome control
            let mut custs_e_x = ears_e_x;

            for (ear_idx, ear) in ears.iter().enumerate() {
                for (set_idx, set) in ear.sets.iter().enumerate() {
                    let cust_b_y = match set.sup_area {
                        Some(sa) => sa.e_y,
                        None => set.hums[0].area.b_y,
                    };
                    let (cust_e_x, _) = customize_ear_set(ear_idx, set_idx, ears_e_x, cust_b_y);

                    custs_e_x = custs_e_x.max(cust_e_x);
                }
            }

            for ear in ears.iter_mut() {
                for set in ear.sets.iter_mut() {
                    if let Some(sa) = set.sup_area.as_mut() {
                        sa.right_align(custs_e_x);
                    }
                }
            }

            // Voices
            let voices_b_x = custs_e_x;
            let mut voices_e_x = voices_b_x.max(box_e_x);

            let tkr_id = tkr.id();

            ui::style::io(control_supply.cc);

            for (port, voice) in tkr.voices().iter().enumerate() {
                let tag = format_tag(voice.tag());
                let (associated_ear, associated_set) = voice.get_associated_ear_set();

                let b_y = if ears.len() > associated_ear && ears[associated_ear].sets.len() > associated_set {
                    voices_e_y.max(ears[associated_ear].sets[associated_set].hums[0].area.b_y)
                }
                else {
                    voices_e_y
                };

                let area = control_supply.area_of(&tag, voices_b_x, b_y)?;

                voices_e_x = voices_e_x.max(area.e_x);
                voices_e_y = area.e_y;
                
                let vc = VoiceControl {
                    tag,
                    area,
                    port_type: voice.port_type(),
                    color: ui::style::make_color(tkr_id as u64, port as u64),
                };
                voices.push(vc);
            }

            for voice in &mut voices {
                voice.area.right_align(voices_e_x);
            }
            destroy_area.right_align(voices_e_x + ui::control::CHIP_W);
        }
        let width = destroy_area.e_x;
        let height = ears_e_y.max(voices_e_y) + ui::control::SPACE;

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
    pub fn new_ref<F>(
        talker: &RTalker,
        control_supply: &ControlSupply,
        draw_model: bool,
        draw_name: bool,
        draw_data: bool,
        minimized: bool,
        customize_ear_set: F,
    ) -> Result<RTalkerControlBase, failure::Error>
    where
        F: FnMut(usize, usize, f64, f64) -> (f64, f64),
    {
        Ok(Rc::new(RefCell::new(TalkerControlBase::new(
            talker,
            control_supply,
            draw_model,
            draw_name,
            draw_data,
            minimized,
            customize_ear_set,
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
    pub fn height(&self) -> f64 {
        self.height
    }
    pub fn move_to(&mut self, x: f64, y: f64) {
        self.x = x;
        self.y = y;
    }

    pub fn draw_control<SS,SSS>(
        &self,
        cc: &Context,
        area: &Area,
        txt: &str,
        style: SS,
        style_selected: SSS,
        selected: bool,
    ) -> Result<(), cairo::Error>
    where SS: Fn(&Context),  SSS: Fn(&Context)
    {
        if selected {
            ui::style::selected(cc);

            cc.rectangle(
                self.x + area.b_x,
                self.y + area.b_y,
                area.e_x - area.b_x,
                area.e_y - area.b_y,
            );
            cc.fill()?;

            style_selected(cc);
        } else {
            style(cc);
        }
        cc.move_to(self.x + area.content_b_x, self.y + area.content_e_y);
        cc.show_text(txt)?;
        Ok(())
    }

    pub fn draw_box(
        &self,
        cc: &Context,
        graph_presenter: &GraphPresenter,
    ) -> Result<(), cairo::Error> {
        let w = self.box_area.e_x - self.box_area.b_x; //self.width;
        let h = self.box_area.e_y - self.box_area.b_y;

        ui::style::box_background(cc);
        cc.rectangle(self.x + self.box_area.b_x, self.y + self.box_area.b_y, w, h);
        cc.fill()?;

        if graph_presenter.talker_selected(self.id) {
            ui::style::selected(cc);
        } else {
            ui::style::box_border(cc);
        }
        cc.rectangle(self.x + self.box_area.b_x, self.y + self.box_area.b_y, w, h);
        cc.stroke()
    }

    fn draw_imize(&self, cc: &Context, area: &Area, minimized: bool) -> Result<(), cairo::Error> {
        if minimized {
            cc.rectangle(
                self.x + area.content_b_x + 3.,
                self.y + area.content_e_y - ui::control::SYM_H,
                3.,
                ui::control::SYM_H,
            );
        } else {
            cc.rectangle(
                self.x + area.content_b_x,
                self.y + area.content_e_y - ui::control::SYM_H + 3.,
                ui::control::SYM_W,
                3.,
            );
        }
        cc.stroke()
    }

    fn draw_cross(&self, cc: &Context, area: &Area) -> Result<(), cairo::Error> {
        let x1 = self.x + area.content_b_x;
        let y1 = self.y + area.content_e_y;
        let x2 = x1 + ui::control::SYM_W;
        let y2 = y1 - ui::control::SYM_H;
        cc.move_to(x1, y1);
        cc.line_to(x2, y2);
        cc.move_to(x1, y2);
        cc.line_to(x2, y1);
        cc.stroke()
    }

    pub fn draw_header(&self, cc: &Context, draw_switch: bool) -> Result<(), cairo::Error> {
        if draw_switch {
            ui::style::switch(cc);
            self.draw_imize(cc, &self.imize_area, self.minimized)?;
            self.draw_cross(cc, &self.destroy_area)?;
        }
        if let Some(model_area) = &self.model_area {
            ui::style::model(cc);
            cc.move_to(
                self.x + model_area.content_b_x,
                self.y + model_area.content_e_y,
            );
            cc.show_text(&self.talker.model())?;
        }
        if let Some(name_area) = &self.name_area {
            ui::style::name(cc);
            cc.move_to(
                self.x + name_area.content_b_x,
                self.y + name_area.content_e_y,
            );
            cc.show_text(&format_name(&self.talker.name()))?;
        }
        if let Some(data_area) = &self.data_area {
            ui::style::data(cc);
            cc.move_to(
                self.x + data_area.content_b_x,
                self.y + data_area.content_e_y,
            );
            cc.show_text(&format_data(self.talker.data()))?;
        }
        Ok(())
    }

    fn draw_add(&self, cc: &Context, area: &Area) -> Result<(), cairo::Error> {
        ui::style::add(cc);
        let x1 = self.x + area.content_b_x;
        let y1 = self.y + area.content_e_y - (ui::control::SYM_H * 0.5);
        let x2 = x1 + ui::control::SYM_W;
        cc.move_to(x1, y1);
        cc.line_to(x2, y1);

        let x3 = x1 + (ui::control::SYM_W * 0.5);
        let y2 = self.y + area.content_e_y;
        let y3 = y2 - ui::control::SYM_H;
        cc.move_to(x3, y2);
        cc.line_to(x3, y3);
        cc.stroke()
    }
    fn draw_add_in(&self, cc: &Context, area: &Area, selected: bool) -> Result<(), cairo::Error> {
        if selected {
            ui::style::selected_io_background(cc);

            cc.rectangle(
                self.x + area.b_x,
                self.y + area.b_y,
                area.e_x - area.b_x,
                area.e_y - area.b_y,
            );
            cc.fill()?;
        }
        self.draw_add(cc, area)?;
        let r = ui::control::SYM_W * 0.5;
        let a = PI * 0.5;
        cc.arc(
            self.x + area.content_b_x + r,
            self.y + area.content_e_y - r,
            r,
            -a,
            a,
        );
        cc.stroke()
    }

    fn draw_value(
        &self,
        cc: &Context,
        area: &Area,
        value: &Option<String>,
    ) -> Result<(), cairo::Error> {
        ui::style::value(cc);
        if let Some(v) = value {
            cc.move_to(self.x + area.content_b_x, self.y + area.content_e_y);
            cc.show_text(&v)
        } else {
            let x1 = self.x + area.content_b_x;
            let y1 = self.y + area.content_e_y - (ui::control::SYM_H * 0.5);
            let x2 = x1 + ui::control::SYM_W;
            cc.move_to(x1, y1);
            cc.line_to(x2, y1);

            let x3 = x1 + (ui::control::SYM_W * 0.5);
            let y2 = self.y + area.content_e_y;
            let y3 = y2 - ui::control::SYM_H;
            cc.move_to(x3, y2);
            cc.line_to(x1, y1);
            cc.line_to(x3, y3);
            cc.stroke()
        }
    }
    fn draw_io(
        &self,
        cc: &Context,
        area: &Area,
        txt: &String,
        port_type: PortType,
        selected: bool,
    ) -> Result<(), cairo::Error> {
        if selected {
            ui::style::selected_io_background(cc);

            cc.rectangle(
                self.x + area.b_x,
                self.y + area.b_y,
                area.e_x - area.b_x,
                area.e_y - area.b_y,
            );
            cc.fill()?;

            match port_type {
                PortType::Audio => ui::style::selected_audio(cc),
                PortType::Control => ui::style::selected_control(cc),
                PortType::Cv => ui::style::selected_cv(cc),
                PortType::Atom => ui::style::selected_atom(cc),
            }
        } else {
            match port_type {
                PortType::Audio => ui::style::audio(cc),
                PortType::Control => ui::style::control(cc),
                PortType::Cv => ui::style::cv(cc),
                PortType::Atom => ui::style::atom(cc),
            }
        }
        cc.move_to(self.x + area.content_b_x, self.y + area.content_e_y);
        cc.show_text(txt)?;
        Ok(())
    }

    pub fn draw_ears_and_voices(
        &self,
        cc: &Context,
        graph_presenter: &GraphPresenter,
    ) -> Result<(), cairo::Error> {
        if !self.minimized {
            for (ear_idx, ear) in self.ears.iter().enumerate() {
                for (set_idx, set) in ear.sets.iter().enumerate() {
                    for (hum_idx, hum) in set.hums.iter().enumerate() {
                        self.draw_add_in(
                            cc,
                            &hum.add_in_area,
                            graph_presenter
                                .ear_hum_add_in_selected(self.id, ear_idx, set_idx, hum_idx),
                        )?;

                        self.draw_io(
                            cc,
                            &hum.tag_area,
                            &hum.tag,
                            hum.port_type,
                            graph_presenter.ear_hum_selected(self.id, ear_idx, set_idx, hum_idx),
                        )?;

                        if let Some(value_area) = &hum.value_area {
                            self.draw_value(cc, value_area, &hum.value)?;
                        }
                    }
                    if let Some(sa) = &set.sup_area {
                        ui::style::sup(cc);
                        self.draw_cross(cc, sa)?;
                    }
                }
                if let Some((tag, area)) = &ear.tag_area {
                    ui::style::name(cc);
                    cc.move_to(self.x + area.content_b_x, self.y + area.content_e_y);
                    cc.show_text(&tag)?;
                }
                if let Some(add_area) = ear.add_set_area {
                    self.draw_add(cc, &add_area)?;
                }
            }

            for (voice_idx, voice) in self.voices.iter().enumerate() {
                // Draw voice tag
                self.draw_io(
                    cc,
                    &voice.area,
                    &voice.tag,
                    voice.port_type,
                    graph_presenter.voice_selected(self.id, voice_idx),
                )?;

                // Draw connection chip
                cc.rectangle(
                    self.x + voice.area.e_x,
                    self.y + (voice.area.b_y + voice.area.e_y - ui::control::CHIP_H) / 2.,
                    ui::control::CHIP_W,
                    ui::control::CHIP_H,
                );
                ui::style::set_color(cc, voice.color);
                cc.fill()?;
            }
        }
        Ok(())
    }

    fn draw_connection(
        &self,
        cc: &Context,
        talk_area: &Area,
        voice_tkrcb: &TalkerControlBase,
        voice_area: &Area,
        voice_color: &Color,
    ) -> Result<(), cairo::Error> {
        ui::style::connection(cc, voice_color);

        let x1 = voice_tkrcb.x + voice_area.e_x;
        let y1 = voice_tkrcb.y + (voice_area.b_y + voice_area.e_y) * 0.5;
        let x2 = self.x + talk_area.b_x;
        let y2 = self.y + (talk_area.b_y + talk_area.e_y) * 0.5;
        let tab = 4.;
        let mdx = (x2 - x1) * 0.5;

        cc.move_to(x1, y1);
        
        if x2 >= x1 {
            cc.line_to(x1 + tab, y1);
            cc.curve_to(x1 + mdx, y1, x2 - mdx, y2, x2 - tab, y2);
            cc.line_to(x2, y2);
        } else {
            let xray = 150.;
            let mdy = (y2 - y1) * 0.5;
            let qdy = mdy * 0.5;
            cc.curve_to(x1 + tab, y1, x1 + xray, y1 + qdy, x1 + mdx, y1 + mdy);
            cc.curve_to(x2 - xray, y2 - qdy, x2 - tab, y2, x2, y2);
        }

        cc.stroke()?;
        Ok(())
    }

    pub fn draw_connections(
        &self,
        cc: &Context,
        talker_controls: &HashMap<Id, RTalkerControl>,
    ) -> Result<(), cairo::Error> {
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
                                            &ui::style::WHITE_COLOR,
                                        )?;
                                    } else {
                                        if let Some(voice) = voice_tkrcb.voices.get(talk.port()) {
                                            self.draw_connection(
                                                cc,
                                                hum_area,
                                                &voice_tkrcb,
                                                &voice.area,
                                                &voice.color,
                                            )?;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(())
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

        if let Some(value_area) = &hum.value_area {
            if value_area.is_under(rx, ry) {
                return Ok(Some(vec![Notification::EarValueSelected(
                    self.id, ear_idx, set_idx, hum_idx,
                )]));
            }
        }
        return Ok(None);
    }

    pub fn relative_coordinates(
        &self,
        x: f64,
        y: f64,
    ) -> (f64, f64) {
        (x - self.x, y - self.y)
    }

    pub fn is_under(
        &self,
        x: f64,
        y: f64,
    ) -> bool {
        let (rx, ry) = self.relative_coordinates(x, y);
        self.area.is_under(rx, ry)
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
            // TODO : edit talker name

            if let Some(data_area) = &self.data_area {
                if data_area.is_under(rx, ry) {
                    let notifications = graph_presenter.borrow_mut().select_data_talker(self.id)?;
                    return Ok(Some(notifications));
                }
            }

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

    fn is_positioned(&self) -> bool {
        let base = self.base().borrow();
        base.row() > -1 && base.column() > -1
    }


    fn draw_connections(&self, cc: &Context, talker_controls: &HashMap<Id, RTalkerControl>) {
        util::print_cairo_result(self.base().borrow().draw_connections(cc, talker_controls));
    }

    fn draw(&self, cc: &Context, graph_presenter: &GraphPresenter) {
        let base = self.base().borrow();

        util::print_cairo_result(base.draw_box(cc, graph_presenter));
        util::print_cairo_result(base.draw_header(cc, true));
        util::print_cairo_result(base.draw_ears_and_voices(cc, graph_presenter));
    }

    fn position(&self) -> (f64, f64) {
        let base = self.base().borrow();
        (base.x, base.y)
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
            base: TalkerControlBase::new_ref(
                talker,
                control_supply,
                false,
                true,
                true,
                minimized,
            |_, _, b_x, b_y| (b_x, b_y))?,
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
