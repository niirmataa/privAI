use std::path::{Path, PathBuf};

fn main() {
    // Allow depending crates (like nxms-mailbox) to use only `wire` without pulling native PQ deps.
    if std::env::var_os("CARGO_FEATURE_CRYPTO").is_none() {
        return;
    }

    // Rebuild if any native sources change.
    println!("cargo:rerun-if-changed=native/");
    println!("cargo:rerun-if-env-changed=NXMS_OQS_LINK_MODE");

    let mut build = cc::Build::new();
    build.warnings(false);
    build.flag_if_supported("-std=c11");
    build.define("FF_FALCON_LOGN", "10"); // Falcon-1024 (logn=10)
    build.define("FALCON_FPEMU", "1");
    build.define("FALCON_FPNATIVE", "0");
    if std::env::var_os("CARGO_FEATURE_FALCON_AUDIT_RAW_API").is_some() {
        build.define("NXMS_FALCON_AUDIT_RAW_API", "1");
    }

    // Include dirs
    build.include("native");
    build.include("native/vendor/falcon");
    build.include("native/nexum_cli_src"); // pqc_* + util

    // NXMS transport
    build.file("native/nxms_ms_transport.c");

    // Nexum CLI PQ wrappers + utilities
    build.file("native/nexum_cli_src/pqc_kem.c");
    build.file("native/nexum_cli_src/pqc_falcon.c");
    build.file("native/nexum_cli_src/util.c");

    // Falcon round3 reference sources (needed by pqc_falcon + nxms_ms_transport)
    for f in [
        "codec.c", "common.c", "falcon.c", "fft.c", "fpr.c", "keygen.c", "rng.c", "shake.c",
        "sign.c", "vrfy.c",
    ] {
        build.file(PathBuf::from("native/vendor/falcon").join(f));
    }

    build.compile("nxms_native");

    emit_optional_search_path("/usr/local/lib");
    emit_optional_search_path("/usr/lib");

    // Link to liboqs (FrodoKEM-640-SHAKE).
    // Keep these after build.compile() so native deps are linked after libnxms_native.
    emit_oqs_link();
    emit_crypto_link();
    // util.c uses libsodium for base64 helpers + secure memory wipes.
    println!("cargo:rustc-link-lib=sodium");
}

fn emit_optional_search_path(path: &str) {
    if Path::new(path).exists() {
        println!("cargo:rustc-link-search=native={path}");
    }
}

fn emit_oqs_link() {
    if let Ok(mode) = std::env::var("NXMS_OQS_LINK_MODE") {
        match mode.as_str() {
            "static" => {
                println!("cargo:rustc-link-lib=static=oqs");
                return;
            }
            "shared" | "dylib" => {
                println!("cargo:rustc-link-lib=dylib=oqs");
                return;
            }
            _ => {}
        }
    }

    let prefer_shared = std::env::var("TARGET")
        .map(|target| target.contains("linux-musl"))
        .unwrap_or(false);

    let shared_candidates = [
        "/usr/local/lib/liboqs.so",
        "/usr/local/lib/liboqs.so.9",
        "/usr/lib/liboqs.so",
    ];
    if prefer_shared && shared_candidates.iter().any(|p| Path::new(p).exists()) {
        println!("cargo:rustc-link-lib=dylib=oqs");
        return;
    }

    println!("cargo:rustc-link-lib=oqs");
}

fn emit_crypto_link() {
    let static_candidates = ["/usr/local/lib/libcrypto.a", "/usr/lib/libcrypto.a"];
    if static_candidates.iter().any(|p| Path::new(p).exists()) {
        println!("cargo:rustc-link-lib=crypto");
        return;
    }

    let shared_candidates = [
        "/usr/local/lib/libcrypto.so",
        "/usr/local/lib/libcrypto.so.3",
        "/usr/lib/libcrypto.so",
        "/usr/lib/libcrypto.so.3",
    ];
    if shared_candidates.iter().any(|p| Path::new(p).exists()) {
        println!("cargo:rustc-link-lib=dylib=crypto");
        return;
    }

    // Fallback for environments where Cargo/toolchain resolves libcrypto implicitly.
    println!("cargo:rustc-link-lib=crypto");
}
