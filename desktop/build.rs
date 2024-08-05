use chrono::Datelike;
use std::env;
use std::error::Error;
use vergen::EmitBuilder;
use winresource::{VersionInfo, WindowsResource};

const DEBUG_FILE_FLAG: u64 = 0x01;
const ENGLISH_LANG_ID: u16 = 0x0009;

fn main() -> Result<(), Box<dyn Error>> {
    // Emit version info, and "rerun-if-changed" for relevant files, including build.rs
    EmitBuilder::builder()
        .build_timestamp()
        .cargo_features()
        .git_sha(false)
        .git_commit_timestamp()
        .git_commit_date()
        .emit()?;

    // Embed metadata file for Windows
    // To allow for cross-compilation, we need to use env::var as cgf!(target_os == "windows")
    // doesn't work in build.rs files
    if env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
        let current_year = chrono::Local::now().year();
        let mut resource = WindowsResource::new();
        // The version number is set automatically
        // TODO Update languages when https://github.com/BenjaminRi/winresource/issues/20
        // has been resolved
        resource
            .set("ProductName", "Ruffle")
            .set("FileDescription", "Ruffle")
            .set("InternalName", "ruffle_desktop")
            .set("OriginalFilename", "ruffle_desktop.exe")
            .set(
                "LegalCopyright",
                &format!("Created by the Ruffle Team ({current_year})"),
            )
            .set("CompanyName", "The Ruffle Team")
            .set_icon("packaging/Windows/favicon.ico")
            .set_language(ENGLISH_LANG_ID);

        if cfg!(debug_assertions) {
            resource.set_version_info(VersionInfo::FILEFLAGS, DEBUG_FILE_FLAG);
        }

        resource
            .compile()
            .expect("The resource file must be compiled.");
    }

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
        "nightly".to_owned()
    }
}
