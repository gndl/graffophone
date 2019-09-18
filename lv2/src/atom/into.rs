use atom::header::AtomType;

pub trait ToAtom {
    type AtomType: AtomType;

    fn to_atom(&self) -> Self::AtomType;
}

impl<'a, T: ToAtom> ToAtom for &'a T {
    type AtomType = T::AtomType;

    #[inline]
    fn to_atom(&self) -> <Self as ToAtom>::AtomType {
        T::to_atom(*self)
    }
}

impl<'a, T: ToAtom> ToAtom for &'a mut T {
    type AtomType = T::AtomType;

    #[inline]
    fn to_atom(&self) -> <Self as ToAtom>::AtomType {
        T::to_atom(*self)
    }
}
