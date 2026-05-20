#![cfg_attr(test, feature(portable_simd, asm_experimental_arch, used_with_arg))]
#![allow(clippy::approx_constant)]

mod binding;
pub use binding::*;

pub mod em_js;

#[cfg(test)]
mod unit_test;

pub const EM_CALLBACK_THREAD_CONTEXT_MAIN_RUNTIME_THREAD: pthread_t = 1 as _;
pub const EM_CALLBACK_THREAD_CONTEXT_CALLING_THREAD: pthread_t = 2 as _;
pub const EM_CALLBACK_THREAD_CONTEXT_MAIN_BROWSER_THREAD: pthread_t =
    EM_CALLBACK_THREAD_CONTEXT_MAIN_RUNTIME_THREAD;
