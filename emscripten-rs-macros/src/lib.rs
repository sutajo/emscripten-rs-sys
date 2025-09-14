use proc_macro::TokenStream;
use quote::{ToTokens, quote};
use syn::{punctuated::Punctuated, *};

#[proc_macro]
pub fn len_in_bytes(input: TokenStream) -> TokenStream {
    let params = parse_macro_input!(input as ExprTuple);

    let args = params.elems[0].to_token_stream().to_string();
    let script = parse2::<LitStr>(params.elems[1].to_token_stream()).unwrap();

    let mut len = script.value().as_bytes().len() + 1;

    // Additional bytes from decoration
    len += 6;

    // Arguments length
    len += args.len();

    // Produce a compile-time usize literal
    let expanded = quote! { #len };
    TokenStream::from(expanded)
}

#[proc_macro]
pub fn get_processed_script(input: TokenStream) -> TokenStream {
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

#[proc_macro]
pub fn len_params(input: TokenStream) -> TokenStream {
    let idents = parse_macro_input!(input with Punctuated::<Ident, Token![,]>::parse_terminated);

    // Calculate total length
    let mut length = 0;
    for ident in idents.iter() {
        length += ident.to_string().len();
    }
    // Add commas: number of commas = number of identifiers - 1 (if at least one ident exists)
    if idents.len() > 1 {
        length += idents.len() - 1;
    }

    // Return a literal
    let output = quote! {
        #length
    };
    output.into()
}
