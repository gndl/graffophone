use atom::header::AtomType;
use atom::types::{AtomSequence, UnknownAtomSequence};
use urid::{URID, URIDCache, URIDCacheMapping};
use std::slice;
use units::Unit;
use std::marker::PhantomData;
use atom::header::Atom;
use atom::into::ToAtom;
use std::mem;

pub trait Forger<'c, C: 'c> :  Sized{
    fn cache(&self) -> &'c C;

    unsafe fn write_raw_padded(&mut self, data: *const u8, size: usize) -> *mut u8;

    #[inline]
    fn get_urid<T: AtomType>(&self) -> URID where C: URIDCacheMapping<T> {
        URIDCacheMapping::get_urid(self.cache()).urid()
    }

    #[inline]
    fn write_atom<T: AtomType, A: ToAtom<AtomType=T>>(&mut self, atom: A) where C: URIDCacheMapping<T> {
        let atom = &atom.to_atom();
        let dst_atom = unsafe { write_atom_inner(self, atom) };
        dst_atom.update_type_id(self.cache());
    }
}

impl<'c, C: URIDCache + 'c> Forger<'c, C> for Forge<'c, C> {
    #[inline]
    fn cache(&self) -> &'c C {
        self.cache
    }

    unsafe fn write_raw_padded(&mut self, data: *const u8, size: usize) -> *mut u8 {
        let end = self.position + size;
        let dst_slice = &mut self.buffer[self.position..end];

        let atom_data = slice::from_raw_parts(data, size);
        dst_slice.copy_from_slice(atom_data);
        self.position += ::lv2_sys::lv2_atom_pad_size(size as u32) as usize;

        dst_slice.as_mut_ptr()
    }
}

// TODO: add error handling

pub struct Forge<'c, C: URIDCache + 'c> {
    buffer: &'c mut [u8],
    cache: &'c C,
    position: usize
}

#[inline]
unsafe fn write_atom_inner<'c, 'x, A: AtomType, F: Forger<'c, C>, C: URIDCache + 'c>(forge: &mut F, atom: &A) -> &'x mut A {
    let size = atom.get_total_size() as usize;
    let data = atom as *const A as *const u8;
    &mut *(forge.write_raw_padded(data, size) as *mut A)
}

impl<'c, C: URIDCache> Forge<'c, C> {
    #[inline]
    pub fn new(buffer: &'c mut [u8], cache: &'c C) -> Forge<'c, C> {
        Forge {
            buffer, cache, position: 0
        }
    }

    #[inline]
    pub fn begin_sequence<'a, U: Unit + 'c>(&'a mut self) -> ForgeSequence<'a, 'c, C, U, Self> where C: URIDCacheMapping<U> + URIDCacheMapping<UnknownAtomSequence> {
        ForgeSequence::new(self)
    }
}

pub struct ForgeSequence<'a, 'c, C: 'c, U: Unit, F: Forger<'c, C> + 'a> {
    parent: &'a mut F,
    atom: &'a mut Atom,
    _unit_type: PhantomData<U>,
    _cache_type: PhantomData<&'c C>
}

impl<'a, 'c, C: URIDCache + 'c, U: Unit + 'c, F: Forger<'c, C> + 'a> ForgeSequence<'a, 'c, C, U, F> {
    #[inline]
    fn new<'b>(parent: &'b mut F) -> ForgeSequence<'b, 'c, C, U, F> where C: URIDCacheMapping<UnknownAtomSequence> + URIDCacheMapping<U>, U: 'b {
        let seq_header = AtomSequence::<U>::new_header(parent.cache());
        let atom = unsafe { write_atom_inner(parent, &seq_header) }.get_header_mut();

        ForgeSequence {
            parent,
            atom,
            _unit_type: PhantomData,
            _cache_type: PhantomData
        }
    }

    #[inline]
    pub fn write_event<T: AtomType, A: ToAtom<AtomType=T>>(&mut self, time: &U, atom: A) where C: URIDCacheMapping<T> {
        unsafe {
            self.write_raw_padded(time as *const U as *const u8, mem::size_of::<U>());
        }
        self.write_atom(atom);
    }

    #[inline]
    pub fn begin_sequence<'b, U2: Unit + 'c>(&'b mut self, time: &U) -> ForgeSequence<'b, 'c, C, U2, Self>
        where C: URIDCacheMapping<U2> + URIDCacheMapping<UnknownAtomSequence> + 'b
    {
        unsafe {
            self.write_raw_padded(time as *const U as *const u8, mem::size_of::<U>());
        }

        ForgeSequence::new(self)
    }
}


impl<'a, 'c, C: URIDCache + 'c, U: Unit, F: Forger<'c, C> + 'a> Forger<'c, C> for ForgeSequence<'a, 'c, C, U, F> {
    #[inline]
    fn cache(&self) -> &'c C {
        self.parent.cache()
    }

    unsafe fn write_raw_padded(&mut self, data: *const u8, size: usize) -> *mut u8 {
        self.atom.add_size(::lv2_sys::lv2_atom_pad_size(size as u32));
        self.parent.write_raw_padded(data, size)
    }
}