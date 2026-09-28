use emscripten_rs_sys::em_asm::*;

#[test]
fn js_asm() {
    let a = 2;
    let b = 3;
    let result = js_asm! { |a,b| -> i32 { return a+b*b; } };
    assert_eq!(result, 2 + 3 * 3);
}

#[test]
fn mixed_argument_types_and_double_return() {
    let integer = 7i32;
    let double = 0.25f64;
    let bigint = 42i64;
    let text = c"hello".as_ptr();
    let result = js_asm! { |integer, double, bigint, text| -> f64 {
        const length = UTF8ToString(text).length;
        const convertedBigint = Number(bigint);
        const fractionalPart = double;
        return integer + fractionalPart + convertedBigint + length;
    }};
    assert_eq!(result, 54.25);
}

#[test]
fn argument_type_differs_from_return_type() {
    let double = 9.5f64;
    assert_eq!(
        js_asm! { |double| -> i32 { return Math.floor(double); } },
        9
    );
}
