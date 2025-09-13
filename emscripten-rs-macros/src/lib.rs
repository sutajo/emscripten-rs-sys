use proc_macro::TokenStream;
use quote::quote;
use syn::{punctuated::Punctuated, *};

fn postprocess_script(script: String) -> (String, usize) {
    let original_len = script.len();

    let trimmed = script
        .lines()
        .map(str::trim)
        .collect::<String>();

    let escaped =
        trimmed
        .chars()
        .map(|c| match c {
            '"' => r#"\""#.into(),
            '{' => "{{".into(),
            '}' => "}}".into(),
            _ => c.to_string()
        })
        .collect::<String>();


    (escaped, original_len)
}

#[proc_macro]
pub fn len_in_bytes(input: TokenStream) -> TokenStream {
    let lit = parse_macro_input!(input as LitStr);
    let len = postprocess_script(lit.value()).1;

    // Produce a compile-time usize literal
    let expanded = quote! { #len };
    TokenStream::from(expanded)
}

#[proc_macro]
pub fn get_processed_script(input: TokenStream) -> TokenStream {
    let lit = parse_macro_input!(input as LitStr);
    let final_script = postprocess_script(lit.value()).0;

    // Produce a compile-time string literal
    let expanded = quote! { #final_script };
    TokenStream::from(expanded)
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
