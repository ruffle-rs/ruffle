mod pmd;

use pmd::Pmd;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::process::Command;

const FLEXPMD_PATH: &str = "flexpmd";
const FLEXPMD_CMD_BIN: &str = "flex-pmd-command-line-1.2.jar";
const RULESET: &str = "ruffle-ruleset.xml";
const GLOBALS_PATH: &str = "core/src/avm2/globals";

fn main() {
    let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let tmp = root.join("tmp");
    if let Err(e) = std::fs::create_dir(&tmp) {
        if e.kind() != ErrorKind::AlreadyExists {
            panic!("Failed to create file {}", e);
        }
    }

    let flexpmd = root.join(FLEXPMD_PATH);
    root.pop();
    let global = root.join(GLOBALS_PATH);
    let output = Command::new("java")
        .arg("-jar")
        .arg(flexpmd.join(FLEXPMD_CMD_BIN))
        .arg("-r")
        .arg(flexpmd.join(RULESET))
        .arg("-s")
        .arg(global)
        .arg("-o")
        .arg(&tmp)
        .output()
        .expect("Failed to execute FlexPMD");

    if !output.status.success() {
        panic!("{}", String::from_utf8_lossy(&output.stderr));
    }

    let pmd = Pmd::open(tmp.join("pmd.xml")).expect("Invalid PMD xml file");
    if pmd.contains_violations() {
        eprintln!("{}", pmd);
        std::process::exit(1);
    }
}
