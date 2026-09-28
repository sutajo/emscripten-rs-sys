use proc_macro2::{Group, Literal, TokenStream, TokenTree};
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

        let bytes = Literal::byte_string(script.as_bytes());
        let code_len = script.len();
        let ret_ty = &self.ret;
        let params = &self.args;
        let signature_len = params.len() + 1;

        let mut signature = if let Some(ret_ty) = ret_ty {
            quote! {
                SignatureBuilder::<#signature_len>::new::<#ret_ty>()
            }
        } else {
            quote! {
                SignatureBuilder::<#signature_len>::new::<()>()
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

        tokens.extend(quote! {
            unsafe {
                #[unsafe(link_section = "em_asm")]
                static __EMSCRIPTEN_ASM_CODE: [u8; #code_len] = *#bytes;

                #invoked_fn(
                    __EMSCRIPTEN_ASM_CODE.as_ptr() as _,
                    #signature.as_ptr(),
                    #params
                )
                #terminator
            }
        });
    }
}
