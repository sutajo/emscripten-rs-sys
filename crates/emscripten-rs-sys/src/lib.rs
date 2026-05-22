#![cfg_attr(test, feature(portable_simd, asm_experimental_arch, used_with_arg))]
#![allow(clippy::approx_constant)]
#![allow(named_asm_labels)]
#![allow(incomplete_features)]
#![feature(const_trait_impl)]
#![feature(unboxed_closures)]
#![feature(generic_const_exprs)]
#![feature(const_default)]
#![feature(const_index)]

mod binding;
pub use binding::*;

pub mod em_js;
pub mod em_asm;

#[cfg(test)]
mod unit_test;

pub const EM_CALLBACK_THREAD_CONTEXT_MAIN_RUNTIME_THREAD: pthread_t = 1 as _;
pub const EM_CALLBACK_THREAD_CONTEXT_CALLING_THREAD: pthread_t = 2 as _;
pub const EM_CALLBACK_THREAD_CONTEXT_MAIN_BROWSER_THREAD: pthread_t =
    EM_CALLBACK_THREAD_CONTEXT_MAIN_RUNTIME_THREAD;
