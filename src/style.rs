/*
 * Copyright (C) 2015 Ga�tan Dubreil
 *
 *  All rights reserved.This file is distributed under the terms of the
 *  GNU General Public License version 3.0.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Lesser General Public License for more details.
 *
 * You should have received a copy of the GNU Lesser General Public License
 * along with this program; if not, write to the Free Software
 * Foundation, Inc., 59 Temple Place - Suite 330, Boston, MA 02111-1307, USA.
 */
//use cairo::enums::{FontSlant, FontWeight};
use cairo::Context;

pub type Color = (f64, f64, f64);

pub fn set_color(cc: &Context, (r, g, b): Color) {
    cc.set_source_rgb(r, g, b);
}

const FONT_SIZE: f64 = 12.;

const H00: f64 = 0.;
const HFF: f64 = 1.;
const H20: f64 = 32. / 255.;
const H50: f64 = 80. / 255.;
const H80: f64 = 128. / 255.;
const H90: f64 = 144. / 255.;
const HD3: f64 = 211. / 255.;
const HE0: f64 = 224. / 255.;
const HEE: f64 = 238. / 255.;

const BACKGROUND_COLOR: Color = (H00, H00, H00);
const DELIMITATION_COLOR: Color = (H50, H50, H50);
const SELECTION_COLOR: Color = (HFF, H80, H00);
const REVERSE_SELECTION_COLOR: Color = (HE0, HFF, HE0);
const BOX_BACKGROUND_COLOR: Color = (H20, H20, H20);
const BOX_BORDER_COLOR: Color = (H50, H50, H50);
const MODEL_COLOR: Color = (HD3, HD3, HD3); // lightgray
const NAME_COLOR: Color = (HFF, HFF, HFF); // white
const DATA_COLOR: Color = (H00, HFF, HFF); // cyan
const EAR_COLOR: Color = (HFF, HFF, H00); // yellow
const SELECTED_EAR_COLOR: Color = (HFF, H00, HFF); // magenta
const VALUE_COLOR: Color = (H00, HFF, HFF); // cyan
const SUP_COLOR: Color = (HFF, H00, H00); // lightgreen
const ADD_COLOR: Color = (H90, HEE, H90); // lightgreen
const VOICE_COLOR: Color = (H90, HEE, H90); // lightgreen
const SELECTED_VOICE_COLOR: Color = (HFF, H00, HFF); // magenta

pub fn background(cc: &Context) {
    set_color(cc, BACKGROUND_COLOR);
}
pub fn delimitation(cc: &Context) {
    set_color(cc, DELIMITATION_COLOR);
}
pub fn selection(cc: &Context) {
    set_color(cc, SELECTION_COLOR);
    cc.set_font_size(FONT_SIZE);
}
pub fn reverse_selection(cc: &Context) {
    set_color(cc, REVERSE_SELECTION_COLOR);
    cc.set_font_size(FONT_SIZE);
}
pub fn box_background(cc: &Context) {
    set_color(cc, BOX_BACKGROUND_COLOR);
}
pub fn box_border(cc: &Context) {
    set_color(cc, BOX_BORDER_COLOR);
    cc.set_line_width(0.5);
}
pub fn selected_box_background(cc: &Context) {
    set_color(cc, SELECTION_COLOR);
}

pub fn model(cc: &Context) {
    set_color(cc, MODEL_COLOR);
    // cc.select_font_face("Sans", FontSlant::Normal, FontWeight::Normal);
    cc.set_font_size(FONT_SIZE);
}
pub fn name(cc: &Context) {
    set_color(cc, NAME_COLOR);
    cc.set_font_size(FONT_SIZE);
}
pub fn data(cc: &Context) {
    set_color(cc, DATA_COLOR);
    cc.set_font_size(FONT_SIZE);
}
pub fn ear(cc: &Context) {
    set_color(cc, EAR_COLOR);
    cc.set_font_size(FONT_SIZE);
}
pub fn selected_ear(cc: &Context) {
    set_color(cc, SELECTED_EAR_COLOR);
    cc.set_font_size(FONT_SIZE);
}
pub fn value(cc: &Context) {
    set_color(cc, VALUE_COLOR);
    cc.set_font_size(FONT_SIZE);
}
pub fn sup(cc: &Context) {
    set_color(cc, SUP_COLOR);
    cc.set_font_size(FONT_SIZE);
}
pub fn add(cc: &Context) {
    set_color(cc, ADD_COLOR);
    cc.set_font_size(FONT_SIZE);
}
pub fn voice(cc: &Context) {
    set_color(cc, VOICE_COLOR);
    cc.set_font_size(FONT_SIZE);
}
pub fn selected_voice(cc: &Context) {
    set_color(cc, SELECTED_VOICE_COLOR);
    cc.set_font_size(FONT_SIZE);
}

pub fn connection(cc: &Context, color: Color) {
    set_color(cc, color);
    cc.set_line_width(2.);
}

//pub fn flow            (cc: &Context,) {set_color(cc, _COLOR);} //Color.ofString "0x00FFFFFF" /* "cyan" */
//pub fn marker          (cc: &Context,) {set_color(cc, _COLOR);} //Color.ofString "0xFF8000FF" /* "orange" */
pub fn make_color(d1: u64, d2: u64) -> Color {
    let v = d1 + (d2 << 14);
    let r = (95 + (v % 3) * 80) as f64 / 255.;
    let g = (95 + (v % 5) * 40) as f64 / 255.;
    let b = (75 + (v % 7) * 30) as f64 / 255.;
    (r, g, b)
}
