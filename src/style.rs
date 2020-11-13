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
/*
pub struct Color {
    r: f64,
    g: f64,
    b: f64,
}
impl Color {
    pub fn new(r: f64, g: f64, b: f64) -> Color {
        Self { r, g, b }
    }
}
pub fn set_color(cc: &Context, color: &Color) {
    cc.set_source_rgb(color.r, color.g, color.b);
}
const BACKGROUND_COLOR: Color = Color {
    r: 0.,
    g: 0.,
    b: 0.,
};
*/

type Color = (f64, f64, f64);

pub fn set_color(cc: &Context, (r, g, b): Color) {
    cc.set_source_rgb(r, g, b);
}

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
const NAME_COLOR: Color = (HFF, HFF, HFF); // white
const MODEL_COLOR: Color = (HD3, HD3, HD3); // lightgray
const VALUE_COLOR: Color = (H00, HFF, HFF); // cyan
const EAR_COLOR: Color = (HFF, HFF, H00); // yellow
const SELECTED_EAR_COLOR: Color = (HFF, H00, HFF); // magenta
const VOICE_COLOR: Color = (H90, HEE, H90); // lightgreen
const SELECTED_VOICE_COLOR: Color = (HFF, H00, HFF); // magenta
const ADD_COLOR: Color = (H90, HEE, H90); // lightgreen
const SUP_COLOR: Color = (HFF, H00, H00); // lightgreen

pub fn background(cc: &Context) {
    set_color(cc, BACKGROUND_COLOR);
}
pub fn delimitation(cc: &Context) {
    set_color(cc, DELIMITATION_COLOR);
}
pub fn selection(cc: &Context) {
    set_color(cc, SELECTION_COLOR);
}
pub fn reverse_selection(cc: &Context) {
    set_color(cc, REVERSE_SELECTION_COLOR);
}
pub fn box_background(cc: &Context) {
    set_color(cc, BOX_BACKGROUND_COLOR);
}
pub fn box_border(cc: &Context) {
    set_color(cc, BOX_BORDER_COLOR);
    cc.set_line_width(0.5);
}
pub fn selected_box(cc: &Context) {
    set_color(cc, SELECTION_COLOR);
}

pub fn name(cc: &Context) {
    set_color(cc, NAME_COLOR);
}
pub fn model(cc: &Context) {
    set_color(cc, MODEL_COLOR);
    // cc.select_font_face("Sans", FontSlant::Normal, FontWeight::Normal);
    // cc.set_font_size(12.);
}
pub fn value(cc: &Context) {
    set_color(cc, VALUE_COLOR);
}
pub fn ear(cc: &Context) {
    set_color(cc, EAR_COLOR);
}
pub fn selected_ear(cc: &Context) {
    set_color(cc, SELECTED_EAR_COLOR);
}
pub fn voice(cc: &Context) {
    set_color(cc, VOICE_COLOR);
}
pub fn selected_voice(cc: &Context) {
    set_color(cc, SELECTED_VOICE_COLOR);
}
pub fn add(cc: &Context) {
    set_color(cc, ADD_COLOR);
}
pub fn sup(cc: &Context) {
    set_color(cc, SUP_COLOR);
}

//pub fn flow            (cc: &Context,) {set_color(cc, _COLOR);} //Color.ofString "0x00FFFFFF" /* "cyan" */
//pub fn marker          (cc: &Context,) {set_color(cc, _COLOR);} //Color.ofString "0xFF8000FF" /* "orange" */
/*
pub fn makeVoiceColor voice =
  let v = (Voice.getTalker voice)#getId + (Voice.getPort voice lsl 14) in
  let r = 95 + (v mod 3) * 80 in let g = 95 + (v mod 5) * 40 in let b = 75 + (v mod 7) * 30 in
  Color.ofRgb8 r g b
*/
