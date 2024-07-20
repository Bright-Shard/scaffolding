use {
    scaffolding::datatypes::StackVec,
    std::hint::black_box,
    test::{bench, Bencher},
};

const NUM_ITEMS: usize = 100;

#[bench]
fn no_stack_vec(b: &mut Bencher) {
    let mut sv = StackVec::<usize, 0>::new();

    b.iter(|| {
        for i in black_box(0..NUM_ITEMS) {
            sv.push(i);
        }
    });
}

#[bench]
fn half_stack_vec(b: &mut Bencher) {
    const SIZE: usize = const { NUM_ITEMS / 2 };
    let mut sv = StackVec::<usize, SIZE>::new();

    b.iter(|| {
        for i in black_box(0..NUM_ITEMS) {
            sv.push(i);
        }
    });
}

#[bench]
fn full_stack_vec(b: &mut Bencher) {
    let mut sv = StackVec::<usize, NUM_ITEMS>::new();

    b.iter(|| {
        for i in black_box(0..NUM_ITEMS) {
            sv.push(i);
        }
    });
}

#[bench]
fn vec(b: &mut Bencher) {
    let mut v = Vec::default();

    b.iter(|| {
        for i in black_box(0..NUM_ITEMS) {
            v.push(i);
        }
    });
}
