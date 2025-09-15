use proc_macro::TokenStream;
use quote::{ToTokens, quote};
use syn::*;

fn trim_script(script: String) -> String {
    script.lines().map(|s| {
        let mut trimmed = s.trim().to_string();
        trimmed.push(' ');
        trimmed
    }).collect::<String>()
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
