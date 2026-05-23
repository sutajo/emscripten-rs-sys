use std::sync::atomic::{AtomicUsize, Ordering};

use proc_macro2::{Group, TokenStream, TokenTree};
use quote::{ToTokens, format_ident, quote};
use syn::{
    Ident, Token, Type, TypePath, braced, parse::Parse, punctuated::Punctuated, spanned::Spanned,
};

use crate::em_js::trim_script;

pub(crate) struct AsmInput {
    args: Punctuated<Ident, Token![,]>,
    ret: Option<Type>,
    body: proc_macro2::TokenStream,
}

impl Parse for AsmInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // ---- parse |x,y| ----
        let _pipe: Token![|] = input.parse()?;

        let mut args = Punctuated::<Ident, Token![,]>::new();

        while !input.peek(Token![|]) {
            let ident: Ident = input.parse()?;
            args.push_value(ident);

            if input.peek(Token![,]) {
                let comma: Token![,] = input.parse()?;
                args.push_punct(comma);
            }
        }

        let _pipe2: Token![|] = input.parse()?;

        // ---- parse -> ReturnType ----

        let ret: Option<Type> = if input.peek(Token![->]) {
            let _arrow: Token![->] = input.parse()?;
            Some(input.parse()?)
        } else {
            None
        };

        // ---- parse { body } ----
        let content;
        let _brace = braced!(content in input);
        let body: TokenStream = content.parse()?;

        Ok(AsmInput { args, ret, body })
    }
}

fn substitute_ident(
    replaced: &mut bool,
    input: TokenStream,
    target: &String,
    i: usize,
) -> TokenStream {
    let mut out = TokenStream::new();

    for tt in input {
        match tt {
            TokenTree::Ident(ident) if ident.to_string() == *target => {
                let replacement = format_ident!("__EM_ASM_PARAM__{i}");
                out.extend([replacement]);
                *replaced = true;
            }
            TokenTree::Group(g) => {
                let mut new_group = Group::new(
                    g.delimiter(),
                    substitute_ident(replaced, g.stream(), target, i),
                );
                new_group.set_span(g.span());
                out.extend(std::iter::once(TokenTree::Group(new_group)));
            }
            tt => {
                out.extend([tt]);
            }
        }
    }
    out
}

enum AbiType {
    Int,
    Double,
    Pointer,
}

fn classify(ty: &Option<Type>) -> syn::Result<AbiType> {
    if let Some(ty) = ty {
        match ty {
            Type::Ptr(ptr) if ptr.mutability.is_some() => Ok(AbiType::Pointer),

            Type::Path(TypePath { path, .. }) => {
                let seg = path.segments.last().unwrap();

                match seg.ident.to_string().as_str() {
                    "i32" => Ok(AbiType::Int),
                    "f64" => Ok(AbiType::Double),
                    _ => Err(syn::Error::new(
                        ty.span(),
                        "expected i32, double or pointer return type",
                    )),
                }
            }

            _ => Err(syn::Error::new(ty.span(), "invalid return type")),
        }
    } else {
        Ok(AbiType::Int)
    }
}

impl ToTokens for AsmInput {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let mut body_tokens = self.body.clone();
        for (i, param) in self.args.iter().enumerate() {
            let mut replaced = false;
            body_tokens = substitute_ident(&mut replaced, body_tokens, &param.to_string(), i);
            if !replaced {
                panic!("Parameter '{param}' is unused");
            }
        }
        let mut script = trim_script(body_tokens.to_string());
        script = script.replace("__EM_ASM_PARAM__", "$");
        script.push('\0');

        let bytes: String = script
            .bytes()
            .map(|byte| format!("0x{byte:x}"))
            .intersperse(",".into())
            .collect();

        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let code_static = format_ident!(
            "__EMSCRIPTEN_ASM_GENERATED__{}",
            COUNTER.fetch_add(1, Ordering::SeqCst)
        );
        let code_static_decl = format!(".globl {code_static}");
        let type_decl = format!(".type {code_static},@object");
        let label = format!("{code_static}:");
        let body = format!(".byte {bytes}",);
        let code_len = script.len() + 1;
        let ret_ty = &self.ret;
        let params = &self.args;

        let mut signature = if let Some(ret_ty) = ret_ty {
            quote! {
                SignatureBuilder::new::<#ret_ty>()
            }
        } else {
            quote! {
                SignatureBuilder::new::<()>()
            }
        };

        for param in params {
            signature = quote! {
                #signature.add_param(&#param)
            }
        }

        let abi_ty = classify(ret_ty).unwrap();

        signature = quote! { #signature.finish() };

        let invoked_fn = match abi_ty {
            AbiType::Int => format_ident!("emscripten_asm_const_int"),
            AbiType::Double => format_ident!("emscripten_asm_const_double"),
            AbiType::Pointer => format_ident!("emscripten_asm_const_ptr"),
        };

        let terminator = if ret_ty.is_none() {
            quote! { ; }
        } else {
            quote! {}
        };

        // global_asm is needed because of rustc bug: https://github.com/rust-lang/rust/issues/146538

        tokens.extend(quote! {
            unsafe {
                unsafe extern "C" {
                    pub unsafe static #code_static: [u8; #code_len];
                }

                mod generated {
                    std::arch::global_asm!(
                        ".section em_asm,\"R\",@",
                        ".p2align 0",
                        #code_static_decl,
                        #type_decl,
                        #label,
                        #body
                    );
                }

                #invoked_fn(
                    #code_static.as_ptr() as _,
                    #signature.as_ptr(),
                    #params
                )
                #terminator
            }
        });
    }
}
