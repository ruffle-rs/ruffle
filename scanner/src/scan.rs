//! Main/scanner process impls

use crate::analyze::analyze;
use crate::cli_options::ScanOpt;
use crate::file_results::FileResults;
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

pub fn scan_file<P: AsRef<OsStr>>(exec_path: P, file: &DirEntry, name: &str) -> FileResults {
    let start = Instant::now();
    let mut file_results = FileResults::new(name);

    let subproc = Command::new(exec_path)
        .args(["execute-report", &file.path().to_string_lossy()])
        .output();
    match subproc {
        Ok(output) => {
            let mut reader = csv::Reader::from_reader(&output.stdout[..]);
            for row in reader.deserialize::<FileResults>() {
                match row {
                    Ok(child_results) => {
                        let FileResults {
                            name: _name,
                            hash,
                            progress,
                            testing_time,
                            compressed_len,
                            uncompressed_len,
                            error,
                            compression,
                            version,
                            stage_size,
                            frame_rate,
                            num_frames,
                            use_direct_blit,
                            use_gpu,
                            use_network_sandbox,
                            vm_type,
                        } = child_results;

                        file_results.hash = hash;
                        file_results.progress = progress;
                        file_results.testing_time = testing_time;
                        file_results.compressed_len = compressed_len;
                        file_results.uncompressed_len = uncompressed_len;
                        file_results.error = error;
                        file_results.compression = compression;
                        file_results.version = version;
                        file_results.stage_size = stage_size;
                        file_results.frame_rate = frame_rate;
                        file_results.num_frames = num_frames;
                        file_results.use_direct_blit = use_direct_blit;
                        file_results.use_gpu = use_gpu;
                        file_results.use_network_sandbox = use_network_sandbox;
                        file_results.vm_type = vm_type;
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
                        .map(|e| format!("{e}\n{panic_error}"))
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
    let mut writer = csv::Writer::from_path(opt.output_path.clone())?;

    let progress = ProgressBar::new(to_scan.len() as u64);
    progress.set_style(
        ProgressStyle::with_template(
            "[{elapsed_precise}] {bar:40.cyan/blue} [{eta_precise}] {pos:>7}/{len:7} {msg}",
        )
        .unwrap()
        .progress_chars("##-"),
    );

    let result_iter = to_scan
        .into_par_iter()
        .map(move |file| {
            let name = file
                .path()
                .strip_prefix(&opt.input_path)
                .unwrap_or_else(|_| file.path())
                .to_slash_lossy();
            let result = scan_file(&binary_path, &file, &name);

            progress.inc(1);
            progress.set_message(name.into_owned());

            result
        })
        .ser_bridge()
        .inspect(|result| {
            if let Err(e) = writer.serialize(result.clone()) {
                eprintln!("{e}");
            };
        });

    analyze(result_iter);

    Ok(())
}
