use rascal::{CompileOptions, FileSystemSourceProvider, ProgramBuilder, SwfOptions};
use std::path::Path;

/// Builds the `playerglobal_avm1.swf` from the `core/src/avm1/globals/` directory within the repo root.
/// It will use `globals.as` as the entry point within that directory.
pub fn build_avm1_playerglobal(
    repo_root: &Path,
    out_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let source_root = repo_root.join("core/src/avm1/globals/");
    let provider = FileSystemSourceProvider::with_root(source_root);
    let compile_options = CompileOptions::default().with_swf_version(8);
    let mut builder = ProgramBuilder::new(provider).with_compile_options(compile_options);
    builder.add_script("globals.as");
    let swf_bytes = builder.build()?.compile().to_swf(&SwfOptions::default())?;
    let out_path = out_dir.join("playerglobal_avm1.swf");
    std::fs::write(out_path, swf_bytes)?;
    Ok(())
}
