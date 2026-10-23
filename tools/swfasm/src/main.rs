mod cli;

use anyhow::{Context, Result, bail};
use clap::Parser;
use cli::Opt;

fn main() -> Result<()> {
    let opt = Opt::parse();

    let text = std::fs::read_to_string(&opt.input)
        .with_context(|| format!("Failed to read {}", opt.input.display()))?;
    let swf = swfasm::compile_str(&text)
        .with_context(|| format!("Failed to compile {}", opt.input.display()))?;

    let output_path = opt
        .output
        .unwrap_or_else(|| opt.input.with_extension("swf"));
    if output_path == opt.input {
        bail!(
            "Refusing to overwrite the input file {}, use --output",
            opt.input.display()
        );
    }
    std::fs::write(&output_path, swf)
        .with_context(|| format!("Failed to write {}", output_path.display()))?;
    Ok(())
}
