#![doc = include_str!("../README.md")]

use std::{any::TypeId, marker::PhantomData};

pub use zonbi_macros::Zonbi;

/// A `ZonbiId` represents a globally unique identifier for the `'static` version of a type.
///
/// Because `ZonbiId` relies on rust's [`TypeId`], please keep their warning in mind
/// that the hashes and ordering will vary between Rust releases. Beware
/// of relying on them inside of your code!
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct ZonbiId(TypeId);

impl ZonbiId {
    /// Returns the `ZonbiId` of the given generic type.
    ///
    /// This is equal to the [`TypeId`] of the `'static` version of the type.
    pub fn of<'life, Z: Zonbi<'life>>() -> ZonbiId {
        Z::zonbi_id()
    }
}

impl From<TypeId> for ZonbiId {
    fn from(value: TypeId) -> Self {
        Self(value) // creating a `ZonbiId` from a `TypeId` should be safe, but not vise versa.
    }
}

/// A trait to make a `'life`-lifetimed version of a type.
///
/// It is unsafe to implement because we can't assure that the `Casted` associated type
/// is the "same" (ignoring lifetimes) as the implementor.
pub unsafe trait Zonbi<'life>: 'life {
    /// A version of this type where all lifetimes are replaced with `'life` so it lives for `'life`.
    type Casted: Sized + Zonbi<'life>;

    /// Returns the `ZonbiId` of this type.
    fn zonbi_id() -> ZonbiId;

    unsafe fn zonbify(self) -> Self::Casted;
    unsafe fn zonbify_ref(&self) -> &Self::Casted;
    unsafe fn zonbify_mut(&mut self) -> &mut Self::Casted;
}

mod private {
    pub trait Sealed {}
}

pub trait AnyZonbi<'life>: private::Sealed + 'life {
    /// Returns the `ZonbiId` of this erased type.
    ///
    /// This needs a shared reference to self in order to be object-safe.
    /// If you have a concrete type without an instance of it, use [`Zonbi::zonbi_id`] instead.
    fn zonbi_id(&self) -> ZonbiId;

    unsafe fn get_raw(&self) -> *const ();
    unsafe fn get_raw_mut(&mut self) -> *mut ();
}

impl<'life> dyn AnyZonbi<'life> {
    /// Returns `true` if the inner type represents the same as `T`,
    /// **excluding** its lifetimes.
    pub fn represents<Z: Zonbi<'life>>(&self) -> bool {
        ZonbiId::of::<Z>().eq(&self.zonbi_id())
    }

    /// Returns a shared reference to the inner value with a lifetime of `'life`
    /// if it represents the zonbi `Z`, or `None` if it doesn't.
    pub fn downcast_ref<Z: Zonbi<'life>>(&self) -> Option<&Z::Casted> {
        if self.represents::<Z>() {
            Some(unsafe { self.downcast_ref_unchecked::<Z>() })
        } else {
            None
        }
    }

    pub unsafe fn downcast_ref_unchecked<Z: Zonbi<'life>>(&self) -> &Z::Casted {
        // SAFETY: caller guarantees that Z is the correct type
        let raw = unsafe { &*self.get_raw().cast::<Z>() };
        Z::zonbify_ref(raw)
    }

    /// Returns an exclusive reference to the inner value with a lifetime of `'life`
    /// if it represents the zonbi `Z`, or `None` if it doesn't.
    pub fn downcast_mut<Z: Zonbi<'life>>(&mut self) -> Option<&mut Z::Casted> {
        if self.represents::<Z>() {
            Some(unsafe { self.downcast_mut_unchecked::<Z>() })
        } else {
            None
        }
    }

    pub unsafe fn downcast_mut_unchecked<Z: Zonbi<'life>>(&mut self) -> &mut Z::Casted {
        // SAFETY: caller guarantees that Z is the correct type
        let raw = unsafe { &mut *self.get_raw_mut().cast::<Z>() };
        Z::zonbify_mut(raw)
    }
}

pub struct Cage<'life, Z: Zonbi<'life>> {
    val: Z,
    phantom: PhantomData<&'life ()>,
}

impl<'life, Z: Zonbi<'life>> Cage<'life, Z> {
    pub fn new(val: Z) -> Self {
        Self {
            val,
            phantom: PhantomData,
        }
    }

    pub fn into_inner(self) -> Z::Casted {
        unsafe { Z::zonbify(self.val) }
    }
}

impl<'life, Z: Zonbi<'life>> private::Sealed for Cage<'life, Z> {}

impl<'life, Z: Zonbi<'life>> AnyZonbi<'life> for Cage<'life, Z> {
    fn zonbi_id(&self) -> ZonbiId {
        Z::zonbi_id()
    }

    unsafe fn get_raw(&self) -> *const () {
        (&self.val as *const Z).cast()
    }

    unsafe fn get_raw_mut(&mut self) -> *mut () {
        (&mut self.val as *mut Z).cast()
    }
}
