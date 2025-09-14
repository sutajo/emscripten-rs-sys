use proc_macro::TokenStream;
use quote::{ToTokens, quote};
use syn::*;

#[proc_macro]
pub fn len_in_bytes(input: TokenStream) -> TokenStream {
    let params = parse_macro_input!(input as ExprTuple);

    let args = params.elems[0].to_token_stream().to_string();
    let script = parse2::<LitStr>(params.elems[1].to_token_stream()).unwrap();

    let mut len = script.value().len() + 1;

    // Additional bytes from decoration
    len += 6;

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
    let script = parse2::<LitStr>(params.elems[1].to_token_stream()).unwrap().value();

    let decorated_script = format!("{args}<::>{{{script}}}");
    let bytes = decorated_script.as_bytes();

    // Turn each byte into a literal token
    let tokens = bytes.iter().map(|b| quote! { #b });

    let output = quote! {
        [ #( #tokens ),*, b'\0' ]
    };

    output.into()
}
