//! Manually builds `playerglobal.swf` without building the `core` crate.
//! This binary is invoked as:
//! `cargo run --package=build_playerglobal <repo_root> <out_dir>`
//! where `<repo_root>` is the location of the Ruffle repository,
//! and `out_dir` is the directory where `playerglobal.swf` should
//! be written

mod cli;
mod pmd;

use clap::Parser;
use pmd::Pmd;

use cli::Commands;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = root.join("../../");

    let args = cli::Cli::parse();
    match args.command {
        Commands::Compile { out_dir } => {
            build_playerglobal::build_playerglobal(repo_root, out_dir.into()).unwrap();
        }
        Commands::Lint => {
            let tmp = root.join("tmp");
            if let Err(e) = std::fs::create_dir(&tmp) {
                if e.kind() != ErrorKind::AlreadyExists {
                    panic!("Failed to create temporary folder {}", e);
                }
            }
            let classes_dir = repo_root.join("core/src/avm2/globals/");
            let flexpmd = root.join("flexpmd");
            let output = Command::new("java")
                .arg("-jar")
                .arg(flexpmd.join("flex-pmd-command-line-1.2.jar"))
                .arg("-r")
                .arg(flexpmd.join("ruffle-ruleset.xml"))
                .arg("-s")
                .arg(&classes_dir)
                .arg("-o")
                .arg(&tmp)
                .output()
                .expect("Failed to execute FlexPMD");

            if !output.status.success() {
                panic!("{}", String::from_utf8_lossy(&output.stderr));
            }

            let pmd = Pmd::open(classes_dir, tmp.join("pmd.xml")).expect("Invalid PMD xml file");
            std::fs::remove_dir_all(tmp).expect("Failed to delete temp folder");
            if pmd.contains_violations() {
                eprintln!("{}", pmd);
                std::process::exit(1);
            }
        }
    }
}
