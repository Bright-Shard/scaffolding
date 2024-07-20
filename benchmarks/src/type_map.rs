#![allow(dead_code, unused_imports)]

use {
    scaffolding::{datatypes::TypeMap, utils::Hashnt},
    std::{
        any::{Any, TypeId},
        collections::HashMap,
        hint::black_box,
        rc::Rc,
    },
    test::{bench, Bencher},
};

#[derive(Debug)]
pub struct OsuMoment {
    num: u32,
    text: String,
}
impl Default for OsuMoment {
    fn default() -> Self {
        Self {
            num: 727,
            text: String::from("WYSI"),
        }
    }
}
#[derive(Default, Debug)]
pub struct EmptyStruct {}
#[derive(Debug)]
pub enum Enum {
    Default,
    OtherVariant,
}

// Normal Insertions

#[bench]
fn typemap(b: &mut Bencher) {
    b.iter(|| {
        let mut map = TypeMap::new(3, 100);
        map.insert(OsuMoment::default());
        map.insert(EmptyStruct::default());
        map.insert(Enum::Default);
    });
}
#[bench]
fn hash_map(b: &mut Bencher) {
    b.iter(|| {
        let mut map = HashMap::new();
        map.insert(
            TypeId::of::<OsuMoment>(),
            Rc::new(OsuMoment::default()) as Rc<dyn Any>,
        );
        map.insert(
            TypeId::of::<EmptyStruct>(),
            Rc::new(EmptyStruct::default()) as Rc<dyn Any>,
        );
        map.insert(TypeId::of::<Enum>(), Rc::new(Enum::Default) as Rc<dyn Any>);
    });
}
#[bench]
fn hash_map_hashnt(b: &mut Bencher) {
    b.iter(|| {
        let mut map = HashMap::with_hasher(Hashnt);
        map.insert(
            TypeId::of::<OsuMoment>(),
            Rc::new(OsuMoment::default()) as Rc<dyn Any>,
        );
        map.insert(
            TypeId::of::<EmptyStruct>(),
            Rc::new(EmptyStruct::default()) as Rc<dyn Any>,
        );
        map.insert(TypeId::of::<Enum>(), Rc::new(Enum::Default) as Rc<dyn Any>);
    });
}

// Forced Reallocs

#[bench]
fn typemap_forced_realloc(b: &mut Bencher) {
    b.iter(|| {
        let mut map = TypeMap::new(2, 100);
        map.insert(OsuMoment::default());
        map.insert(EmptyStruct::default());
        map.insert(Enum::Default);
    });
}
#[bench]
fn hash_map_forced_realloc(b: &mut Bencher) {
    b.iter(|| {
        let mut map = HashMap::with_capacity(2);
        map.insert(
            TypeId::of::<OsuMoment>(),
            Rc::new(OsuMoment::default()) as Rc<dyn Any>,
        );
        map.insert(
            TypeId::of::<EmptyStruct>(),
            Rc::new(EmptyStruct::default()) as Rc<dyn Any>,
        );
        map.insert(TypeId::of::<Enum>(), Rc::new(Enum::Default) as Rc<dyn Any>);
    });
}
#[bench]
fn hash_map_hashnt_forced_realloc(b: &mut Bencher) {
    b.iter(|| {
        let mut map = HashMap::with_capacity_and_hasher(2, Hashnt);
        map.insert(
            TypeId::of::<OsuMoment>(),
            Rc::new(OsuMoment::default()) as Rc<dyn Any>,
        );
        map.insert(
            TypeId::of::<EmptyStruct>(),
            Rc::new(EmptyStruct::default()) as Rc<dyn Any>,
        );
        map.insert(TypeId::of::<Enum>(), Rc::new(Enum::Default) as Rc<dyn Any>);
    });
}

// Reads & Writes

