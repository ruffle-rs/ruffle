use anyhow::{anyhow, Context, Result};
use clap::Parser;
use ruffle_render::pixel_bender::{
    assembly::PixelBenderShaderAssembly, disassembly::PixelBenderShaderDisassembly, parse_shader,
};
use std::io::{IsTerminal, Write};

#[derive(Parser, Debug)]
#[clap(name = "pbasm", author, version)]
pub struct Opt {
    #[clap(name = "source")]
    pub source: String,

    #[clap(short = 'd', long = "disassemble")]
    pub disassemble: bool,

    #[clap(short = 'o', long = "output")]
    pub output: Option<String>,
}

fn disassemble(opt: Opt, write: &mut dyn Write) -> Result<()> {
    let data = std::fs::read(&opt.source).context("Failed to open source file")?;
    let parsed =
        parse_shader(&data, false).map_err(|e| anyhow!("Failed to parse the shader: {e}"))?;
    write!(write, "{}", PixelBenderShaderDisassembly(&parsed))
        .context("Failed to write disassembly")?;
    Ok(())
}

fn assemble(opt: Opt, write: &mut dyn Write) -> Result<()> {
    let input = std::fs::read_to_string(&opt.source).context("Failed to open source file")?;
    let assembly = PixelBenderShaderAssembly::new(&input, write);
    assembly.assemble()?;
    Ok(())
}

pub fn run_main(opt: Opt) -> Result<()> {
    let mut out: Box<dyn Write> = if let Some(output) = opt.output.as_ref().filter(|o| *o != "-") {
        Box::new(std::fs::File::create(output).context("Failed to create the output file")?)
    } else {
        if !opt.disassemble && std::io::stdout().is_terminal() {
            return Err(anyhow!(
                "Cowardly refusing to output binary data to a terminal"
            ));
        }
        Box::new(std::io::stdout())
    };

    if opt.disassemble {
        disassemble(opt, &mut out)?;
    } else {
        assemble(opt, &mut out)?;
    }

    Ok(())
}
