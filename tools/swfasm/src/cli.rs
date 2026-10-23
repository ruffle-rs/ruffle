use clap::Parser;
use std::path::PathBuf;

/// Compiles a textual (RON) representation of a SWF file into a SWF file.
#[derive(Parser, Debug)]
#[clap(name = "swfasm", author, version)]
pub struct Opt {
    /// The source file.
    pub input: PathBuf,

    /// The output file.
    ///
    /// Defaults to the input file with the `.swf` extension.
    #[clap(short, long)]
    pub output: Option<PathBuf>,
}
