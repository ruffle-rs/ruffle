use crate::compiler::SwfCompiler;
use crate::util::{read_bytes, write_bytes_and_verify_if_changed};
use serde::Deserialize;
use swfasm::files::FileProvider;
use vfs::VfsPath;

#[derive(Clone, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct SwfasmOptions {
    pub target: String,
    pub source: String,
}

impl SwfasmOptions {
    pub fn validate(&self) -> anyhow::Result<()> {
        Ok(())
    }

    pub fn create_compiler(&self) -> anyhow::Result<Box<dyn SwfCompiler>> {
        Ok(Box::new(SwfasmCompiler {
            target: self.target.clone(),
            source: self.source.clone(),
        }))
    }
}

#[derive(Debug)]
pub struct SwfasmCompiler {
    target: String,
    source: String,
}

impl SwfCompiler for SwfasmCompiler {
    fn compile(self: Box<Self>, root_dir: &VfsPath, verify_if_changed: bool) -> anyhow::Result<()> {
        let source_path = root_dir.join(&self.source)?;
        let source = source_path.read_to_string()?;

        // Paths in the source are relative to the directory of the source.
        let files = VfsFileProvider(source_path.parent());
        let swf = swfasm::compile_str(&source, &files)
            .map_err(|e| anyhow::anyhow!("Failed to compile '{}': {e:#}", self.source))?;
        let output_path = root_dir.join(&self.target)?;

        write_bytes_and_verify_if_changed(&output_path, &swf, verify_if_changed)
    }
}

struct VfsFileProvider(VfsPath);

impl FileProvider for VfsFileProvider {
    fn read(&self, path: &str) -> anyhow::Result<Vec<u8>> {
        Ok(read_bytes(&self.0.join(path)?)?)
    }
}
