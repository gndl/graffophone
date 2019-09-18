use super::*;
use std::mem::size_of;

#[inline]
pub unsafe fn lv2_atom_pad_size(size: u32) -> u32 {
    (size + 7u32) & (!7u32)
}

#[inline]
pub unsafe fn lv2_atom_sequence_begin(body: *const LV2_Atom_Sequence_Body) -> *mut LV2_Atom_Event {
    body.offset(1) as _
}

#[inline]
pub unsafe fn lv2_atom_sequence_is_end(body: *const LV2_Atom_Sequence_Body, size: u32, i: *const LV2_Atom_Event) -> bool {
    (i as *const u8) >= (body as *const u8).offset(size as isize)
}

#[inline]
pub unsafe fn lv2_atom_sequence_next(event: *const LV2_Atom_Event) -> *mut LV2_Atom_Event {
    (event as *mut u8)
        .offset(size_of::<LV2_Atom_Event>() as isize)
        .offset(lv2_atom_pad_size((*event).body.size) as isize) as _
}
