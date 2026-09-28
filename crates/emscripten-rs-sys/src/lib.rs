#![cfg_attr(feature = "force_export", feature(asm_experimental_arch))]
#![allow(clippy::approx_constant)]
#![doc = include_str!("../README.md")]

mod binding;
pub use binding::*;

pub mod em_asm;
pub mod em_js;

#[cfg(test)]
mod unit_test;

pub const EM_CALLBACK_THREAD_CONTEXT_MAIN_RUNTIME_THREAD: pthread_t = 1 as _;
pub const EM_CALLBACK_THREAD_CONTEXT_CALLING_THREAD: pthread_t = 2 as _;
pub const EM_CALLBACK_THREAD_CONTEXT_MAIN_BROWSER_THREAD: pthread_t =
    EM_CALLBACK_THREAD_CONTEXT_MAIN_RUNTIME_THREAD;
