use cairo::Context;

use crate::ui;

pub const ADD_TAG: &str = "+";
pub const SUP_TAG: &str = "-";
pub const VAL_TAG: &str = "←"; // ⟵
pub const ADD_IN_TAG: &str = "⊕"; // ● ⟴ ⊕
pub const DESTROY_TAG: &str = "✖";
pub const MINIMIZE_TAG: &str = "▬";
pub const SOLO_TAG: &str = "SOLO";
pub const MUTE_TAG: &str = "MUTE";

pub const SPACE: f64 = 4.;

pub const H_PADDING: f64 = 3.;
pub const V_PADDING: f64 = 3.;

pub const SYM_W: f64 = ui::style::FONT_SIZE;
pub const SYM_H: f64 = ui::style::FONT_SIZE;

pub const CHIP_W: f64 = 6.;
pub const CHIP_H: f64 = 6.;

pub const LINE_H: f64 = V_PADDING + ui::style::FONT_SIZE + V_PADDING + 1.;

#[derive(PartialEq, Debug, Copy, Clone)]
pub struct Area {
pub    b_x: f64,
pub    e_x: f64,
pub    b_y: f64,
pub    e_y: f64,
pub    content_b_x: f64,
pub    content_e_y: f64,
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
            b_x,
            e_x: b_x + w,
            b_y: self.b_y,
            e_y: self.e_y,
            content_b_x: b_x + H_PADDING,
            content_e_y: self.content_e_y,
        }
    }

    pub fn farthest_right_adjustment(&mut self, other: &mut Self) {
        if self.e_x != other.e_x {
            let (model, copy) = if self.e_x > other.e_x { (self, other) } else { (other, self) };
            let half_gap = (model.e_x - copy.e_x) * 0.5;
            copy.content_b_x += half_gap;
            copy.e_x = model.e_x;
        }
    }

    pub fn is_under(&self, x: f64, y: f64) -> bool {
        x >= self.b_x && x < self.e_x && y >= self.b_y && y < self.e_y
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Dim {
    pub w: f64,
    pub h: f64,
}
impl Dim {
    pub fn new(w: f64, h: f64) -> Dim {
        Self { w, h }
    }

    pub fn of_symbol(_cc: &Context, _txt: &str) -> Result<Dim, failure::Error> {
        Ok(Dim::new(SYM_W, SYM_H))
    }

    pub fn of_text(cc: &Context, txt: &str) -> Result<Dim, failure::Error> {
        let te = cc.text_extents(txt).map_err(|e| {
            failure::err_msg(format!(
                "Dim::of_text text_extents {} -> {}",
                txt, e
            ))
        })?;
        Ok(Dim::new(te.x_advance(), te.height()))
    }
}

pub struct ControlSupply<'a> {
pub    cc: &'a Context,
pub    add_dim: Dim,
pub sup_dim: Dim,
pub val_dim: Dim,
pub add_in_dim: Dim,
pub    minimize_dim: Dim,
pub    destroy_dim: Dim,
pub    solo_dim: Dim,
pub    mute_dim: Dim,
}

pub fn dim_to_area(b_x: f64, b_y: f64, dim: &Dim) -> Area {
    Area::of_content(b_x, b_y, dim.w, dim.h)
}

impl<'a> ControlSupply<'a> {
    pub fn new(cc: &'a Context) -> Result<ControlSupply<'a>, failure::Error> {
        ui::style::add(cc);
        let add_dim = Dim::of_symbol(cc, ADD_TAG)?;
        ui::style::sup(cc);
        let sup_dim = Dim::of_symbol(cc, SUP_TAG)?;
        ui::style::value(cc);
        let val_dim = Dim::of_symbol(cc, VAL_TAG)?;
        ui::style::add(cc);
        let add_in_dim = Dim::of_symbol(cc, ADD_IN_TAG)?;
        ui::style::switch(cc);
        let minimize_dim = Dim::of_symbol(cc, MINIMIZE_TAG)?;
        let destroy_dim = Dim::of_symbol(cc, DESTROY_TAG)?;
        let solo_dim = Dim::of_text(cc, SOLO_TAG)?;
        let mute_dim = Dim::of_text(cc, MUTE_TAG)?;
        Ok(Self {
            cc,
            add_dim,
            sup_dim,
            val_dim,
            add_in_dim,
            minimize_dim,
            destroy_dim,
            solo_dim,
            mute_dim,
        })
    }
    pub fn area_of(&self, txt: &str, b_x: f64, b_y: f64) -> Result<Area, failure::Error> {
        let te = self.cc.text_extents(txt).map_err(|e| {
            failure::err_msg(format!(
                "ControlSupply::area_of text_extents {} -> {}",
                txt, e
            ))
        })?;
        Ok(Area::of_content(b_x, b_y, te.x_advance(), te.height()))
    }
}
