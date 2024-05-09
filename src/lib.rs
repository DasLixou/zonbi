///! # Zonbi
///!
///! This is an experiment to make it possible to type-erase non-`'static` types.
///!
///! ## How it works
///!
///! With `#[derive(Zonbi)]`, the type gets an implementation for getting a version of the type where all lifetimes are replaced with the given one.
///! Manual implementation is unsafe because the user must assure that the `Casted` type is the same as the one of the implementer.
///!
///! This is used in `ZonbiId`, a wrapper around `TypeId`, which different from its inside value, has the additional definition of behaviour for non-`'static` types.
///! `ZonbiId` is unique for every type, **excluding** its lifetimes.
///! Under the hood, it just uses the `Zonbi` trait to get the `'static` version of the type and gets its `TypeId`.
///!
///! To hold such type-erased value inside for example a box, you can create a `Cage<'life, Z>` of the zonbi `Z` and then hold that in a `dyn AnyZonbi<'life>` with the associated minimal lifetime.
///! Every zonbi that lives for at least `'life` can be upcasted into this trait and downcasted back into it with all the lifetimes being this mininal `'life` one.
///!
///! ## Example
///!
///! ```ignore
///! let mut type_map: HashMap<ZonbiId, Box<dyn AnyZonbi<'a>>> = HashMap::new();
///!
///! let id = ZonbiId::of::<MyStruct>();
///! type_map.insert(id, Cage::new(Box::new(MyStruct { my_reference: &val })));
///!
///! let r: &MyStruct<'a> = type_map[&id].downcast_ref::<MyStruct<'a>>().unwrap();
///! ```
///!
///! _This is a broken down snippet of the [`type_map` example](https://github.com/DasLixou/zonbi/blob/master/examples/type_map.rs)._
///!
///! ## License
///!
///! Dual-licensed under [`Apache-2.0`](LICENSE-APACHE) and [`MIT`](LICENSE-MIT)
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

/// The trait implemented by [`Cage`] to blur its generic [`Zonbi`].
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

/// Holds a [`Zonbi`] with the `'life` lifetime and can be casted into a [`dyn AnyZonbi`].
///
/// This is done because it forbids to have a lower-lifetimed mutable reference to the zonbi,
/// preventing a possible use-after-free reference.
/// See [this comment][https://internals.rust-lang.org/t/type-erasing-non-static-types/20785/2] for more.
///
/// [`dyn AnyZonbi`]: AnyZonbi
pub struct Cage<'life, Z: Zonbi<'life>> {
    val: Z,
    phantom: PhantomData<&'life ()>,
}

impl<'life, Z: Zonbi<'life>> Cage<'life, Z> {
    /// Creates a new `Cage` for a zonbi.
    pub fn new(val: Z) -> Self {
        Self {
            val,
            phantom: PhantomData,
        }
    }

    /// Returns the inner zonbi of this cage with a lifetime of `'life`
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
