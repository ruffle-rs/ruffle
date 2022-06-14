//! An internal Ruffle utility to build our playerglobal
//! `library.swf`

use std::collections::HashSet;
use std::fs::File;
use std::path::PathBuf;
use std::process::Command;
use swf::DoAbc;
use swf::Header;
use swf::SwfStr;
use swf::Tag;
use walkdir::WalkDir;

/// If successful, returns a list of paths that were used. If this is run
/// from a build script, these paths should be printed with
/// cargo:rerun-if-changed
pub fn build_playerglobal(
    repo_root: PathBuf,
    out_dir: PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let classes_dir = repo_root.join("core/src/avm2/globals/");
    let asc_path = repo_root.join("core/build_playerglobal/asc.jar");

    let out_path = out_dir.join("playerglobal.swf");

    // These classes are currently stubs - they're referenced by
    // other classes that we need to compile, but the real definition
    // is in Ruffle itself (in Rust code).
    // As a result, we don't emit them into the final SWF (but we do
    // provide them to asc.jar with '-import' to link against).
    let stub_classes: HashSet<_> = ["Object", "Number", "Boolean", "String"].into();

    // This will create 'playerglobal.abc', 'playerglobal.cpp', and 'playerglobal.h'
    // in `out_dir`
    let mut cmd = Command::new("java");
    cmd.args(&[
        "-classpath",
        &asc_path.to_string_lossy(),
        "macromedia.asc.embedding.ScriptCompiler",
        "-outdir",
        &out_dir.to_string_lossy(),
        "-out",
        "playerglobal",
    ]);

    for entry in WalkDir::new(&classes_dir) {
        let entry = entry?;
        if entry.path().extension().and_then(|e| e.to_str()) != Some("as") {
            continue;
        }
        let class = entry.into_path();
        let class_name: String = class
            .strip_prefix(&classes_dir)?
            .with_extension("")
            .iter()
            .map(|c| c.to_string_lossy())
            .collect::<Vec<_>>()
            .join("/");

        if stub_classes.contains(class_name.as_str()) {
            cmd.arg("-import");
        }
        cmd.arg(class);
    }

    println!("Compiling: {:?}", cmd);
    let code = cmd.status()?;
    if !code.success() {
        return Err(format!("Compiling failed with code {:?}", code).into());
    }

    let playerglobal = out_dir.join("playerglobal");
    let bytes = std::fs::read(playerglobal.with_extension("abc"))?;

    // Cleanup the temporary files written out by 'asc.jar'
    std::fs::remove_file(playerglobal.with_extension("abc"))?;
    std::fs::remove_file(playerglobal.with_extension("cpp"))?;
    std::fs::remove_file(playerglobal.with_extension("h"))?;

    let tags = vec![Tag::DoAbc(DoAbc {
        name: SwfStr::from_utf8_str(""),
        is_lazy_initialize: true,
        data: &bytes,
    })];

    let header = Header::default_with_swf_version(19);
    let out_file = File::create(out_path).unwrap();
    swf::write_swf(&header, &tags, out_file)?;

    Ok(())
}
