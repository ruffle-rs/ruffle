fn main() {
    build_playerglobal::build_avm2_playerglobal(
        "../".into(),
        std::env::var("OUT_DIR").unwrap().into(),
        cfg!(feature = "known_stubs"),
    )
    .expect("Failed to build playerglobal_avm2");

    // This is overly conservative - it will cause us to rebuild playerglobal_avm2.swf
    // if *any* files in this directory change, not just .as files.
    // However, this script is fast to run, so it shouldn't matter in practice.
    // If Cargo ever adds glob support to 'rerun-if-changed', we should use it.
    println!("cargo:rerun-if-changed=src/avm2/globals/");
}
