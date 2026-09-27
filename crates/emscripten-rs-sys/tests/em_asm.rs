#![feature(min_generic_const_args, generic_const_args)]
#![allow(incomplete_features)]

use emscripten_rs_sys::em_asm::*;

#[test]
fn js_asm() {
    let a = 2;
    let b = 3;
    let result = js_asm! { |a,b| -> i32 { return a+b*b; } };
    assert_eq!(result, 2 + 3 * 3);
}
