fn main() {
    #[cfg(all(target_os = "linux", not(feature = "wasm")))]
    println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN");
}
