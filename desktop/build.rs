use std::env;
use std::error::Error;
use vergen::EmitBuilder;

fn main() -> Result<(), Box<dyn Error>> {
    // Emit version info, and "rerun-if-changed" for relevant files, including build.rs
    EmitBuilder::builder()
        .build_timestamp()
        .cargo_features()
        .git_sha(false)
        .git_commit_timestamp()
        .git_commit_date()
        .emit()?;

    // Embed resource file w/ icon on windows
    // To allow for cross-compilation, this must not be behind cfg(windows)!
    println!("cargo:rerun-if-changed=assets/ruffle_desktop.rc");
    embed_resource::compile("assets/ruffle_desktop.rc", embed_resource::NONE);

    println!("cargo:rerun-if-env-changed=CFG_RELEASE_CHANNEL");
    let channel = channel();
    if channel == "nightly" || channel == "dev" {
        println!("cargo:rustc-cfg=nightly");
    }
    println!("cargo:rustc-env=CFG_RELEASE_CHANNEL={channel}");

    // Some SWFS have a large amount of recursion (particularly
    // around `goto`s). Increase the stack size on Windows
    // accommodate this (the default on Linux is high enough). We
    // do the same thing for wasm in web/build.rs.
    if std::env::var("TARGET").unwrap().contains("windows") {
        if std::env::var("TARGET").unwrap().contains("msvc") {
            println!("cargo:rustc-link-arg=/STACK:4000000");
        } else {
            println!("cargo:rustc-link-arg=-Xlinker");
            println!("cargo:rustc-link-arg=--stack");
            println!("cargo:rustc-link-arg=4000000");
        }
    }

    Ok(())
}

fn channel() -> String {
    if let Ok(channel) = env::var("CFG_RELEASE_CHANNEL") {
        channel
    } else {
        "none".to_owned()
    }
}
