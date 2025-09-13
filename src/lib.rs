#![cfg_attr(
    feature = "em_js",
    feature(asm_experimental_arch, macro_metavar_expr_concat, link_arg_attribute)
)]
#![cfg_attr(all(feature = "em_js", test), feature(portable_simd))]
#![allow(clippy::approx_constant)]

pub mod binding;
pub use binding::*;

#[cfg(feature = "em_js")]
pub mod em_js;

#[cfg(test)]
mod unit_test;

pub const EM_CALLBACK_THREAD_CONTEXT_MAIN_RUNTIME_THREAD: pthread_t = 1 as _;
pub const EM_CALLBACK_THREAD_CONTEXT_CALLING_THREAD: pthread_t = 2 as _;
pub const EM_CALLBACK_THREAD_CONTEXT_MAIN_BROWSER_THREAD: pthread_t =
    EM_CALLBACK_THREAD_CONTEXT_MAIN_RUNTIME_THREAD;
