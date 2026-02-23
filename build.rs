use bindgen::Abi;
use std::env;
use std::path::PathBuf;

fn main() {
    use std::io::{Read, Seek, Write};
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();

    let bindings_rs_path = PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("bindings.rs");

    // Generate the dns_sd.h bindings first so we can
    // add some marks eg. #[link] and abi.
    let mut builder = bindgen::Builder::default()
        .header("wrapper.h")
        .ctypes_prefix("::libc")
        .allowlist_file(".*dns_sd.h")
        .allowlist_recursively(false)
        .trust_clang_mangling(false)
        .override_abi(Abi::System, "DNSService.*")
        .override_abi(Abi::System, "TXTRecord.*")
        .rust_edition(bindgen::RustEdition::Edition2024);

    // FreeBSD: add include path for mDNSResponder headers
    if target_os == "freebsd" {
        builder = builder.clang_arg("-I/usr/local/include");
    }

    builder
        .generate()
        .expect("failed to generate dns_sd.h bindings")
        .write_to_file(&bindings_rs_path)
        .expect("failed to write dns_sd.h bindings to file");

    if target_os == "windows" {
        // On Windows, we need to mark the functions as being available in dnssd.dll
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .read(true)
            .open(&bindings_rs_path)
            .unwrap();

        let mut contents: String = String::new();

        file.read_to_string(&mut contents).unwrap();

        if target_arch == "x86" {
            // Decorated names are used by default on Win x86,
            // so we need to specify import_name_type = "decorated"
            contents = contents.replace(
                "unsafe extern \"system\" {",
                "#[link(name = \"dnssd\", kind = \"raw-dylib\", import_name_type = \"undecorated\")]\r\nunsafe extern \"system\" {",
            );
        } else {
            contents = contents.replace(
                "unsafe extern \"system\" {",
                "#[link(name = \"dnssd\", kind = \"raw-dylib\")]\r\nunsafe extern \"system\" {",
            );
        }

        file.seek(std::io::SeekFrom::Start(0)).unwrap();
        file.set_len(0).unwrap();
        file.write_all(contents.as_bytes())
            .expect("Unable to write data");
    }

    // FreeBSD: link to mDNSResponder library
    if target_os == "freebsd" {
        println!("cargo:rustc-link-search=/usr/local/lib");
        println!("cargo:rustc-link-lib=dns_sd");
    }

    // Generate bindings for everything else
    // Certain windows types we don't need have some issues with rust & bindgen
    // Further discussion: https://github.com/rust-lang/rust-bindgen/issues/1562
    let mut builder2 = bindgen::Builder::default()
        .header("wrapper.h")
        .ctypes_prefix("::libc")
        .rust_edition(bindgen::RustEdition::Edition2024)
        .blocklist_file(".*dns_sd.h")
        .blocklist_type("IMAGE_TLS_DIRECTORY")
        .blocklist_type("PIMAGE_TLS_DIRECTORY")
        .blocklist_type("IMAGE_TLS_DIRECTORY64")
        .blocklist_type("PIMAGE_TLS_DIRECTORY64")
        .blocklist_type("_IMAGE_TLS_DIRECTORY64");

    // FreeBSD: add include path for second builder too
    if target_os == "freebsd" {
        builder2 = builder2.clang_arg("-I/usr/local/include");
    }

    builder2
        .generate()
        .expect("failed to generate system bindings")
        .write_to_file(PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("bindings2.rs"))
        .expect("failed to write system bindings to file");
}
