fn main() {
    // librdkafka's Windows SASL/SSPI code needs these; MSVC auto-links them
    // but MinGW does not, leaving `AcquireCredentialsHandleW` etc. unresolved.
    // Must be appended *after* the object files that reference them —
    // GNU ld resolves static/import libs left-to-right, and
    // `rustc-link-lib` alone places them too early in the link line.
    if std::env::var("CARGO_CFG_WINDOWS").is_ok() {
        println!("cargo:rustc-link-arg-bins=-lsecur32");
        println!("cargo:rustc-link-arg-bins=-lcrypt32");
    }
}
