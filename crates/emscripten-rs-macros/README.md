# emscripten-rs-macros

Procedural macros for [emscripten_rs_sys](https://docs.rs/emscripten_rs_sys).
Use them through the sys crate, which also provides the FFI types and helpers:

```toml
emscripten_rs_sys = "0.4.0"
```

- `js!` declares JavaScript functions callable from Rust, including async functions.
- `inline_js!` executes JavaScript with explicitly typed arguments and a return type.
- `js_asm!` executes JavaScript using inferred Rust argument types.

The supported target is `wasm32-unknown-emscripten`. Activate the Emscripten SDK
before linking an application.

## Export configuration

By default, `js!` and `inline_js!` require `-C link-dead-code` in the calling
application's Rust compiler flags. They need no experimental compiler features.

The optional `force_export` feature emits WebAssembly global assembly to export
the script symbols without `-C link-dead-code`. Enable it through the sys crate:

```toml
emscripten_rs_sys = { version = "0.4.0", features = ["force_export"] }
```

This mode requires nightly Rust and `#![feature(asm_experimental_arch)]` in each
crate that invokes `js!` or `inline_js!`.

`js_asm!` also needs a compiler containing the
[Wasm `link_section` fix](https://github.com/rust-lang/rust/commit/281be7d22d011325881b39a57b1292c11d8f262e).
The workspace currently uses nightly for that fix; stable Rust 1.98 predates it.
See the [sys crate documentation](https://docs.rs/emscripten_rs_sys) for setup and examples.
