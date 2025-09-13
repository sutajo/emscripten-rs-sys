use proc_macro::TokenStream;
use syn::parse2;

use crate::em_js::{InlineJsInput, JsInputs, inline_js_impl, js_impl};

mod em_js;

#[proc_macro]
pub fn js(input: TokenStream) -> TokenStream {
    let tokens: proc_macro2::TokenStream = input.into();

    parse2::<JsInputs>(tokens)
        .and_then(js_impl)
        .unwrap_or_else(|err| err.into_compile_error())
        .into()
}

#[proc_macro]
pub fn inline_js(input: TokenStream) -> TokenStream {
    let tokens: proc_macro2::TokenStream = input.into();

    parse2::<InlineJsInput>(tokens)
        .and_then(inline_js_impl)
        .unwrap_or_else(|err| err.into_compile_error())
        .into()
}

#[cfg(test)]
mod tests {
    use crate::JsInputs;
    use quote::quote;
    use syn::parse2;

    #[test]
    fn simple() {
        let input = quote! {fn f(){return 1;}
        };
        assert_eq!(
            crate::js_impl(parse2::<JsInputs>(input).unwrap())
                .unwrap()
                .to_string(),
            quote! {
                mod _em_js_exports___em_js__f {
                    #[used(linker)]
                    #[unsafe(no_mangle)]
                    #[allow(non_upper_case_globals)]
                    static __em_js__f: [u8; 20usize] =
                        *b"()<::>{return 1 ; }\0";
                    std::arch::global_asm!(".globl __em_js__f");
                }
                #[link(wasm_import_module = "env")]
                #[allow(dead_code)]
                unsafe extern "C" {
                    pub unsafe fn f();
                }
            }
            .to_string()
        )
    }
}
