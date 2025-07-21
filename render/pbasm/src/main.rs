use anyhow::Result;
use clap::Parser;
use pbasm::{run_main, Opt};

fn main() -> Result<()> {
    let opt: Opt = Opt::parse();
    run_main(opt)
}
