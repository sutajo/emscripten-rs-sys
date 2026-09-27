#![cfg_attr(all(test, feature = "nightly"), feature(asm_experimental_arch))]
#![allow(clippy::approx_constant)]
#![allow(named_asm_labels)]
#![allow(incomplete_features)]
#![cfg_attr(feature = "nightly", feature(const_trait_impl))]
#![cfg_attr(feature = "nightly", feature(unboxed_closures))]
#![cfg_attr(
    feature = "nightly",
    feature(min_generic_const_args, generic_const_args, generic_const_items)
)]
#![cfg_attr(feature = "nightly", feature(const_default))]
#![cfg_attr(feature = "nightly", feature(const_index))]

mod binding;
pub use binding::*;

#[cfg(feature = "nightly")]
pub mod em_asm;
#[cfg(feature = "nightly")]
pub mod em_js;

#[cfg(test)]
mod unit_test;

pub const EM_CALLBACK_THREAD_CONTEXT_MAIN_RUNTIME_THREAD: pthread_t = 1 as _;
pub const EM_CALLBACK_THREAD_CONTEXT_CALLING_THREAD: pthread_t = 2 as _;
pub const EM_CALLBACK_THREAD_CONTEXT_MAIN_BROWSER_THREAD: pthread_t =
    EM_CALLBACK_THREAD_CONTEXT_MAIN_RUNTIME_THREAD;
