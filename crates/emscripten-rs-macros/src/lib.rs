#![doc = include_str!("../README.md")]

use proc_macro::TokenStream;
use quote::ToTokens;
use syn::parse2;

use crate::{
    em_asm::AsmInput,
    em_js::{InlineJsInput, JsInputs},
};

mod em_asm;
mod em_js;

/// Declares JavaScript functions callable from Rust.
///
/// Requires `-C link-dead-code`, or the `force_export` feature and the caller's
/// `#![feature(asm_experimental_arch)]` attribute on nightly Rust.
#[proc_macro]
pub fn js(input: TokenStream) -> TokenStream {
    let tokens: proc_macro2::TokenStream = input.into();

    parse2::<JsInputs>(tokens)
        .map(ToTokens::into_token_stream)
        .unwrap_or_else(|err| err.into_compile_error())
        .into()
}

/// Executes JavaScript with explicitly typed arguments and an optional return type.
///
/// Uses the same export configuration as [`js!`].
#[proc_macro]
pub fn inline_js(input: TokenStream) -> TokenStream {
    let tokens: proc_macro2::TokenStream = input.into();

    parse2::<InlineJsInput>(tokens)
        .map(ToTokens::into_token_stream)
        .unwrap_or_else(|err| err.into_compile_error())
        .into()
}

/// Executes JavaScript with inferred Rust argument types through Emscripten's EM_ASM API.
///
/// Import this macro and its helpers through `emscripten_rs_sys::em_asm::*`.
/// The compiler must include the Wasm `link_section` data-segment fix.
#[proc_macro]
pub fn js_asm(input: TokenStream) -> TokenStream {
    let tokens: proc_macro2::TokenStream = input.into();

    parse2::<AsmInput>(tokens)
        .map(ToTokens::into_token_stream)
        .unwrap_or_else(|err| err.into_compile_error())
        .into()
}

#[cfg(test)]
mod tests {
    use crate::JsInputs;
    use quote::quote;
    use syn::parse2;

    #[test]
    fn parses_multiple_js_functions() {
        assert!(
            parse2::<JsInputs>(quote! {
                fn f() { return 1; }
                async fn g(x: i32) -> i32 { return await Promise.resolve(x); }
            })
            .is_ok()
        );
    }
}
