use std::{any::TypeId, intrinsics::transmute_unchecked, marker::PhantomData};

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

pub unsafe trait Zonbi {
    type Casted<'z>: Sized + Zonbi + 'z
    where
        Self: Sized;

    fn zonbi_id(&self) -> ZonbiId;
}

impl dyn Zonbi {
    /// Returns `true` if the inner type represents the same as `T`,
    /// **excluding** its lifetimes.
    pub fn represents<T: Zonbi>(&self) -> bool {
        ZonbiId::of::<T>().eq(&self.zonbi_id())
    }

    pub unsafe fn downcast_ref<T: Zonbi>(&self) -> Option<&T> {
        if self.represents::<T>() {
            unsafe { Some(self.downcast_ref_unchecked()) }
        } else {
            None
        }
    }

    pub unsafe fn downcast_ref_unchecked<T: Zonbi>(&self) -> &T {
        // SAFETY: caller guarantees that T is the correct type
        unsafe { &*(self as *const dyn Zonbi as *const T) }
    }
}

#[repr(transparent)]
pub struct Cage<'life, Z: 'static + Zonbi + ?Sized> {
    life: PhantomData<&'life ()>,
    zonbi: Z,
}

impl<'life, Z: Zonbi> Cage<'life, Z> {
    pub fn new<V: Sized + Zonbi + 'life>(val: V) -> Self
    where
        Z: Zonbi<Casted<'life> = V>,
        V: Zonbi<Casted<'static> = Z>,
    {
        let as_static = unsafe { transmute_unchecked::<V, V::Casted<'static>>(val) };
        Self {
            life: PhantomData,
            zonbi: as_static,
        }
    }
}

impl<'life> Cage<'life, dyn Zonbi> {
    pub fn represents<T: Zonbi>(&self) -> bool {
        self.zonbi.represents::<T>()
    }

    pub fn downcast_ref<T: Zonbi>(&self) -> Option<&T::Casted<'life>> {
        unsafe { self.zonbi.downcast_ref::<T::Casted<'life>>() }
    }
}
