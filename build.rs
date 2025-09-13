use std::fs::{self, FileType};
use which::which;

fn main() {
    let compiler = which("emcc").expect("emcc not found");
    let sysroot = compiler.parent().unwrap().join("cache").join("sysroot");
    let excluded_headers = ["wire.h"];

    let headers = fs::read_dir(sysroot.join("include").join("emscripten"))
        .expect("include directory not found")
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_type()
                .as_ref()
                .map(FileType::is_file)
                .unwrap_or(false)
        })
        .filter_map(|e| {
            let path = e.path();
            let is_header = path
                .extension()
                .map(|extension| extension == "h")
                .unwrap_or(false);
            if is_header
                && !excluded_headers
                    .iter()
                    .any(|excluded| path.ends_with(excluded))
            {
                println!("{}", path.display());
                Some(path.to_str().unwrap().to_string())
            } else {
                None
            }
        });

    bindgen::Builder::default()
        .headers(headers)
        .clang_arg("-x")
        .clang_arg("c++")
        .clang_arg("-std=c++20")
        .clang_arg("-Wno-nullability-completeness")
        .clang_arg("-Wno-return-type-c-linkage")
        .clang_arg("-Wno-pragma-once-outside-header")
        .clang_arg("-fvisibility=default") // https://github.com/rust-lang/rust-bindgen/issues/2989
        .clang_arg(format!("--sysroot={}", sysroot.display()))
        .clang_arg(format!("-I{}", sysroot.join("include").display()))
        .clang_arg(format!(
            "-I{}",
            sysroot.join("include").join("compat").display()
        ))
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .wrap_unsafe_ops(true)
        .allowlist_file(format!("{}/emscripten/.*", sysroot.display()))
        .allowlist_item(".*_?emscripten_.*")
        .allowlist_item(".*_?EMSCRIPTEN_.*")
        .allowlist_item(".*_?emval_.*")
        .allowlist_item(".*_?EMVAL_.*")
        .allowlist_item(".*_?em_.*")
        .allowlist_item(".*_?EM_.*")
        .blocklist_item("std::.*")
        .raw_line("#![allow(non_camel_case_types)]")
        .raw_line("#![allow(non_snake_case)]")
        .raw_line("#![allow(non_upper_case_globals)]")
        .derive_default(true)
        .derive_eq(true)
        .derive_hash(true)
        .generate()
        .expect("Binding generation failed")
        .write_to_file("src/binding.rs")
        .expect("Could not write binding to file");

    println!("cargo::rustc-link-arg-examples=-sEXPORTED_RUNTIME_METHODS=['webSockets']");
}
