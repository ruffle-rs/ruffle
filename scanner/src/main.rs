use crate::analyze::analyze_main;
use crate::cli_options::{Mode, Opt};
use crate::execute::execute_report_main;
use crate::scan::scan_main;
use clap::Clap;

mod analyze;
mod cli_options;
mod execute;
mod file_results;
mod logging;
mod scan;
mod ser_bridge;

fn main() -> Result<(), std::io::Error> {
    let opt = Opt::parse();

    match opt.mode {
        Mode::Scan(scan_opt) => scan_main(scan_opt),
        Mode::Analyze(analyze_opt) => analyze_main(analyze_opt),
        Mode::ExecuteReport(exeute_report_opt) => {
            if execute_report_main(exeute_report_opt).is_err() {
                // Do nothing.
            }

            // Do NOT report errors in this function so it doesn't pollute the
            // CSV output.
            Ok(())
        }
    }
}
