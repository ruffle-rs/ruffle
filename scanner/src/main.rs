use clap::Clap;
use indicatif::{ProgressBar, ProgressStyle};
use path_slash::PathExt;
use ruffle_core::swf::{read_swf, read_swf_header};

use serde::Serialize;
use std::path::{Path, PathBuf};

use std::panic::catch_unwind;
use walkdir::{DirEntry, WalkDir};

#[derive(Serialize, Debug)]
struct FileResults {
    name: String,
    error: Option<String>,
}

#[derive(Clap, Debug)]
#[clap(version, about, author)]
struct Opt {
    /// The directory (containing SWF files) to scan
    #[clap(name = "directory", parse(from_os_str))]
    input_path: PathBuf,

    /// The file to store results in CSV format
    #[clap(name = "results", parse(from_os_str))]
    output_path: PathBuf,

    /// Filenames to ignore
    #[clap(short = 'i', long = "ignore")]
    ignore: Vec<String>,
}

fn find_files(root: &Path, ignore: &[String]) -> Vec<DirEntry> {
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
            progress.set_message(&format!("Searching for swf files... {}", results.len()));
        }
    }

    progress.finish_with_message(&format!("Found {} swf files to scan", results.len()));
    results
}

fn scan_file(file: DirEntry, name: String) -> FileResults {
    let data = match std::fs::read(file.path()) {
        Ok(data) => data,
        Err(e) => {
            return {
                FileResults {
                    name,
                    error: Some(format!("File error: {}", e.to_string())),
                }
            }
        }
    };

    let swf_stream = read_swf_header(&data[..]).unwrap();
    match catch_unwind(|| read_swf(&swf_stream)) {
        Ok(swf) => match swf {
            Ok(_swf) => FileResults { name, error: None },
            Err(e) => FileResults {
                name,
                error: Some(format!("Parse error: {}", e.to_string())),
            },
        },
        Err(e) => match e.downcast::<String>() {
            Ok(e) => FileResults {
                name,
                error: Some(format!("PANIC: {}", e.to_string())),
            },
            Err(_) => FileResults {
                name,
                error: Some("PANIC".to_string()),
            },
        },
    }
}

fn main() -> Result<(), std::io::Error> {
    env_logger::init();

    let opt = Opt::parse();
    let to_scan = find_files(&opt.input_path, &opt.ignore);
    let total = to_scan.len() as u64;
    let mut good = 0;
    let mut bad = 0;
    let progress = ProgressBar::new(total);
    let mut writer = csv::Writer::from_path(opt.output_path)?;

    progress.set_style(
        ProgressStyle::default_bar()
            .template(
                "[{elapsed_precise}] {bar:40.cyan/blue} [{eta_precise}] {pos:>7}/{len:7} {msg}",
            )
            .progress_chars("##-"),
    );

    writer.write_record(&["Filename", "Error"])?;

    for file in to_scan {
        let name = file
            .path()
            .strip_prefix(&opt.input_path)
            .unwrap_or_else(|_| file.path())
            .to_slash_lossy();
        progress.inc(1);
        progress.set_message(&name);
        let result = scan_file(file, name);

        if result.error.is_none() {
            good += 1;
        } else {
            bad += 1;
        }

        writer.serialize(result)?;
    }

    progress.finish_with_message(&format!(
        "Scanned {} swf files. {} successfully parsed, {} encountered errors",
        total, good, bad
    ));

    Ok(())
}
