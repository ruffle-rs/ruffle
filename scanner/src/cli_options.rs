//! CLI Options

use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap(version, about, author)]
pub struct Opt {
    #[clap(subcommand)]
    pub mode: Mode,
}

#[derive(Parser, Debug)]
pub enum Mode {
    /// Scan an entire directory for SWF files
    Scan(ScanOpt),

    /// Analyze a previously executed scan and compile statistics on it
    Analyze(AnalyzeOpt),

    /// Execute a single SWF file and generate a machine-readable report
    ExecuteReport(ExecuteReportOpt),
}

#[derive(Parser, Debug)]
pub struct ScanOpt {
    /// The directory (containing SWF files) to scan
    #[clap(name = "directory", parse(from_os_str))]
    pub input_path: PathBuf,

    /// The file to store results in CSV format
    #[clap(name = "results", parse(from_os_str))]
    pub output_path: PathBuf,

    /// Filenames to ignore
    #[clap(short = 'i', long = "ignore")]
    pub ignore: Vec<String>,
}

#[derive(Parser, Debug)]
pub struct AnalyzeOpt {
    /// The CSV file to reanalyze
    #[clap(name = "input", parse(from_os_str))]
    pub input_path: PathBuf,
}

#[derive(Parser, Debug)]
pub struct ExecuteReportOpt {
    /// The single SWF file to parse and run
    #[clap(name = "file", parse(from_os_str))]
    pub input_path: PathBuf,
}
