//! Manually builds `playerglobal.swf` without building the `core` crate.
//! This binary is invoked as:
//! `cargo run --package=build_playerglobal <repo_root> <out_dir>`
//! where `<repo_root>` is the location of the Ruffle repository,
//! and `out_dir` is the directory where `playerglobal.swf` should
//! be written

fn main() {
    build_playerglobal::build_playerglobal(
        std::env::args().nth(1).unwrap().into(),
        std::env::args().nth(2).unwrap().into(),
    )
    .unwrap();
}
