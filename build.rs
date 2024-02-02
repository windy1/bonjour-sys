extern crate bindgen;

use std::env;
use std::path::PathBuf;

#[cfg(target_vendor = "apple")]
fn main() {
    bindgen::Builder::default()
        .header("wrapper.h")
        .ctypes_prefix("::libc")
        .generate()
        .expect("failed to generate bindings")
        .write_to_file(PathBuf::from(env::var("OUT_DIR").unwrap()).join("bindings.rs"))
        .expect("failed to write bindings to file");
}

#[cfg(target_vendor = "pc")]
fn main() {
    use std::io::{Read, Seek, Write};
    let bindings_rs_path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("bindings.rs");

    // Generate the dns_sd.h bindings first so we can
    // mark them as being available in dnssd.dll.
    bindgen::Builder::default()
        .header("wrapper.h")
        .ctypes_prefix("::libc")
        .allowlist_file(".*dns_sd.h")
        .allowlist_recursively(false)
        .generate()
        .expect("failed to generate dns_sd.h bindings")
        .write_to_file(&bindings_rs_path)
        .expect("failed to write dns_sd.h bindings to file");

    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .read(true)
        .open(&bindings_rs_path)
        .unwrap();
    let mut contents: String = String::new();
    file.read_to_string(&mut contents).unwrap();
    contents = contents.replace(
        "extern \"C\" {",
        "#[link(name = \"dnssd\", kind = \"raw-dylib\")]\r\nextern \"C\" {",
    );
    file.seek(std::io::SeekFrom::Start(0)).unwrap();
    file.set_len(0).unwrap();
    file.write_all(contents.as_bytes())
        .expect("Unable to write data");

    // Generate bindings for everything else
    // Certain windows types we don't need have some issues with rust & bindgen
    // Further discussion: https://github.com/rust-lang/rust-bindgen/issues/1562
    bindgen::Builder::default()
        .header("wrapper.h")
        .ctypes_prefix("::libc")
        .blocklist_file(".*dns_sd.h")
        .blocklist_type("IMAGE_TLS_DIRECTORY")
        .blocklist_type("PIMAGE_TLS_DIRECTORY")
        .blocklist_type("IMAGE_TLS_DIRECTORY64")
        .blocklist_type("PIMAGE_TLS_DIRECTORY64")
        .blocklist_type("_IMAGE_TLS_DIRECTORY64")
        .generate()
        .expect("failed to generate system bindings")
        .write_to_file(&PathBuf::from(env::var("OUT_DIR").unwrap()).join("bindings2.rs"))
        .expect("failed to write system bindings to file");
}
