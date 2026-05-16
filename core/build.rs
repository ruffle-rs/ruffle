use std::path::{Path, PathBuf};

fn main() {
    let repo_root = Path::new("../");
    let out_dir = PathBuf::from(std::env::var_os("OUT_DIR").unwrap());

    build_playerglobal::build_avm1_playerglobal(repo_root, &out_dir)
        .expect("Failed to build playerglobal_avm1");
    build_playerglobal::build_avm2_playerglobal(repo_root, &out_dir, cfg!(feature = "known_stubs"))
        .expect("Failed to build playerglobal_avm2");

    // This is overly conservative - it will cause us to rebuild playerglobals
    // if *any* files in these directories change, not just .as files.
    // However, this script is fast to run, so it shouldn't matter in practice.
    // If Cargo ever adds glob support to 'rerun-if-changed', we should use it.
    println!("cargo:rerun-if-changed=src/avm1/globals/");
    println!("cargo:rerun-if-changed=src/avm2/globals/");
}
