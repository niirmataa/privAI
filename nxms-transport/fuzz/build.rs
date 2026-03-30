fn main() {
    cc::Build::new()
        .warnings(false)
        .flag_if_supported("-std=c11")
        .file("native/openssl_malloc_shim.c")
        .compile("nxms_fuzz_native");

    println!("cargo:rerun-if-changed=native/openssl_malloc_shim.c");
    println!("cargo:rustc-link-lib=dylib=crypto");
}
