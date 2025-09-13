#![feature(used_with_arg)]
#![feature(asm_experimental_arch)]

use emscripten_rs_macros::{inline_js, js};

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
