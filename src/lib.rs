use std::{any::TypeId, marker::PhantomData};

/// A `ZonbiId` represents a globally unique identifier for the `'static` version of a type.
///
/// Because `ZonbiId` relies on rust's [`TypeId`], please keep their warning in mind
/// that the hashes and ordering will vary between Rust releases. Beware
/// of relying on them inside of your code!
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct ZonbiId(TypeId);

impl ZonbiId {
    /// Returns the `ZonbiId` of the given generic type
    pub fn of<Z: Zonbi>() -> ZonbiId {
        ZonbiId(TypeId::of::<Z::Casted<'static>>())
    }
}

pub unsafe trait Zonbi: AnyZonbi {
    type Casted<'z>: Sized + Zonbi + 'z;

    unsafe fn zonbify<'z>(self) -> Self::Casted<'z>;
    unsafe fn zonbify_ref<'z>(&self) -> &Self::Casted<'z>;
}

pub trait AnyZonbi {
    fn zonbi_id(&self) -> ZonbiId;
}

impl dyn AnyZonbi {
    /// Returns `true` if the inner type represents the same as `T`,
    /// **excluding** its lifetimes.
    pub fn represents<Z: Zonbi>(&self) -> bool {
        ZonbiId::of::<Z>().eq(&self.zonbi_id())
    }

    pub unsafe fn downcast_ref<'a, 'z, Z: Zonbi + 'a>(&'a self) -> Option<&Z::Casted<'z>> {
        if self.represents::<Z>() {
            let raw = unsafe { self.downcast_ref_unchecked::<Z>() };
            Some(raw)
        } else {
            None
        }
    }

    pub unsafe fn downcast_ref_unchecked<'a, 'z, Z: Zonbi + 'a>(&'a self) -> &Z::Casted<'z> {
        // SAFETY: caller guarantees that T is the correct type
        let raw = unsafe { &*(self as *const dyn AnyZonbi as *const Z) };
        Z::zonbify_ref(raw)
    }
}

#[repr(transparent)]
pub struct Cage<'life> {
    phantom: PhantomData<&'life ()>,
    c: Box<dyn AnyZonbi>,
}

impl<'life> Cage<'life> {
    pub fn new<Z: Zonbi>(zonbi: Z) -> Self {
        Self {
            phantom: PhantomData,
            c: Box::new(unsafe { zonbi.zonbify() }),
        }
    }

    pub fn downcast_ref<'a, Z: Zonbi + 'a>(&'a self) -> Option<&Z::Casted<'life>> {
        unsafe { self.c.downcast_ref::<'a, 'life, Z>() }
    }
}
