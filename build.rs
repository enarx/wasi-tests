fn main() {
    for (k, v) in std::env::vars() {
        if k.starts_with("CARGO_BIN_FILE_WASI_TESTS_BINS_") {
            println!("cargo:rustc-env={}={}", k, v);
        }
    }
}
