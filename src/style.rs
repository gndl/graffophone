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
const SELECTED_IO_BACKGROUND_COLOR: Color = (1., 0.8, 0.4);
const SELECTED_BOX_BACKGROUND_COLOR: Color = (0.6, 0.3, 0.);
const REVERSE_SELECTION_COLOR: Color = (HE0, HFF, HE0);
const BOX_BACKGROUND_COLOR: Color = (H20, H20, H20);
const BOX_BORDER_COLOR: Color = (H50, H50, H50);
const MODEL_COLOR: Color = (HD3, HD3, HD3); // lightgray
const NAME_COLOR: Color = (HFF, HFF, HFF); // white
const DATA_COLOR: Color = (H90, HEE, H90); // lightgreen
const AUDIO_COLOR: Color = (0., 1., 1.); // cyan
const CONTROL_COLOR: Color = (1., 1., 0.); // yellow
const CV_COLOR: Color = (1., 0., 1.); // magenta
const IO_COLOR: Color = (0.7, 0.7, 0.7);
const SELECTED_AUDIO_COLOR: Color = (0., 0.5, 0.5); // cyan
const SELECTED_CONTROL_COLOR: Color = (0.5, 0.5, 0.); // yellow
const SELECTED_CV_COLOR: Color = (0.5, 0., 0.5); // magenta
const SELECTED_IO_COLOR: Color = (0.1, 0.1, 0.1);
const VALUE_COLOR: Color = (0.5, 0.5, 1.); // cyan
const SUP_COLOR: Color = (HFF, H00, H00); // red
const ADD_COLOR: Color = (0., 1., 0.); // green

pub fn background(cc: &Context) {
    set_color(cc, BACKGROUND_COLOR);
}
pub fn selected_io_background(cc: &Context) {
    set_color(cc, SELECTED_IO_BACKGROUND_COLOR);
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
    set_color(cc, SELECTED_BOX_BACKGROUND_COLOR);
}
pub fn selected(cc: &Context) {
    selected_io_background(cc);
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
pub fn audio(cc: &Context) {
    set_color(cc, AUDIO_COLOR);
    cc.set_font_size(FONT_SIZE);
}
pub fn control(cc: &Context) {
    set_color(cc, CONTROL_COLOR);
    cc.set_font_size(FONT_SIZE);
}
pub fn cv(cc: &Context) {
    set_color(cc, CV_COLOR);
    cc.set_font_size(FONT_SIZE);
}
pub fn io(cc: &Context) {
    set_color(cc, IO_COLOR);
    cc.set_font_size(FONT_SIZE);
}
pub fn selected_audio(cc: &Context) {
    set_color(cc, SELECTED_AUDIO_COLOR);
    cc.set_font_size(FONT_SIZE);
}
pub fn selected_control(cc: &Context) {
    set_color(cc, SELECTED_CONTROL_COLOR);
    cc.set_font_size(FONT_SIZE);
}
pub fn selected_cv(cc: &Context) {
    set_color(cc, SELECTED_CV_COLOR);
    cc.set_font_size(FONT_SIZE);
}
pub fn selected_io(cc: &Context) {
    set_color(cc, SELECTED_IO_COLOR);
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
