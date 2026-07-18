use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use ruffle_core::PlayerBuilder;

#[derive(Parser, Debug)]
#[clap(name = "Ruffle Stub Report Generator", author, version)]
struct Opt {
    /// The file to store the stub report output.
    #[clap(long)]
    stub_report: Option<PathBuf>,

    /// The file to store the AVM2 report output.
    #[clap(long)]
    avm2_report: Option<PathBuf>,
}

fn main() -> Result<()> {
    let opt: Opt = Opt::parse();

    let _tempfile;
    let stub_report = match opt.stub_report {
        Some(path) => path,
        None => {
            _tempfile = tempfile::NamedTempFile::new()?;
            _tempfile.path().to_path_buf()
        }
    };

    PlayerBuilder::new()
        .with_stub_report_output(stub_report.clone())
        .build();

    if let Some(avm2_report) = opt.avm2_report {
        let spec = ruffle_api_report::default_spec()?;
        let implementation = ruffle_api_report::read_spec(&stub_report)?;
        let report = ruffle_api_report::generate_report(&spec, &implementation);
        report.write(&avm2_report)?;
    }

    Ok(())
}
