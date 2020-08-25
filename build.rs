extern crate bindgen;

use std::path::PathBuf;
use std::env;

fn main() {
    bindgen::Builder::default()
        .header("wrapper.h")
        .generate()
        .expect("failed to generate bindings")
        .write_to_file(PathBuf::from(env::var("OUT_DIR").unwrap()).join("bindings.rs"))
        .expect("failed to write bindings to file");
}
