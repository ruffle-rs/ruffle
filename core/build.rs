fn main() {
    let paths = build_playerglobal::build_playerglobal(
        "../".into(),
        std::env::var("OUT_DIR").unwrap().into(),
    )
    .expect("Failed to build playerglobal");

    for path in paths {
        println!("cargo:rerun-if-changed={}", path.to_string_lossy());
    }
}
