use emscripten_rs_sys::em_asm::SignatureBuilder;
use std::ffi::c_char;

#[test]
fn empty_and_mixed_signatures_are_const() {
    const EMPTY: [c_char; 1] = SignatureBuilder::<1>::new::<i32>().finish();
    const MIXED: [c_char; 5] = SignatureBuilder::<5>::new::<f64>()
        .add_param(&1i32)
        .add_param(&2f64)
        .add_param(&3i64)
        .add_param(&std::ptr::null::<u8>())
        .finish();
    assert_eq!(EMPTY, [0]);
    assert_eq!(MIXED, [b'i' as _, b'd' as _, b'j' as _, b'p' as _, 0]);
}

#[test]
#[should_panic(expected = "signature buffer is full")]
fn capacity_reserves_the_null_terminator() {
    SignatureBuilder::<1>::new::<i32>().add_param(&1);
}
