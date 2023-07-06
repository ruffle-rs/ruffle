fn main() {
    // See 'desktop/build.rs' for more information.
    if std::env::var("TARGET").unwrap().contains("wasm") {
        println!("cargo:rustc-link-arg=-z");
        println!("cargo:rustc-link-arg=stack-size=2000000");
    }
}
