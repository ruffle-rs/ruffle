//! Post-scan analysis

use crate::cli_options::AnalyzeOpt;
use crate::file_results::{FileResults, Step};
use std::fs::File;

/// Generate and print statistics related to a scan's results
pub fn analyze(results: impl Iterator<Item = FileResults>) {
    let mut total = 0;
    let mut start = 0;
    let mut read = 0;
    let mut decompress = 0;
    let mut parse = 0;
    let mut execute = 0;
    let mut complete = 0;

    for result in results {
        total += 1;

        match result.progress {
            Step::Start => start += 1,
            Step::Read => read += 1,
            Step::Decompress => decompress += 1,
            Step::Parse => parse += 1,
            Step::Execute => execute += 1,
            Step::Complete => complete += 1,
        }
    }

    println!("Scanned {total} swf files.");

    let digits = [start, read, decompress, parse, execute, complete]
        .iter()
        .map(|x| (*x as f64).log10().ceil() as usize)
        .max()
        .unwrap()
        + 4;

    println!();

    if start > 0 {
        println!("{start:>digits$} movies panicked or crashed the scanner");
    }

    println!("{read:>digits$} movies failed when reading");
    println!("{decompress:>digits$} movies failed to decompress");
    println!("{parse:>digits$} movies failed to parse");
    println!("{execute:>digits$} movies failed to execute");
    println!("{complete:>digits$} movies completed without errors");
    println!();
}

pub fn analyze_main(opt: AnalyzeOpt) -> Result<(), std::io::Error> {
    let file = File::open(opt.input_path)?;
    let reader = csv::Reader::from_reader(file);

    analyze(reader.into_deserialize::<FileResults>().map(|r| {
        match r {
            Ok(fr) => fr,
            Err(e) => {
                // Treat unparsable CSV rows as a scanner panic
                FileResults {
                    error: Some(format!("{e}")),
                    ..FileResults::default()
                }
            }
        }
    }));

    Ok(())
}
