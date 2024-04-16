use std::collections::HashMap;

use zonbi::{Cage, Zonbi, ZonbiId};

#[derive(Debug)]
struct NonCopyI32(i32);

struct MyStruct<'a> {
    val: &'a NonCopyI32,
}

unsafe impl<'a> Zonbi for MyStruct<'a> {
    type Casted<'z> = MyStruct<'z>;

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

    let mut type_map: HashMap<ZonbiId, Box<Cage<'a, dyn Zonbi>>> = HashMap::new();
    let zonbi_id = ZonbiId::of::<MyStruct>();
    type_map.insert(zonbi_id, Box::new(Cage::new(my_struct)));

    type_map[&zonbi_id].represents::<MyStruct<'a>>();
    //let my_ref = unsafe { type_map[&zonbi_id].downcast_ref::<MyStruct<'a>>().unwrap() };
}

// Try commenting the code out
// fn fails<'a>(a: &'a NonCopyI32) {
//     use std::any::{Any, TypeId};

//     let my_struct = MyStruct { val: a };

//     let mut type_map: HashMap<TypeId, Box<dyn Any>> = HashMap::new();
//     type_map.insert(TypeId::of::<MyStruct>(), Box::new(my_struct));
// }
