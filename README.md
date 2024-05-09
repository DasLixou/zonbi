# Zonbi

This is an experiment to make it possible to type-erase non-`'static` types.

## How it works

With `#[derive(Zonbi)]`, the type gets an implementation for getting a version of the type where all lifetimes are replaced with the given one.
Manual implementation is unsafe because the user must assure that the `Casted` type is the same as the one of the implementer.

This is used in `ZonbiId`, a wrapper around `TypeId`, which different from its inside value, has the additional definition of behaviour for non-`'static` types.
`ZonbiId` is unique for every type, **excluding** its lifetimes.
Under the hood, it just uses the `Zonbi` trait to get the `'static` version of the type and gets its `TypeId`.

To hold such type-erased value inside for example a box, you can create a `Cage<'life, Z>` of the zonbi `Z` and then hold that in a `dyn AnyZonbi<'life>` with the associated minimal lifetime.
Every zonbi that lives for at least `'life` can be upcasted into this trait and downcasted back into it with all the lifetimes being this mininal `'life` one.

## Example

```rs
let mut type_map: HashMap<ZonbiId, Box<dyn AnyZonbi<'a>>> = HashMap::new();

let id = ZonbiId::of::<MyStruct>();
type_map.insert(id, Cage::new(Box::new(MyStruct { my_reference: &val })));

let r: &MyStruct<'a> = type_map[&id].downcast_ref::<MyStruct<'a>>().unwrap();
```

_This is a broken down snippet of the [`type_map` example](examples/type_map.rs)._

## License

Dual-licensed under [`Apache-2.0`](LICENSE-APACHE) and [`MIT`](LICENSE-MIT)
