// This should have been much less painful to implement.
// The problem is that using the declarations below, rustc should emit
// external global definitions in the LLVM IR, but instead it emits internal ones if the
// static variables are defined and used within the same compilation unit, which prevents
// Emscripten from picking up the symbols at link time.
// For this reason, I had to resort to inline assembly to properly export the
// symbols for Emscripten.
//
// Inline assembly for WASM is only supported on nightly at the time of writing.
//
// #[used]
// #[unsafe(no_mangle)]
// pub static mut __em_js_ref_FUN: usize = 0;
//
// #[used]
// #[unsafe(no_mangle)]
// pub static mut __em_js__FUN: [u8; _] = *b"(args)<::>{ <script> }\0";

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
        mod ${concat($name, symbols)} {
            use std::{arch::global_asm};
            use emscripten_rs_macros::*;

            macro_rules! declare_global {
                // Pattern: function signature, name, string
                ($exported_symbol:expr, $code:expr, $size_params:expr, $size_body:expr) => {
                    global_asm!(
                        concat!(".type	", $exported_symbol, ",@object"),
                        concat!(".globl ", $exported_symbol),
                        concat!(".section .data.", $exported_symbol, ",\"R\",@"),
                        concat!(".p2align 1, 0x0"),
                        concat!($exported_symbol, ":"),
                        concat!(r#".asciz ""#, $code, r#"""#),
                        // Need to calculate the final size
                        // ( Len of params ) + <::> + {} + \n + len of body + terminating zero
                        concat!(".size ", $exported_symbol, ", ", "2+", $size_params, "+4+2+2+", $size_body, "+1"),
                        concat!(".no_dead_strip ", $exported_symbol)
                    );
                };
            }

            declare_global!(stringify!(${concat(__em_js_ref_, $name)}), "", 0, 1);
            declare_global!(
                stringify!(${concat(__em_js__, $name)}),
                concat!("(", 
                    stringify!($($arg_name),*) ,r#")<::>\n{{"#,
                    get_processed_script!($body), r#"}}"#
                ),
                len_params!($($arg_name),*),
                len_in_bytes!($body)
            );
        }

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
        assert_eq!(unsafe { first_js(5) }, 20)
    }
}
