use anyhow::Result;
use clap::Parser;
use exporter::{cli::Opt, run_main};

fn main() -> Result<()> {
    let opt: Opt = Opt::parse();
    run_main(opt)
}
