//! Manually builds `playerglobal_avm1.swf` and `playerglobal_avm2.swf` without building the `core` crate.
//! This binary is invoked as:
//! `cargo run --package=build_playerglobal <repo_root> <out_dir>`
//! where `<repo_root>` is the location of the Ruffle repository,
//! and `out_dir` is the directory where the two SWFs should
//! be written

mod cli;

use clap::Parser;

use cli::Commands;
use std::path::{Path, PathBuf};

fn main() {
    // Read at runtime: `env!` would bake in the path of whichever checkout
    // compiled this binary, which may not be the one running it.
    let root = PathBuf::from(
        std::env::var_os("CARGO_MANIFEST_DIR").expect("must be run through `cargo run`"),
    );
    let repo_root = root.join("../../");

    let args = cli::Cli::parse();
    match args.command {
        Commands::Compile { out_dir } => {
            build_playerglobal::build_avm2_playerglobal(&repo_root, Path::new(&out_dir), false)
                .unwrap();
        }
    }
}
