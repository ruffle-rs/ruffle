use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use ruffle_core::PlayerBuilder;

#[derive(Parser, Debug)]
#[clap(name = "Ruffle Stub Report Generator", author, version)]
struct Opt {
    /// The file to store the stub report output
    #[clap(name = "output")]
    output_path: PathBuf,
}

fn main() -> Result<()> {
    let opt: Opt = Opt::parse();
    PlayerBuilder::new()
        .with_stub_report_output(opt.output_path)
        .build();
    Ok(())
}
