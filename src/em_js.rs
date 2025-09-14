
#[macro_export]
macro_rules! declare_global_js_fn {
    // Pattern: function signature, name, string
    ($exported_symbol:ident, $code:expr, $size_code:expr) => {
        #[used]
        #[unsafe(no_mangle)]
        #[allow(non_upper_case_globals)]
        pub static $exported_symbol : [u8; $size_code] = $code;
        
        // Inline assembly for WASM is only supported on nightly at the time of writing.
        // Make sure to export the symbol for Emscripten
        std::arch::global_asm!(concat!(".globl ", stringify!($exported_symbol)));
    };
}

/// Define JS functions in Rust.
///
/// This macro provides a faster alternative to the `emscripten_run_script_*` family of functions.
///
/// See the documentation for more: <https://emscripten.org/docs/porting/connecting_cpp_and_javascript/Interacting-with-code.html#interacting-with-code-call-javascript-from-native>
#[macro_export]
macro_rules! em_js {
    (
        fn $name:ident ( $( $arg_name:ident : $arg_ty:ty ),* ) -> $ret:ty, $body:expr
    ) => {
        declare_global_js_fn!(${concat(__em_js_ref_, $name)}, *b"\0", 1);
        declare_global_js_fn!(
            ${concat(__em_js__, $name)},
            emscripten_rs_macros::get_decorated_script!(([$($arg_name),*], $body)),
            emscripten_rs_macros::len_in_bytes!((($($arg_name),*), $body))
        );

        #[link(wasm_import_module = "env")]
        #[allow(dead_code)]
        unsafe extern "C" {
            pub unsafe fn $name($( $arg_name : $arg_ty ),*) -> $ret;
        }
    };
}

#[cfg(test)]
mod tests {
    use std::ffi::{CStr, c_char, c_int};

    em_js!(fn get_string_from_js() -> *mut c_char, r#"
        var jsString = 'hello from js';
        var lengthBytes = jsString.length+1;
        var stringOnWasmHeap = _malloc(lengthBytes);
        stringToUTF8(jsString, stringOnWasmHeap, lengthBytes);
        return stringOnWasmHeap;
    "#);

    #[test]
    fn test_string_result() {
        unsafe {
            let c_str = get_string_from_js();
            assert_eq!(CStr::from_ptr(c_str), c"hello from js");
            libc::free(c_str as _);
        }
    }

    em_js!(fn string_param(url: *const c_char) -> (), r#"
        url = UTF8ToString(Number(url));
        out(`Query url is: ${url}`);
    "#);

    #[test]
    fn test_string_param() {
        unsafe {
            string_param(c"https://reqbin.com/echo/get/json".as_ptr());
        }
    }

    em_js!(fn sum(n: c_int) -> c_int, r#"
        let sum = 0;
        for(let i=1; i<n; i++)
        {
            sum += i;
        }
        return sum;
    "#);

    #[test]
    fn test_sum() {
        assert_eq!(unsafe { sum(100) }, 4950);
    }

    use std::simd::i32x4;
    use std::simd::num::SimdInt;

    #[unsafe(no_mangle)]
    #[target_feature(enable = "simd128")]
    pub extern "C" fn hadd_rs(v1: i32, v2: i32, v3: i32, v4: i32) -> i32
    {
        i32x4::from_array([v1,v2,v3,v4]).reduce_sum()
    }

    em_js!(fn second_js(param: i32) -> i32, r#"
        return _hadd_rs(param, param, param, param);
    "#);

    em_js!(fn first_js(param: i32) -> i32, r#"
        return second_js(param)
    "#);

    #[test]
    fn test_transitiveness() {
        assert_eq!(unsafe { first_js(5) }, 20);
    }
}
