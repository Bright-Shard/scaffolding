use {
    scaffolding::datatypes::ArenaVec,
    std::{hint::black_box, mem::size_of},
    test::{bench, Bencher},
};

#[bench]
fn arenavec(b: &mut Bencher) {
    let mut vec = black_box(ArenaVec::with_reserved_memory(size_of::<u32>() * 100));
    b.iter(|| {
        vec.clear();
        for i in black_box(0..100u32) {
            vec.push(black_box(i));
        }
    });
}

#[bench]
fn vec(b: &mut Bencher) {
    let mut vec = black_box(Vec::default());
    b.iter(|| {
        vec.clear();
        for i in black_box(0..100u32) {
            vec.push(black_box(i));
        }
    });
}
