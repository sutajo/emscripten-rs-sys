use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use proc_macro2::Literal;
use proc_macro2::TokenStream;
use quote::ToTokens;
use quote::format_ident;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    *,
};

fn arg_names(args: &Punctuated<Arg, Token![,]>) -> Punctuated<Ident, Token![,]> {
    args.iter().map(|arg| arg.name.clone()).collect()
}

pub(crate) fn trim_script(script: String) -> String {
    script
        .lines()
        .map(|s| {
            let mut trimmed = s.trim().to_string();
            trimmed.push(' ');
            trimmed
        })
        .collect::<String>()
}

fn get_decorated_script(args: &Punctuated<Arg, Token![,]>, body: &TokenStream) -> String {
    let arg_names = arg_names(args);
    format!(
        "({})<::>{{{}}}",
        quote! { #arg_names },
        trim_script(body.to_string())
    )
}

fn export_to_linker(global: bool, item_name: Ident, mut contents: String) -> TokenStream {
    // For Emscripten to see these symbols, they need to be true global variables.
    // Despite using pub static, in the LLVM IR the variable is defined with internal const.
    // Thankfully this can be overriden with inline ASM.
    // Inline assembly for WASM is only supported on nightly at the time of writing.
    let asm_ident = format_ident!("{}", if global { "global_asm" } else { "asm" });

    let asm = format!(".globl {item_name}");

    contents.push('\0');
    let length = contents.len();
    let bytes = Literal::byte_string(contents.as_bytes());
    let module = format_ident!("_em_js_exports_{item_name}");
    quote! {
        mod #module {
            #[used(linker)]
            #[unsafe(no_mangle)]
            #[allow(non_upper_case_globals)]
            static #item_name: [u8; #length] = *#bytes;

            std::arch::#asm_ident!(#asm);
        }
    }
}

struct JsInput {
    name: Ident,
    args: Punctuated<Arg, Token![,]>,
    ret: Option<Type>,
    body: proc_macro2::TokenStream,
    is_async: bool,
}

impl ToTokens for JsInput {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = &self.name;
        let args = &self.args;
        let ret_type = if let Some(ty) = &self.ret {
            quote! {
                -> #ty
            }
        } else {
            quote! {}
        };
        let body = if self.is_async {
            let inner = &self.body;
            quote! {
                { return Asyncify.handleAsync(async () => { #inner }); }
            }
        } else {
            self.body.clone()
        };
        let script = get_decorated_script(&self.args, &body);
        let link_name = if self.is_async {
            format!("__asyncjs__{}", self.name)
        } else {
            self.name.to_string()
        };
        let js_name = format_ident!("__em_js__{}", link_name);
        tokens.extend([
            export_to_linker(true, js_name, script),
            quote! {
                #[link(wasm_import_module = "env")]
                #[allow(dead_code)]
                unsafe extern "C" {
                    #[link_name = #link_name]
                    pub unsafe fn #name(#args) #ret_type;
                }
            },
        ]);
    }
}

struct Arg {
    name: Ident,
    ty: Type,
}

impl ToTokens for Arg {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = &self.name;
        let ty = &self.ty;
        tokens.extend(quote! {
            #name: #ty
        });
    }
}

impl Parse for Arg {
    fn parse(input: ParseStream) -> Result<Self> {
        let name = input.parse()?;
        input.parse::<Token![:]>()?;
        Ok(Arg {
            name,
            ty: input.parse()?,
        })
    }
}

impl Parse for JsInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut is_async = false;
        if input.parse::<Token![async]>().is_ok() {
            is_async = true;
        }

        input.parse::<Token![fn]>()?;

        let name = input.parse()?;

        let content;
        syn::parenthesized!(content in input);

        let args = content.parse_terminated(Arg::parse, Token![,])?;

        let ret = if input.peek(Token![->]) {
            input.parse::<Token![->]>()?;
            Some(input.parse()?)
        } else {
            None
        };

        let content;
        braced!(content in input);

        Ok(JsInput {
            name,
            args,
            ret,
            is_async,
            body: content.parse()?,
        })
    }
}

pub(crate) struct JsInputs {
    items: Vec<JsInput>,
}

impl ToTokens for JsInputs {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        for item in &self.items {
            item.to_tokens(tokens);
        }
    }
}

impl Parse for JsInputs {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut items = Vec::new();

        while !input.is_empty() {
            items.push(input.parse::<JsInput>()?);
        }

        Ok(JsInputs { items })
    }
}

pub(crate) struct InlineJsInput {
    args: Punctuated<Arg, Token![,]>,
    ret: Option<Type>,
    body: proc_macro2::TokenStream,
}

impl Parse for InlineJsInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let args: Punctuated<Arg, Token![,]> = if input.peek(syn::token::Paren) {
            let content;
            syn::parenthesized!(content in input);

            content.parse_terminated(Arg::parse, Token![,])?
        } else {
            Punctuated::new()
        };

        let ret = if input.peek(Token![->]) {
            input.parse::<Token![->]>()?;
            Some(input.parse()?)
        } else {
            None
        };

        Ok(InlineJsInput {
            args,
            ret,
            body: input.parse()?,
        })
    }
}

impl ToTokens for InlineJsInput {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);

        let args = &self.args;
        let arg_names = arg_names(args);
        let name = format!("inline_{}", COUNTER.fetch_add(1, Ordering::SeqCst));
        let name_ident = format_ident!("{name}");
        let ret_type = if let Some(ty) = &self.ret {
            quote! {
                -> #ty
            }
        } else {
            quote! {}
        };
        let export = export_to_linker(
            true,
            format_ident!("__em_js__{}", name),
            format!("{}", get_decorated_script(&self.args, &self.body)),
        );
        tokens.extend(quote! {
            {
                #export

                #[link(wasm_import_module = "env")]
                #[allow(dead_code)]
                unsafe extern "C" {
                    unsafe fn #name_ident(#args) #ret_type;
                }

                unsafe { #name_ident(#arg_names) }
            }
        });
    }
}