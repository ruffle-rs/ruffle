use crate::compiler::SwfCompiler;
use crate::util::{read_bytes, write_bytes};
use anyhow::anyhow;
use ruffle_core::swf::{
    Compression, DoAbc2, DoAbc2Flag, FileAttributes, Fixed8, Header, Rectangle, SwfStr,
    SymbolClassLink, Tag, Twips,
};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use tempfile::TempDir;
use vfs::VfsPath;

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct StageTransform {
    pub x: f64,
    pub y: f64,
}

impl Default for StageTransform {
    fn default() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
}

#[derive(Clone, Deserialize, Debug)]
#[serde(default, deny_unknown_fields)]
pub struct AscOptions {
    pub target: String,
    pub class: Option<String>,
    pub scripts: Vec<String>,
    pub swf_version: Option<u8>,
    pub stage_transform: StageTransform,
    pub use_network: Option<bool>,
    pub optimize: Option<bool>,
    pub debug: Option<bool>,
}

impl AscOptions {
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.swf_version.is_none() {
            return Err(anyhow::anyhow!("Missing swf_version"));
        }
        if self.class.is_none() {
            return Err(anyhow::anyhow!("Missing class"));
        }
        if self.scripts.is_empty() {
            return Err(anyhow::anyhow!("Missing scripts"));
        }
        Ok(())
    }

    pub fn create_compiler(&self) -> anyhow::Result<Box<dyn SwfCompiler>> {
        Ok(Box::new(AscCompiler {
            target: self.target.clone(),
            class: self.class.as_ref().unwrap().to_string(),
            scripts: self.scripts.clone(),
            swf_version: self.swf_version.unwrap(),
            stage_transform: self.stage_transform,
            use_network: self.use_network.unwrap_or(false),
            optimize: self.optimize.unwrap_or(true),
            debug: self.debug.unwrap_or(true),
        }))
    }
}

impl Default for AscOptions {
    fn default() -> Self {
        Self {
            target: "test.swf".to_string(),
            class: None,
            scripts: vec![],
            swf_version: None,
            stage_transform: Default::default(),
            use_network: None,
            optimize: None,
            debug: None,
        }
    }
}

#[derive(Debug)]
pub struct AscCompiler {
    target: String,
    class: String,
    scripts: Vec<String>,
    swf_version: u8,
    stage_transform: StageTransform,
    use_network: bool,
    optimize: bool,
    debug: bool,
}

impl SwfCompiler for AscCompiler {
    fn compile(self: Box<Self>, root_dir: &VfsPath, verify_if_changed: bool) -> anyhow::Result<()> {
        let tmp_dir = TempDir::new().unwrap();
        let tmp_dir = tmp_dir.as_ref();

        let mut java_args = vec!["-AS3", "-md", "-import", ruffle_core::PLAYERGLOBAL_ABC_PATH];

        if self.optimize {
            java_args.push("-optimize");
        }

        if self.debug {
            java_args.push("-d");
        }

        let class_script = format!("{}.as", self.class);

        for script in &self.scripts {
            let src = root_dir.join(script)?;
            let dest = tmp_dir.join(script);
            if let Some(parent) = dest.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&dest, read_bytes(&src)?)?;

            if script != &class_script {
                java_args.push("-in");
                java_args.push(script);
            }
        }

        java_args.push(&class_script);

        let mut asc_config = asc::AscConfig::new(java_args);
        asc_config.working_directory(tmp_dir);
        asc::run_asc(asc_config)?;

        let abc_bytes = std::fs::read(tmp_dir.join(format!("{}.abc", self.class)))?;

        let (width, height, fps) = read_swf_metadata(&abc_bytes, &self.class)?;
        let StageTransform { x, y } = self.stage_transform;

        let header = Header {
            compression: Compression::None,
            version: self.swf_version,
            stage_size: Rectangle {
                x_min: Twips::from_pixels(x),
                x_max: Twips::from_pixels(x + width),
                y_min: Twips::from_pixels(y),
                y_max: Twips::from_pixels(y + height),
            },
            frame_rate: Fixed8::from_f32(fps),
            num_frames: 1,
        };

        let mut attributes = FileAttributes::IS_ACTION_SCRIPT_3;
        if self.use_network {
            attributes.set(FileAttributes::USE_NETWORK_SANDBOX, true);
        }

        let tags = [
            Tag::FileAttributes(attributes),
            Tag::DoAbc2(DoAbc2 {
                flags: DoAbc2Flag::LAZY_INITIALIZE,
                name: SwfStr::from_utf8_str(""),
                data: &abc_bytes,
            }),
            Tag::SymbolClass(vec![SymbolClassLink {
                id: 0,
                class_name: SwfStr::from_utf8_str(&self.class),
            }]),
            Tag::ShowFrame,
        ];

        let mut swf_bytes = Vec::new();
        ruffle_core::swf::write_swf(&header, &tags, &mut swf_bytes)?;

        let output_path = root_dir.join(&self.target)?;
        if verify_if_changed {
            if !output_path.is_file()? {
                write_bytes(&output_path, &swf_bytes)?;
                return Err(anyhow::anyhow!(
                    "Output file '{}' does not exist or is not a file",
                    self.target
                ));
            }

            let mut existing_hash = Sha256::new();
            existing_hash.update(read_bytes(&output_path)?);

            let mut new_hash = Sha256::new();
            new_hash.update(&swf_bytes);

            if existing_hash.finalize() != new_hash.finalize() {
                write_bytes(&output_path, &swf_bytes)?;
                Err(anyhow::anyhow!(
                    "Output file '{}' has changed during compilation",
                    self.target
                ))
            } else {
                Ok(())
            }
        } else {
            Ok(write_bytes(&output_path, &swf_bytes)?)
        }
    }
}

fn read_swf_metadata(abc_bytes: &[u8], class: &str) -> anyhow::Result<(f64, f64, f32)> {
    use ruffle_core::swf::avm2::read::Reader;
    use ruffle_core::swf::avm2::types::Multiname;

    let abc = Reader::new(abc_bytes).read().unwrap();
    let strings = &abc.constant_pool.strings;

    let mut width: f64 = 550.0;
    let mut height: f64 = 400.0;
    let mut fps: f32 = 24.0;

    let Some(class_trait) = abc
        .scripts
        .iter()
        .flat_map(|s| &s.traits)
        .filter(|t| t.name.0 != 0)
        .find(|t| {
            matches!(
                abc.constant_pool.multinames[t.name.0 as usize - 1],
                Multiname::QName { name, .. }
                if std::str::from_utf8(&strings[name.0 as usize - 1]).unwrap_or("") == class
            )
        })
    else {
        return Ok((width, height, fps));
    };

    let Some(metadata) = class_trait
        .metadata
        .iter()
        .map(|&idx| &abc.metadata[idx.0 as usize])
        .find(|m| m.name.0 != 0 && strings[m.name.0 as usize - 1] == b"SWF")
    else {
        return Ok((width, height, fps));
    };

    for item in &metadata.items {
        if item.key.0 == 0 || item.value.0 == 0 {
            continue;
        }
        let key = &strings[item.key.0 as usize - 1];
        let value = std::str::from_utf8(&strings[item.value.0 as usize - 1]).unwrap();

        match key.as_slice() {
            b"width" => width = value.parse()?,
            b"height" => height = value.parse()?,
            b"frameRate" => fps = value.parse()?,
            key => {
                return Err(anyhow!(
                    "Unknown SWF annotation key: {}",
                    String::from_utf8_lossy(key)
                ));
            }
        }
    }

    Ok((width, height, fps))
}
