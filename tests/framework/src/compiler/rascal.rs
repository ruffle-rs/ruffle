use crate::compiler::SwfCompiler;
use crate::util::{read_bytes, write_bytes};
use rascal::{CompileOptions, OptimizationOptions, ProgramBuilder, SourceProvider, SwfOptions};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::io::Error;
use vfs::VfsPath;

#[derive(Clone, Deserialize, Debug)]
#[serde(default, deny_unknown_fields)]
pub struct RascalOptions {
    pub target: String,
    pub swf_version: Option<u8>,
    pub frame_rate: f32,
    pub scripts: Vec<String>,
    pub classes: Vec<String>,
    pub pcode: Vec<String>,
    pub stage_rect: StageSize,
    pub use_network: bool,
    pub optimizations: RascalOptimizations,
}

#[derive(Clone, Deserialize, Debug)]
#[serde(default, deny_unknown_fields)]
pub struct RascalOptimizations {
    pub fold_constants: bool,
    pub promote_variables_to_registers: bool,
}

impl Default for RascalOptimizations {
    fn default() -> Self {
        Self {
            fold_constants: true,
            promote_variables_to_registers: true,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct StageSize {
    pub x_min: f64,
    pub y_min: f64,
    pub x_max: f64,
    pub y_max: f64,
}

impl Default for StageSize {
    fn default() -> Self {
        Self {
            x_min: 0.0,
            y_min: 0.0,
            x_max: 550.0,
            y_max: 400.0,
        }
    }
}

#[derive(Debug)]
pub struct RascalCompiler {
    target: String,
    swf_options: SwfOptions,
    compile_options: CompileOptions,
    scripts: Vec<String>,
    classes: Vec<String>,
    pcode: Vec<String>,
    optimizations: OptimizationOptions,
}

impl SwfCompiler for RascalCompiler {
    fn compile(self: Box<Self>, root_dir: &VfsPath, verify_if_changed: bool) -> anyhow::Result<()> {
        let provider = VfsSourceProvider(root_dir.clone());
        let mut builder = ProgramBuilder::new(provider).with_optimizations(self.optimizations);
        for script in &self.scripts {
            builder.add_script(script);
        }
        for class in &self.classes {
            builder.add_class(class);
        }
        for pcode in &self.pcode {
            builder.add_pcode(pcode);
        }
        let program = builder.build()?;
        let swf = program
            .compile(self.compile_options)
            .to_swf(&self.swf_options)?;
        let output_path = root_dir.join(&self.target)?;
        if verify_if_changed {
            if !output_path.is_file()? {
                write_bytes(&output_path, &swf)?;
                return Err(anyhow::anyhow!(
                    "Output file '{}' does not exist or is not a file",
                    self.target
                ));
            }
            let mut existing_hash = Sha256::new();
            existing_hash.update(read_bytes(&output_path)?);
            let mut new_hash = Sha256::new();
            new_hash.update(&swf);
            if existing_hash.finalize() != new_hash.finalize() {
                write_bytes(&output_path, &swf)?;
                Err(anyhow::anyhow!(
                    "Output file '{}' has changed during compilation",
                    self.target
                ))
            } else {
                Ok(())
            }
        } else {
            Ok(write_bytes(&output_path, &swf)?)
        }
    }
}

impl RascalOptions {
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.swf_version.is_none() {
            return Err(anyhow::anyhow!("swf_version is required"));
        }
        if self.scripts.is_empty() && self.classes.is_empty() && self.pcode.is_empty() {
            return Err(anyhow::anyhow!(
                "At least one of scripts, classes or pcode must be specified"
            ));
        }
        Ok(())
    }

    pub fn create_compiler(&self) -> anyhow::Result<Box<dyn SwfCompiler>> {
        Ok(Box::new(RascalCompiler {
            target: self.target.clone(),
            classes: self.classes.clone(),
            scripts: self.scripts.clone(),
            pcode: self.pcode.clone(),
            compile_options: CompileOptions::default().with_swf_version(
                self.swf_version
                    .expect("swf_version is validated elsewhere"),
            ),
            swf_options: SwfOptions::default()
                .with_frame_rate(self.frame_rate)
                .with_stage_size(
                    self.stage_rect.x_min,
                    self.stage_rect.y_min,
                    self.stage_rect.x_max,
                    self.stage_rect.y_max,
                )
                .with_network_sandbox(self.use_network),
            optimizations: OptimizationOptions::none()
                .with_fold_constants(self.optimizations.fold_constants)
                .with_promote_variables_to_registers(
                    self.optimizations.promote_variables_to_registers,
                ),
        }))
    }
}

impl Default for RascalOptions {
    fn default() -> Self {
        Self {
            target: "test.swf".to_string(),
            swf_version: None,
            frame_rate: 24.0,
            scripts: vec![],
            classes: vec![],
            pcode: vec![],
            stage_rect: Default::default(),
            use_network: false,
            optimizations: Default::default(),
        }
    }
}

struct VfsSourceProvider(VfsPath);

impl SourceProvider for VfsSourceProvider {
    fn load(&self, path: &str) -> Result<String, Error> {
        let actual_path = self
            .0
            .join(path)
            .map_err(|e| Error::new(std::io::ErrorKind::NotFound, e))?;
        actual_path.read_to_string().map_err(Error::other)
    }

    fn is_file(&self, path: &str) -> bool {
        self.0.join(path).and_then(|p| p.is_file()).unwrap_or(false)
    }
}
