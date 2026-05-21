use proc_macro::TokenStream;
use quote::ToTokens;
use syn::parse2;

use crate::{
    em_asm::AsmInput,
    em_js::{InlineJsInput, JsInputs},
};

mod em_asm;
mod em_js;

#[proc_macro]
pub fn js(input: TokenStream) -> TokenStream {
    let tokens: proc_macro2::TokenStream = input.into();

    parse2::<JsInputs>(tokens)
        .map(ToTokens::into_token_stream)
        .unwrap_or_else(|err| err.into_compile_error())
        .into()
}

#[proc_macro]
pub fn inline_js(input: TokenStream) -> TokenStream {
    let tokens: proc_macro2::TokenStream = input.into();

    parse2::<InlineJsInput>(tokens)
        .map(ToTokens::into_token_stream)
        .unwrap_or_else(|err| err.into_compile_error())
        .into()
}

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
    use quote::{ToTokens, quote};
    use syn::parse2;

    #[test]
    fn simple() {
        let input = quote! {fn f(){return 1;}
        };
        assert_eq!(
            parse2::<JsInputs>(input)
                .unwrap()
                .into_token_stream()
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
                    #[link_name = "f"]
                    pub unsafe fn f();
                }
            }
            .to_string()
        )
    }
}
