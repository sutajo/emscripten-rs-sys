use proc_macro::TokenStream;
use quote::{ToTokens, quote};
use syn::*;

fn trim_script(script: String) -> String {
    script
        .lines()
        .map(|s| {
            let mut trimmed = s.trim().to_string();
            trimmed.push(' ');
            trimmed
        })
        .collect::<String>()
}

#[proc_macro]
pub fn len_in_bytes(input: TokenStream) -> TokenStream {
    let params = parse_macro_input!(input as ExprTuple);

    let args = params.elems[0].to_token_stream().to_string();
    let stringify = parse2::<ExprMacro>(params.elems[1].to_token_stream()).unwrap();
    let script = trim_script(stringify.mac.tokens.to_string());

    let mut len = script.len() + 1;

    // Additional bytes from decoration
    len += 4;

    // Arguments length
    len += args.len();

    // Produce a compile-time usize literal
    let expanded = quote! { #len };
    TokenStream::from(expanded)
}

#[proc_macro]
pub fn get_decorated_script(input: TokenStream) -> TokenStream {
    let params = parse_macro_input!(input as ExprTuple);

    let args = params.elems[0].to_token_stream().to_string();
    let stringify = parse2::<ExprMacro>(params.elems[1].to_token_stream()).unwrap();
    let script = trim_script(stringify.mac.tokens.to_string());

    let decorated_script = format!("{args}<::>{script}");
    let bytes = decorated_script.as_bytes();

    // Turn each byte into a literal token
    let tokens = bytes.iter().map(|b| quote! { #b });

    let output = quote! {
        [ #( #tokens ),*, b'\0' ]
    };

    output.into()
}

/// Generates a unique macro for each call site.
/// This is an ugly workaround for the problem that normal
/// macros don't accept the result of proc macros.
#[proc_macro]
pub fn inline_js(input: TokenStream) -> TokenStream {
    let tokens: proc_macro2::TokenStream = input.into();

    let call_site = proc_macro2::Span::call_site();
    let loc = call_site.unwrap();
    let mut file = loc.file();
    file.retain(|c| c.is_ascii_alphanumeric());
    let name = syn::Ident::new(
        &format!("_em_asm_{}_{}_{}", file, loc.line(), loc.column()),
        call_site,
    );

    let expanded = quote! {
        {
            macro_rules! inline_js_impl {
                (
                    ( $( $arg_name:ident : $arg_ty:ty ),*) $(-> $ret:ty)?, $($body:tt)*
                ) => {
                    {
                        unsafe {
                            $crate::export_script_to_linker!(asm, #name, $($arg_name)*, $($body)*);
                        }
                
                        #[link(wasm_import_module = "env")]
                        #[allow(dead_code)]
                        unsafe extern "C" {
                            pub unsafe fn #name($( $arg_name : $arg_ty ),*) $(-> $ret)?;
                        }
    
                        unsafe { #name($($arg_name),*) }
                    }
                };
    
                (
                    $($body:tt)*
                ) => {
                    {
                        unsafe {
                            $crate::export_script_to_linker!(asm, #name,  , $($body)*);
                        }
                
                        #[link(wasm_import_module = "env")]
                        #[allow(dead_code)]
                        unsafe extern "C" {
                            pub unsafe fn #name();
                        }
    
                        unsafe { #name() }
                    }
                };
            }

            inline_js_impl!(#tokens)
        }
    };
    expanded.into()
}