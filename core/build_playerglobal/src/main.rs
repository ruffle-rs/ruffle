//! Manually builds `playerglobal.swf` without building the `core` crate.
//! This binary is invoked as:
//! `cargo run --package=build_playerglobal <repo_root> <out_dir>`
//! where `<repo_root>` is the location of the Ruffle repository,
//! and `out_dir` is the directory where `playerglobal.swf` should
//! be written

mod cli;

use clap::Parser;

use cli::Commands;
use std::path::PathBuf;

fn main() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = root.join("../../");

    let args = cli::Cli::parse();
    match args.command {
        Commands::Compile { out_dir } => {
            build_playerglobal::build_playerglobal(repo_root, out_dir.into(), false).unwrap();
        }
    }
}
