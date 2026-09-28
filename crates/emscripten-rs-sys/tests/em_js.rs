#![cfg_attr(feature = "force_export", feature(asm_experimental_arch))]

use emscripten_rs_sys::em_js::*;

js! {
    fn test(x: i32) -> i32 {
        return x+1;
    }
}

#[test]
fn em_js() {
    assert_eq!(
        inline_js! {
            () -> i32
            return 342;
        },
        342,
    );

    assert_eq!(unsafe { test(1) }, 2);
}

js! {
    fn stable_js_leaf(value: i32) -> i32 { return value * 2; }
    fn stable_js_parent(value: i32) -> i32 { return stable_js_leaf(value) + 1; }
    async fn stable_js_async(value: i32) -> i32 {
        return await Promise.resolve(value + 1);
    }
}

#[test]
fn transitive_and_async_js() {
    assert_eq!(unsafe { stable_js_parent(20) }, 41);
    assert_eq!(unsafe { stable_js_async(41) }, 42);
}

#[test]
fn typed_inline_js() {
    let text = c"hello".as_ptr();
    let value = 1.5f64;
    assert_eq!(
        inline_js! {
            (text: *const std::ffi::c_char, value: f64) -> f64
            return UTF8ToString(text).length + value;
        },
        6.5
    );
}
