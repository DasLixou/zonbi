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
    unsafe fn zonbify_mut<'z>(&mut self) -> &mut Self::Casted<'z>;
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

    /// Returns a shared reference to the inner value with a lifetime of `'z`
    /// if it represents the zonbi `Z`, or `None` if it doesn't.
    pub unsafe fn downcast_ref<'a, 'z, Z: Zonbi + 'a>(&'a self) -> Option<&Z::Casted<'z>> {
        if self.represents::<Z>() {
            Some(unsafe { self.downcast_ref_unchecked::<Z>() })
        } else {
            None
        }
    }

    pub unsafe fn downcast_ref_unchecked<'a, 'z, Z: Zonbi + 'a>(&'a self) -> &Z::Casted<'z> {
        // SAFETY: caller guarantees that Z is the correct type
        let raw = unsafe { &*(self as *const dyn AnyZonbi as *const Z) };
        Z::zonbify_ref(raw)
    }

    /// Returns an exclusive reference to the inner value with a lifetime of `'z`
    /// if it represents the zonbi `Z`, or `None` if it doesn't.
    pub unsafe fn downcast_mut<'a, 'z, Z: Zonbi + 'a>(&'a mut self) -> Option<&mut Z::Casted<'z>> {
        if self.represents::<Z>() {
            Some(unsafe { self.downcast_mut_unchecked::<Z>() })
        } else {
            None
        }
    }

    pub unsafe fn downcast_mut_unchecked<'a, 'z, Z: Zonbi + 'a>(
        &'a mut self,
    ) -> &mut Z::Casted<'z> {
        // SAFETY: caller guarantees that Z is the correct type
        let raw = unsafe { &mut *(self as *mut dyn AnyZonbi as *mut Z) };
        Z::zonbify_mut(raw)
    }
}

/// A wrapper over a `Box`, containing a `dyn AnyZonbi`.
/// It is marked with a `'life` lifetime, which it's data can't underlive.
#[repr(transparent)]
pub struct BoxCage<'life> {
    phantom: PhantomData<&'life ()>,
    c: Box<dyn AnyZonbi>,
}

impl<'life> BoxCage<'life> {
    /// Creates a new `BoxCage` over a zonbi value which must at least live for `'life`
    pub fn new<Z: Zonbi + 'life>(zonbi: Z) -> Self {
        Self {
            phantom: PhantomData,
            c: Box::new(unsafe { zonbi.zonbify() }),
        }
    }

    /// Returns a shared reference to the inner value with a lifetime of `'life`
    /// if it represents the zonbi `Z`, or `None` if it doesn't.
    pub fn downcast_ref<'a, Z: Zonbi + 'a>(&'a self) -> Option<&Z::Casted<'life>> {
        unsafe { self.c.downcast_ref::<'a, 'life, Z>() }
    }

    /// Returns an exclusive reference to the inner value with a lifetime of `'life`
    /// if it represents the zonbi `Z`, or `None` if it doesn't.
    pub fn downcast_mut<'a, Z: Zonbi + 'a>(&'a mut self) -> Option<&mut Z::Casted<'life>> {
        unsafe { self.c.downcast_mut::<'a, 'life, Z>() }
    }
}
