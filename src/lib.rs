use std::any::TypeId;

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

/// A trait to make different-lifetimed versions of a type.
///
/// It is unsafe to implement because we can't assure that the `Cased<'z>` associated type
/// is the "same" (ignoring lifetimes) as the implementor.
pub unsafe trait Zonbi<'z>: AnyZonbi<'z> {
    /// A version of this type where all lifetimes are replaced with `'z` so it lives for `'z`.
    type Casted: Sized + Zonbi<'z>;

    fn zonbi_id() -> ZonbiId;

    unsafe fn zonbify(self) -> Self::Casted;
    unsafe fn zonbify_ref(&self) -> &Self::Casted;
    unsafe fn zonbify_mut(&mut self) -> &mut Self::Casted;
}

pub trait AnyZonbi<'life>: 'life {
    /// Returns the `ZonbiId` of the given generic type.
    fn erased_zonbi_id(&self) -> ZonbiId;
}

impl<'life, Z> AnyZonbi<'life> for Z
where
    Z: Zonbi<'life>,
{
    fn erased_zonbi_id(&self) -> ZonbiId {
        Z::zonbi_id()
    }
}

impl<'life> dyn AnyZonbi<'life> {
    /// Returns `true` if the inner type represents the same as `T`,
    /// **excluding** its lifetimes.
    pub fn represents<Z: Zonbi<'life>>(&self) -> bool {
        ZonbiId::of::<Z>().eq(&self.erased_zonbi_id())
    }

    /// Returns a shared reference to the inner value with a lifetime of `'z`
    /// if it represents the zonbi `Z`, or `None` if it doesn't.
    pub fn downcast_ref<'a, Z: Zonbi<'life> + 'a>(&'a self) -> Option<&Z::Casted> {
        if self.represents::<Z>() {
            Some(unsafe { self.downcast_ref_unchecked::<Z>() })
        } else {
            None
        }
    }

    pub unsafe fn downcast_ref_unchecked<'a, Z: Zonbi<'life> + 'a>(&'a self) -> &Z::Casted {
        // SAFETY: caller guarantees that Z is the correct type
        let raw = unsafe { &*(self as *const dyn AnyZonbi as *const Z) };
        Z::zonbify_ref(raw)
    }

    /// Returns an exclusive reference to the inner value with a lifetime of `'z`
    /// if it represents the zonbi `Z`, or `None` if it doesn't.
    pub fn downcast_mut<'a, Z: Zonbi<'life> + 'a>(&'a mut self) -> Option<&mut Z::Casted> {
        if self.represents::<Z>() {
            Some(unsafe { self.downcast_mut_unchecked::<Z>() })
        } else {
            None
        }
    }

    pub unsafe fn downcast_mut_unchecked<'a, Z: Zonbi<'life> + 'a>(&'a mut self) -> &mut Z::Casted {
        // SAFETY: caller guarantees that Z is the correct type
        let raw = unsafe { &mut *(self as *mut dyn AnyZonbi as *mut Z) };
        Z::zonbify_mut(raw)
    }
}

/// A wrapper over a `Box`, containing a `dyn AnyZonbi`.
/// It is marked with a `'life` lifetime, which it's data can't underlive.
#[repr(transparent)]
pub struct BoxCage<'life> {
    c: Box<dyn AnyZonbi<'life> + 'life>,
}

impl<'life> BoxCage<'life> {
    /// Creates a new `BoxCage` over a zonbi value which must at least live for `'life`
    pub fn new<Z: Zonbi<'life> + 'life>(zonbi: Z) -> Self {
        Self { c: Box::new(zonbi) }
    }

    /// Returns a shared reference to the inner value with a lifetime of `'life`
    /// if it represents the zonbi `Z`, or `None` if it doesn't.
    pub fn downcast_ref<'a, Z: Zonbi<'life> + 'a>(&'a self) -> Option<&Z::Casted> {
        self.c.downcast_ref::<'a, Z>()
    }

    /// Returns an exclusive reference to the inner value with a lifetime of `'life`
    /// if it represents the zonbi `Z`, or `None` if it doesn't.
    pub fn downcast_mut<'a, Z: Zonbi<'life> + 'a>(&'a mut self) -> Option<&mut Z::Casted> {
        self.c.downcast_mut::<'a, Z>()
    }
}
