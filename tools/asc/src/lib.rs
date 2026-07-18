use anyhow::anyhow;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::process::Command;

pub struct AscConfig {
    args: Vec<String>,
    custom_main_class: Option<String>,
    working_directory: Option<PathBuf>,
}

impl AscConfig {
    pub fn new<I, A>(args: I) -> Self
    where
        I: IntoIterator<Item = A>,
        A: Into<String>,
    {
        Self {
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
    let asc_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("asc.jar");

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
