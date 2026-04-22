mod rascal;

use crate::compiler::rascal::RascalOptions;
use serde::Deserialize;
use vfs::VfsPath;

#[derive(Clone, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
#[serde(tag = "type")]
pub enum SwfCompilerOptions {
    Rascal(RascalOptions),
}

impl SwfCompilerOptions {
    pub fn validate(&self) -> anyhow::Result<()> {
        match self {
            SwfCompilerOptions::Rascal(options) => options.validate(),
        }
    }

    pub fn create_compiler(&self) -> anyhow::Result<Box<dyn SwfCompiler>> {
        match self {
            SwfCompilerOptions::Rascal(options) => options.create_compiler(),
        }
    }
}

pub trait SwfCompiler {
    fn compile(self: Box<Self>, root_dir: &VfsPath) -> anyhow::Result<()>;
}
