use std::marker::PhantomData;
use core::uri::UriBound;

use urid::URID;
use urid::features::URIDMap;

use std::fmt::{Debug, Formatter, Error};

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct URIDOf<U: UriBound> {
    urid: URID,
    _phantom: PhantomData<U>
}

impl<U: UriBound> URIDOf<U> {
    pub fn map(mapper: &URIDMap) -> Self {
        Self {
            urid: mapper.map(U::uri()).unwrap(), // TODO: remove unwrap
            _phantom: PhantomData
        }
    }

    #[inline]
    pub fn urid(&self) -> URID {
        self.urid
    }
}

impl<U: UriBound> Copy for URIDOf<U> {}
impl<U: UriBound> Clone for URIDOf<U> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<U: UriBound> Debug for URIDOf<U> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "URIDOf<{}>({})", U::uri(), self.urid)
    }
}

impl<U: UriBound> AsRef<URID> for URIDOf<U> {
    #[inline]
    fn as_ref(&self) -> &URID {
        &self.urid
    }
}

impl<U: UriBound> Into<URID> for URIDOf<U> {
    #[inline]
    fn into(self) -> URID {
        self.urid
    }
}

impl<T, U: UriBound> PartialEq<T> for URIDOf<U> where URID: PartialEq<T> {
    #[inline]
    fn eq(&self, other: &T) -> bool {
        self.urid == *other
    }
}

impl<T, U: UriBound> PartialOrd<T> for URIDOf<U> where URID: PartialOrd<T> {
    #[inline]
    fn partial_cmp(&self, other: &T) -> Option<std::cmp::Ordering> {
        PartialOrd::partial_cmp(&self.urid, other)
    }
}
