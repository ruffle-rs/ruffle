//! Main/scanner process impls

use crate::cli_options::ScanOpt;
use crate::file_results::{FileResults, Progress};
use crate::ser_bridge::SerBridge;
use indicatif::{ProgressBar, ProgressStyle};
use path_slash::PathExt;
use rayon::prelude::*;

use std::path::Path;

use walkdir::{DirEntry, WalkDir};

use std::env;
use std::ffi::OsStr;
use std::process::Command;
use std::time::Instant;

pub fn find_files(root: &Path, ignore: &[String]) -> Vec<DirEntry> {
    let progress = ProgressBar::new_spinner();
    let mut results = Vec::new();

    for entry in WalkDir::new(root)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let f_name = entry.file_name().to_string_lossy();

        if f_name.ends_with(".swf") && !ignore.iter().any(|x| x == &f_name) {
            results.push(entry);
            progress.set_message(format!("Searching for swf files... {}", results.len()));
        }
    }

    progress.finish_with_message(format!("Found {} swf files to scan", results.len()));
    results
}

pub fn scan_file<P: AsRef<OsStr>>(exec_path: P, file: DirEntry, name: String) -> FileResults {
    let start = Instant::now();
    let mut file_results = FileResults {
        name,
        hash: vec![],
        progress: Progress::Nothing,
        testing_time: start.elapsed().as_millis(),
        error: None,
        vm_type: None,
    };

    let subproc = Command::new(exec_path)
        .args(&[
            "execute-report",
            &file.path().to_string_lossy().into_owned(),
        ])
        .output();
    match subproc {
        Ok(output) => {
            let mut reader = csv::Reader::from_reader(&output.stdout[..]);
            for row in reader.deserialize::<FileResults>() {
                match row {
                    Ok(child_results) => {
                        file_results.progress = child_results.progress;
                        file_results.error = child_results.error;
                        file_results.vm_type = child_results.vm_type;
                    }
                    Err(e) => {
                        file_results.error = Some(e.to_string());
                    }
                }
            }

            if !output.stderr.is_empty() {
                let panic_error = String::from_utf8_lossy(&output.stderr).into_owned();
                file_results.error = Some(
                    file_results
                        .error
                        .map(|e| format!("{}\n{}", e, panic_error))
                        .unwrap_or(panic_error),
                );
            }
        }
        Err(e) => file_results.error = Some(e.to_string()),
    }

    file_results.testing_time = start.elapsed().as_millis();

    file_results
}

/// The main scanner process.
///
/// Should be called with parsed options corresponding to the `scan` command.
pub fn scan_main(opt: ScanOpt) -> Result<(), std::io::Error> {
    let binary_path = env::current_exe()?;
    let to_scan = find_files(&opt.input_path, &opt.ignore);
    let total = to_scan.len() as u64;
    let mut good = 0;
    let mut bad = 0;
    let progress = ProgressBar::new(total);
    let mut writer = csv::Writer::from_path(opt.output_path.clone())?;

    progress.set_style(
        ProgressStyle::default_bar()
            .template(
                "[{elapsed_precise}] {bar:40.cyan/blue} [{eta_precise}] {pos:>7}/{len:7} {msg}",
            )
            .progress_chars("##-"),
    );

    writer.write_record(&[
        "Filename",
        "SHA256 Hash",
        "Progress",
        "Test Duration",
        "Error",
        "AVM Version",
    ])?;

    let input_path = opt.input_path;
    let closure_progress = progress.clone();

    let result_iter = to_scan
        .into_par_iter()
        .map(move |file| {
            let name = file
                .path()
                .strip_prefix(&input_path)
                .unwrap_or_else(|_| file.path())
                .to_slash_lossy();
            let result = scan_file(&binary_path, file, name.clone());

            closure_progress.inc(1);
            closure_progress.set_message(name);

            result
        })
        .ser_bridge();

    for result in result_iter {
        if result.error.is_none() {
            good += 1;
        } else {
            bad += 1;
        }

        writer.serialize(result)?;
    }

    progress.finish_with_message(format!(
        "Scanned {} swf files. {} successfully parsed, {} encountered errors",
        total, good, bad
    ));

    Ok(())
}
