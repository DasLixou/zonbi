# Zonbi

This is an experiment to make it possible to type-erase non-`'static` types.

## How it works

With `#[derive(Zonbi)]`, the type gets an implementation for getting a version of the type where all lifetimes are replaced with the given one.
Manual implementation is unsafe because the user must assure that the `Casted<'z>` type is the same as the one of the implementer.

This is used in `ZonbiId`, a wrapper around `TypeId`, which different from it's inside value, has the additional definition of behaviour for non-`'static` types.
`ZonbiId` is unique for every type, **excluding** its lifetimes. 
Under the hood, it just uses the `Zonbi` trait to get the `'static` version of the type and gets its `TypeId`.

For saving this in a Box, we can use `BoxCage<'life>`. 
> _I tried to not force the wrapper type, so it's also possible to for example use `Rc` or `Arc`, but I couldn't get it running. Help appreciated :D_

The cage has an associated lifetime and can hold any type whose lifetimes are equal or longer than its assocaited one.
When downcasting the cage back to its type, we get a version of the type back where all the lifetimes are replaced with `'life` of the cage, thus being safe. 

## License

Dual-licensed under [`Apache-2.0`](LICENSE-APACHE) and [`MIT`](LICENSE-MIT)