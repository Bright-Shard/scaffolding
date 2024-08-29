use scaffolding::plugin_prelude::*;

#[test]
fn works() {
    let uniq = Uniq::default();

    for current in 0..5 {
        let stored: &mut u32 = uniq.get_or_default(uniq_key!());

        assert_eq!(*stored, current);

        *stored += 1;
    }
}
