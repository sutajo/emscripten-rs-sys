#[macro_export]
macro_rules! export_bytes {
    ($asm:ident, $exported_symbol:ident, $code:expr, $size_code:expr) => {
        #[used]
        #[unsafe(no_mangle)]
        #[allow(non_upper_case_globals)]
        pub static $exported_symbol: [u8; $size_code] = $code;

        // For Emscripten to see these symbols, they need to be true global variables.
        // Despite using pub static, in the LLVM IR the variable is defined with internal const.
        // Thankfully this can be overriden with inline ASM.
        // Inline assembly for WASM is only supported on nightly at the time of writing.
        std::arch::$asm!(concat!(".globl ", stringify!($exported_symbol)));
    };
}

#[macro_export]
macro_rules! export_script_to_linker {
    (
       $export:ident, $name:ident, $($arg_name:ident)*, $($body:tt)*
    ) => {
        $crate::export_bytes!($export, ${concat(__em_js_ref_, $name)}, *b"\0", 1);
        $crate::export_bytes!($export,
            ${concat(__em_js__, $name)},
            emscripten_rs_macros::get_decorated_script!((
                ($($arg_name),*),
                stringify!({ $($body)* })
            )),
            emscripten_rs_macros::len_in_bytes!((
                ($($arg_name),*),
                stringify!({ $($body)* })
            ))
        );
    };
}

/// Define JS functions in Rust.
///
/// This macro provides a faster alternative to the `emscripten_run_script_*` family of functions.
///
/// See the documentation for more: <https://emscripten.org/docs/porting/connecting_cpp_and_javascript/Interacting-with-code.html#interacting-with-code-call-javascript-from-native>
#[macro_export]
macro_rules! js {
    (
        $( fn $name:ident ( $( $arg_name:ident : $arg_ty:ty ),*) $(-> $ret:ty)? { $($body:tt)* } )*
    ) => 
    (
        $( $crate::export_script_to_linker!(global_asm, $name, $($arg_name)*, $($body)*); )*

        $(
            #[link(wasm_import_module = "env")]
            #[allow(dead_code)]
            unsafe extern "C" {
                pub unsafe fn $name($( $arg_name : $arg_ty ),*) $(-> $ret)?;
            }
        )*
    )
}

/// Executes a Javascript snippet inside a Rust function.
pub use emscripten_rs_macros::inline_js;

#[cfg(test)]
mod tests {
    use std::ffi::{CStr, c_char, c_int};

    use crate::{em_js::inline_js, emscripten_builtin_free};

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

    js! {
        fn h() -> f32
        {
            return 2.0;
        }

        fn f(x: f32) -> f32
        {
            return Math.pow(x, h());
        } 

        fn g(x: f32) -> f32
        {
            return Math.sqrt(f(x));
        }
    }

    #[test]
    fn test_call_fn_from_js_and_rust() {
        unsafe {
            assert_eq!(f(g(10.0)), f(h() * 5.0));
        }
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
            () -> bool,
            return eval("2 + 2") === eval("4");
        });

        let result = inline_js! {
            () -> i32,
            return 432;
        };
        assert_eq!(result, 432);

        let x = 6.342342131f32;
        let cos_x = inline_js! {
            (x: f32) -> f32,
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
}
