extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn main() {
    bindgen::Builder::default()
        .header("wrapper.h")
        .ctypes_prefix("::libc")
        .generate()
        .expect("failed to generate bindings")
        .write_to_file(PathBuf::from(env::var("OUT_DIR").unwrap()).join("bindings.rs"))
        .expect("failed to write bindings to file");
}
