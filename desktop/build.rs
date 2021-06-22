use std::env;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    // This build script to generate Git version info is from the rustfmt project

    // Only check .git/HEAD dirty status if it exists - doing so when
    // building dependent crates may lead to false positives and rebuilds
    println!("cargo:rerun-if-changed=build.rs");
    if Path::new(".git/HEAD").exists() {
        println!("cargo:rerun-if-changed=.git/HEAD");
    }

    // Embed resource file w/ icon on windows
    // To allow for cross-compilation, this must not be behind cfg(windows)!
    println!("cargo:rerun-if-changed=assets/ruffle_desktop.rc");
    embed_resource::compile("assets/ruffle_desktop.rc");

    println!("cargo:rerun-if-env-changed=CFG_RELEASE_CHANNEL");
    if option_env!("CFG_RELEASE_CHANNEL").map_or(true, |c| c == "nightly" || c == "dev") {
        println!("cargo:rustc-cfg=nightly");
    }

    let out_dir: PathBuf = env::var_os("OUT_DIR").unwrap().into();

    File::create(out_dir.join("version-info.txt"))
        .unwrap()
        .write_all(commit_info().as_bytes())
        .unwrap();
}

// Try to get hash and date of the last commit on a best effort basis. If anything goes wrong
// (git not installed or if this is not a git repository) just return an empty string.
fn commit_info() -> String {
    match (channel(), commit_hash(), commit_date()) {
        (channel, Some(hash), Some(date)) => format!(
            "{}-{} ({} {})",
            option_env!("CARGO_PKG_VERSION").unwrap_or("unknown"),
            channel,
            hash.trim_end(),
            date
        ),
        _ => String::new(),
    }
}

fn channel() -> String {
    if let Ok(channel) = env::var("CFG_RELEASE_CHANNEL") {
        channel
    } else {
        "nightly".to_owned()
    }
}

fn commit_hash() -> Option<String> {
    Command::new("git")
        .args(&["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|r| String::from_utf8(r.stdout).ok())
}

fn commit_date() -> Option<String> {
    Command::new("git")
        .args(&["log", "-1", "--date=short", "--pretty=format:%cd"])
        .output()
        .ok()
        .and_then(|r| String::from_utf8(r.stdout).ok())
}