#[bench]
fn typemap_rw(b: &mut Bencher) {
    b.iter(|| {
        let mut map = TypeMap::new(3, 100);
        map.insert(OsuMoment::default());
        map.insert(EmptyStruct::default());
        map.insert(Enum::Default);

        let _osu_moment: &OsuMoment = black_box(map.get().unwrap());
        let _empty_struct: &EmptyStruct = black_box(map.get().unwrap());
        let _enum_: &Enum = black_box(map.get().unwrap());
    });
}
#[bench]
fn hash_map_rw(b: &mut Bencher) {
    b.iter(|| {
        let mut map = HashMap::new();
        map.insert(
            TypeId::of::<OsuMoment>(),
            Rc::new(OsuMoment::default()) as Rc<dyn Any>,
        );
        map.insert(
            TypeId::of::<EmptyStruct>(),
            Rc::new(EmptyStruct::default()) as Rc<dyn Any>,
        );
        map.insert(TypeId::of::<Enum>(), Rc::new(Enum::Default) as Rc<dyn Any>);

        let _osu_moment: Rc<OsuMoment> = black_box(
            map.get(&TypeId::of::<OsuMoment>())
                .cloned()
                .unwrap()
                .downcast()
                .unwrap(),
        );
        let _empty_struct: Rc<EmptyStruct> = black_box(
            map.get(&TypeId::of::<EmptyStruct>())
                .cloned()
                .unwrap()
                .downcast()
                .unwrap(),
        );
        let _enum_: Rc<Enum> = black_box(
            map.get(&TypeId::of::<Enum>())
                .cloned()
                .unwrap()
                .downcast()
                .unwrap(),
        );
    });
}
#[bench]
fn hash_map_hashnt_rw(b: &mut Bencher) {
    b.iter(|| {
        let mut map = HashMap::with_hasher(Hashnt);
        map.insert(
            TypeId::of::<OsuMoment>(),
            Rc::new(OsuMoment::default()) as Rc<dyn Any>,
        );
        map.insert(
            TypeId::of::<EmptyStruct>(),
            Rc::new(EmptyStruct::default()) as Rc<dyn Any>,
        );
        map.insert(TypeId::of::<Enum>(), Rc::new(Enum::Default) as Rc<dyn Any>);

        let _osu_moment: Rc<OsuMoment> = black_box(
            map.get(&TypeId::of::<OsuMoment>())
                .cloned()
                .unwrap()
                .downcast()
                .unwrap(),
        );
        let _empty_struct: Rc<EmptyStruct> = black_box(
            map.get(&TypeId::of::<EmptyStruct>())
                .cloned()
                .unwrap()
                .downcast()
                .unwrap(),
        );
        let _enum_: Rc<Enum> = black_box(
            map.get(&TypeId::of::<Enum>())
                .cloned()
                .unwrap()
                .downcast()
                .unwrap(),
        );
    });
}

// Reads

#[bench]
fn typemap_read(b: &mut Bencher) {
    let mut map = TypeMap::new(3, 100);
    map.insert(OsuMoment::default());
    map.insert(EmptyStruct::default());
    map.insert(Enum::Default);

    b.iter(|| {
        let _osu_moment: &OsuMoment = black_box(map.get().unwrap());
        let _empty_struct: &EmptyStruct = black_box(map.get().unwrap());
        let _enum_: &Enum = black_box(map.get().unwrap());
    });
}
#[bench]
fn hash_map_read(b: &mut Bencher) {
    let mut map = HashMap::new();
    map.insert(
        TypeId::of::<OsuMoment>(),
        Rc::new(OsuMoment::default()) as Rc<dyn Any>,
    );
    map.insert(
        TypeId::of::<EmptyStruct>(),
        Rc::new(EmptyStruct::default()) as Rc<dyn Any>,
    );
    map.insert(TypeId::of::<Enum>(), Rc::new(Enum::Default) as Rc<dyn Any>);

    b.iter(|| {
        let _osu_moment: Rc<OsuMoment> = black_box(
            map.get(&TypeId::of::<OsuMoment>())
                .cloned()
                .unwrap()
                .downcast()
                .unwrap(),
        );
        let _empty_struct: Rc<EmptyStruct> = black_box(
            map.get(&TypeId::of::<EmptyStruct>())
                .cloned()
                .unwrap()
                .downcast()
                .unwrap(),
        );
        let _enum_: Rc<Enum> = black_box(
            map.get(&TypeId::of::<Enum>())
                .cloned()
                .unwrap()
                .downcast()
                .unwrap(),
        );
    });
}
#[bench]
fn hash_map_hashnt_read(b: &mut Bencher) {
    let mut map = HashMap::with_hasher(Hashnt);
    map.insert(
        TypeId::of::<OsuMoment>(),
        Rc::new(OsuMoment::default()) as Rc<dyn Any>,
    );
    map.insert(
        TypeId::of::<EmptyStruct>(),
        Rc::new(EmptyStruct::default()) as Rc<dyn Any>,
    );
    map.insert(TypeId::of::<Enum>(), Rc::new(Enum::Default) as Rc<dyn Any>);

    b.iter(|| {
        let _osu_moment: Rc<OsuMoment> = black_box(
            map.get(&TypeId::of::<OsuMoment>())
                .cloned()
                .unwrap()
                .downcast()
                .unwrap(),
        );
        let _empty_struct: Rc<EmptyStruct> = black_box(
            map.get(&TypeId::of::<EmptyStruct>())
                .cloned()
                .unwrap()
                .downcast()
                .unwrap(),
        );
        let _enum_: Rc<Enum> = black_box(
            map.get(&TypeId::of::<Enum>())
                .cloned()
                .unwrap()
                .downcast()
                .unwrap(),
        );
    });
}

// misc

#[bench]
fn typemap_osu_moment(b: &mut Bencher) {
    let mut map = TypeMap::new(1, 10);
    map.insert(OsuMoment::default());
    b.iter(|| {
        let mapped: &OsuMoment = map.get().unwrap();
        println!("val: {}", mapped.num);
    })
}

#[bench]
fn box_osu_moment(b: &mut Bencher) {
    let unmapped = Box::new(OsuMoment::default());
    b.iter(|| {
        println!("val: {}", unmapped.num);
    })
}
