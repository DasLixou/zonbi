use std::collections::HashMap;

use zonbi::{AnyZonbi, Cage, Zonbi, ZonbiId};

#[derive(Debug)]
struct NonCopyI32(i32);

struct MyStruct<'a> {
    val: &'a NonCopyI32,
}

unsafe impl<'a> Zonbi for MyStruct<'a> {
    type Casted<'z> = MyStruct<'z>;

    unsafe fn zonbify<'z>(self) -> Self::Casted<'z> {
        core::mem::transmute(self)
    }

    unsafe fn zonbify_ref<'z>(&self) -> &Self::Casted<'z> {
        core::mem::transmute(self)
    }
}

impl<'a> AnyZonbi for MyStruct<'a> {
    fn zonbi_id(&self) -> ZonbiId {
        ZonbiId::of::<Self>()
    }
}

fn main() {
    let a = NonCopyI32(42);

    with_zonbi(&a);
}

fn with_zonbi<'a>(a: &'a NonCopyI32) {
    let my_struct = MyStruct { val: a };

    let mut type_map: HashMap<ZonbiId, Cage<'a>> = HashMap::new();
    let id = ZonbiId::of::<MyStruct>();
    type_map.insert(id, Cage::new(my_struct));

    let r: &MyStruct<'a> = type_map[&id].downcast_ref::<'_, MyStruct<'a>>().unwrap();
    println!("{:?}", r.val);
}

// Try commenting the code out
// fn fails<'a>(a: &'a NonCopyI32) {
//     use std::any::{Any, TypeId};

//     let my_struct = MyStruct { val: a };

//     let mut type_map: HashMap<TypeId, Box<dyn Any>> = HashMap::new();
//     let id = TypeId::of::<MyStruct>();
//     type_map.insert(id, Box::new(my_struct));
// }
