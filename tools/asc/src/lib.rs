use anyhow::anyhow;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct AscConfig {
    asc_jar: PathBuf,
    args: Vec<String>,
    custom_main_class: Option<String>,
    working_directory: Option<PathBuf>,
}

impl AscConfig {
    /// Creates a config that runs the `asc.jar` found in `repo_root`, the
    /// root of the Ruffle repository.
    ///
    /// The jar is located from a path supplied at runtime rather than from
    /// this crate's own `CARGO_MANIFEST_DIR`: a compile-time path would be
    /// baked into the binary, and could point into a different checkout
    /// (such as a since-deleted worktree) that shared the same target dir.
    pub fn new<I, A>(repo_root: &Path, args: I) -> Self
    where
        I: IntoIterator<Item = A>,
        A: Into<String>,
    {
        Self {
            asc_jar: repo_root.join("tools/asc/asc.jar"),
            args: args.into_iter().map(|a| a.into()).collect(),
            custom_main_class: None,
            working_directory: None,
        }
    }

    pub fn custom_main_class<S: Into<String>>(&mut self, custom_main_class: S) -> &mut Self {
        self.custom_main_class = Some(custom_main_class.into());
        self
    }

    pub fn working_directory<P: Into<PathBuf>>(&mut self, working_directory: P) -> &mut Self {
        self.working_directory = Some(working_directory.into());
        self
    }
}

pub fn run_asc(config: AscConfig) -> anyhow::Result<()> {
    // Make the path absolute, since `working_directory` may change
    // the directory `java` resolves it against.
    let asc_path = std::path::absolute(&config.asc_jar)?;
    if !asc_path.is_file() {
        return Err(anyhow!("asc.jar not found at {}", asc_path.display()));
    }

    let mut cli_args: Vec<String> = vec![
        "-classpath".to_string(),
        asc_path.to_string_lossy().into_owned(),
        config
            .custom_main_class
            .unwrap_or_else(|| "macromedia.asc.embedding.Main".to_string()),
    ];

    cli_args.extend(config.args);

    let mut command = Command::new("java");

    command.args(cli_args);

    if let Some(current_dir) = config.working_directory {
        command.current_dir(current_dir);
    }

    let status = command.status();

    match status {
        Ok(code) => {
            if !code.success() {
                return Err(anyhow!("Compiling failed with code {code:?}"));
            }
        }
        Err(err) => {
            if err.kind() == ErrorKind::NotFound {
                return Err(anyhow!(
                    "Java could not be found on your computer. Please install java, then try compiling again."
                ));
            }
            return Err(err.into());
        }
    }

    Ok(())
}
