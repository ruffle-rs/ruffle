//! An internal Ruffle utility to build our playerglobal
//! `library.swf`

use std::fs::File;
use std::path::PathBuf;
use std::process::Command;
use swf::DoAbc;
use swf::Header;
use swf::SwfStr;
use swf::Tag;

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

    // This will create 'playerglobal.abc', 'playerglobal.cpp', and 'playerglobal.h'
    // in `out_dir`
    let code = Command::new("java")
        .args(&[
            "-classpath",
            &asc_path.to_string_lossy(),
            "macromedia.asc.embedding.ScriptCompiler",
            "-optimize",
            "-outdir",
            &out_dir.to_string_lossy(),
            "-out",
            "playerglobal",
            "-import",
            &classes_dir.join("stubs.as").to_string_lossy(),
            &classes_dir.join("globals.as").to_string_lossy(),
        ])
        .status()?;
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
