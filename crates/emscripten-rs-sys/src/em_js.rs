/// Executes a Javascript snippet inside a Rust function.
pub use emscripten_rs_macros::{inline_js, js};

#[cfg(test)]
mod tests {
    use std::ffi::{CStr, c_char, c_int};

    use super::*;
    use crate::emscripten_builtin_free;

    js! {
        fn get_string_from_js() -> *mut c_char
        {
            var jsString = "hello from js";
            var lengthBytes = jsString.length+1;
            var stringOnWasmHeap = _malloc(lengthBytes);
            stringToUTF8(jsString, stringOnWasmHeap, lengthBytes);
            return stringOnWasmHeap;
        }
    }

    #[test]
    fn test_string_result() {
        unsafe {
            let c_str = get_string_from_js();
            assert_eq!(CStr::from_ptr(c_str), c"hello from js");
            emscripten_builtin_free(c_str as _);
        }
    }

    js! {
        fn string_param(url: *const c_char)
        {
            if (UTF8ToString(url) != "test")
            {
                throw("strings are not equal")
            }
        }
    }

    #[test]
    fn test_string_param() {
        unsafe {
            string_param(c"test".as_ptr());
        }
    }

    js! {
        fn sum(n: c_int) -> c_int
        {
            let sum = 0;
            for(let i=1; i<n; i++)
            {
                sum += i;
            }
            return sum;
        }
    }

    #[test]
    fn test_sum() {
        assert_eq!(unsafe { sum(100) }, 4950);
    }

    use std::simd::i32x4;
    use std::simd::num::SimdInt;

    #[unsafe(no_mangle)]
    #[target_feature(enable = "simd128")]
    pub extern "C" fn hadd_rs(v1: i32, v2: i32, v3: i32, v4: i32) -> i32 {
        i32x4::from_array([v1, v2, v3, v4]).reduce_sum()
    }

    js! {
        fn second_js(param: i32) -> i32
        {
            return _hadd_rs(param, param, param, param);
        }

        fn first_js(param: i32) -> i32
        {
            return second_js(param);
        }
    }

    #[test]
    fn test_transitiveness() {
        assert_eq!(unsafe { first_js(5) }, 20);
    }

    js! {
        fn multiple_params(a: i32, b: i32, c: i32) -> i32
        {
            return a+b*c;
        }
    }

    #[test]
    fn test_multiple_params() {
        assert_eq!(unsafe { multiple_params(3, 4, 5) }, 23);
    }

    #[test]
    fn test_inline_js() {
        assert!(inline_js! {
            () -> bool
            return eval("2 + 2") === eval("4");
        });

        let result = inline_js! {
            () -> i32
            return 432;
        };
        assert_eq!(result, 432);

        let x = 6.342342131f32;
        let cos_x = inline_js! {
            (x: f32) -> f32
            return Math.cos(x);
        };
        assert!((cos_x - x.cos()).abs() < 0.00001);

        inline_js! {
            const os = require("os");

            // Basic system information
            console.log("OS Platform: " + os.platform());
            console.log("OS Type: " + os.type());
            console.log("OS Release: " + os.release());
            console.log("CPU Architecture: " + os.arch());
            console.log("Hostname: " + os.hostname());

            // Memory information
            const totalMemGB = (os.totalmem() / (1024 * 1024 * 1024)).toFixed(2);
            const freeMemGB = (os.freemem() / (1024 * 1024 * 1024)).toFixed(2);
            console.log("Memory: " + freeMemGB + " GB free of " + totalMemGB + " GB");

            // User information
            const userInfo = os.userInfo();
            console.log("Current User: " + userInfo.username);
            console.log("Home Directory: " + os.homedir);
        };
    }

    js! {
        async fn fetch_google() -> *mut c_char
        {
            const response = await fetch("https://google.com");
            const result = await response.text();
            var lengthBytes = result.length+1;
            var stringOnWasmHeap = _malloc(lengthBytes);
            stringToUTF8(result, stringOnWasmHeap, lengthBytes);
            return stringOnWasmHeap;
        }
    }

    #[test]
    fn async_js() {
        let google_html = unsafe { CStr::from_ptr(fetch_google()) }
            .to_string_lossy()
            .to_string();
        assert!(google_html.contains("<!doctype html>"));
    }
}
