# emscripten-rs-sys

[![Crates.io](https://img.shields.io/crates/v/emscripten_rs_sys.svg)](https://crates.io/crates/emscripten_rs_sys)
[![Docs.rs](https://img.shields.io/docsrs/emscripten_rs_sys)](https://img.shields.io/docsrs/emscripten_rs_sys)
![License](https://img.shields.io/crates/l/emscripten_rs_sys.svg)

Low-level Rust FFI bindings to the [Emscripten](https://emscripten.org/) C API, generated using [bindgen](https://github.com/rust-lang/rust-bindgen).

This crate provides raw, unsafe bindings to the C API of Emscripten, allowing Rust code to interface directly with the Emscripten runtime.

⚠️ The only supported target is `wasm32-unknown-emscripten`.

## Features

The raw FFI bindings and the `em_js` (`js!`, `inline_js!`) and `em_asm`
(`js_asm!`) macros are available by default. The default configuration requires
no experimental compiler feature attributes in the calling crate.

**Using `em_js` (`js!` or `inline_js!`) requires `-Clink-dead-code`**
(equivalently, `-C link-dead-code`) unless you enable `force_export`.
See [Linker setup](#linker-setup) for the Cargo configuration.

```toml
emscripten_rs_sys = "0.4.0"
```

The optional `force_export` feature uses WebAssembly global assembly to export
`em_js` script symbols, removing the need for `-C link-dead-code`. It requires
nightly Rust:

```toml
emscripten_rs_sys = { version = "0.4.0", features = ["force_export"] }
```

Because the assembly is emitted by the macros, each crate that calls `js!` or
`inline_js!` with this feature enabled must add this to its crate root:

```rust
#![feature(asm_experimental_arch)]
```

When using `SignatureBuilder` directly, specify its buffer capacity (the number
of arguments plus one for the null terminator), for example
`SignatureBuilder::<3>::new::<i32>().add_param(&x).add_param(&y).finish()`.
The `js_asm!` macro calculates this capacity automatically.

## Prerequisites

You must have the **Emscripten SDK (emsdk)** installed and activated on your system before building.

Follow the official installation instructions here: [Emscripten SDK Installation Guide](https://emscripten.org/docs/getting_started/downloads.html)

After installing, ensure `emcc` is on your PATH for linking and optional binding
generation. Both macro modules emit their JavaScript strings directly from Rust;
macro expansion does not compile C or generate native archives.

The project currently selects nightly for the
[Wasm `link_section` fix](https://github.com/rust-lang/rust/commit/281be7d22d011325881b39a57b1292c11d8f262e).
Stable Rust 1.98 predates that fix and cannot link `js_asm!` correctly. The fix
itself needs no feature attribute; `force_export` separately requires the
nightly assembly feature described above.

## Linker setup

Without `force_export`, using `em_js` (`js!` or `inline_js!`) requires the Rust
compiler flag `-C link-dead-code`. This preserves the exported `__em_js__*` data
symbols that Emscripten reads to discover the JavaScript functions. Add it to your application's
`.cargo/config.toml` (append it to any existing `rustflags`):

```toml
[target.wasm32-unknown-emscripten]
rustflags = ["-C", "link-dead-code"]
```

This repository already enables the flag for its tests. It retains otherwise
unused code, so it can increase the output size. When enabling `force_export`,
you can remove this flag while retaining any other linker settings you need.

Some APIs of Emscripten only work at runtime if they are explicitly enabled at link time.

See the [settings Reference](https://emscripten.org/docs/tools_reference/settings_reference.html) for all `-s` options.

Ways to set linker settings:

- With the RUSTFLAGS environment variable: `RUSTFLAGS='-C link-args=-sALLOW_MEMORY_GROWTH=1'`
- In the build script: `println!("cargo:rustc-link-arg=-sALLOW_MEMORY_GROWTH=1");`
- In the `.cargo/config.toml` with the `rustflags` field.

## Highlights

- The complete C API is covered.
- `EM_JS` support for inline JS functions.

## Example

Define Javascript function in Rust:

```rust,no_run
# #![cfg_attr(feature = "force_export", feature(asm_experimental_arch))]
use emscripten_rs_sys::em_js::js;
use std::ffi::c_int;

js! {
    fn compute_sum(n: c_int) -> c_int
    {
        let sum = 0;
        for(let i=1; i<n; i++)
        {
            sum += i;
        }
        return sum;
    }
}

fn test_sum() {
    assert_eq!(unsafe { compute_sum(100) }, 4950)
}
```

Conveniently call Javascript from Rust, with parameter type inference:

```rust,no_run
use emscripten_rs_sys::em_asm::*;

fn js_asm() {
    let a = 2;
    let b = 3;
    let result = js_asm! { 
        |a,b| -> i32 { return a+b*b; } 
    };
    assert_eq!(result, 2+3*3);
}
```
