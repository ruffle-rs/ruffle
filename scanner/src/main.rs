use clap::Clap;
use indicatif::{ProgressBar, ProgressStyle};
use log::{Level, LevelFilter, Log, Metadata, Record};
use path_slash::PathExt;
use rayon::prelude::*;
use rayon::ThreadPoolBuilder;
use ruffle_core::backend::audio::NullAudioBackend;
use ruffle_core::backend::locale::NullLocaleBackend;
use ruffle_core::backend::log::LogBackend;
use ruffle_core::backend::navigator::{NullExecutor, NullNavigatorBackend};
use ruffle_core::backend::render::NullRenderer;
use ruffle_core::backend::storage::MemoryStorageBackend;
use ruffle_core::backend::ui::NullUiBackend;
use ruffle_core::backend::video::NullVideoBackend;
use ruffle_core::swf::{decompress_swf, parse_swf};
use ruffle_core::tag_utils::SwfMovie;
use ruffle_core::Player;
use swf::{FileAttributes, Tag};

use serde::Serialize;
use std::path::{Path, PathBuf};

use std::panic::catch_unwind;
use walkdir::{DirEntry, WalkDir};

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Serialize, Debug)]
enum AvmType {
    Avm1,
    Avm2,
}

#[derive(Serialize, Debug)]
enum Progress {
    Nothing,
    Read,
    Decompressed,
    Parsed,
    Executed,
    Completed,
}

#[derive(Serialize, Debug)]
struct FileResults {
    name: String,
    progress: Progress,
    testing_time: u128,
    error: Option<String>,
    vm_type: Option<AvmType>,
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
            progress.set_message(format!("Searching for swf files... {}", results.len()));
        }
    }

    progress.finish_with_message(format!("Found {} swf files to scan", results.len()));
    results
}

/// Log backend that specifically discards AVM trace output
struct ScanLogBackend();

impl ScanLogBackend {
    pub fn new() -> Self {
        Self()
    }
}

impl LogBackend for ScanLogBackend {
    fn avm_trace(&self, message: &str) {}
}

thread_local! {
    /// Thread local log buffer.
    pub static LOCAL_LOGGER: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
}

/// `log` backend (not to be confused with Ruffle's notion of a log backend)
/// that only logs errors to a thread-local area.
struct ThreadLocalScanLogger();

static GLOBAL_LOGGER: ThreadLocalScanLogger = ThreadLocalScanLogger();

impl ThreadLocalScanLogger {
    fn init() {
        log::set_logger(&GLOBAL_LOGGER)
            .map(|()| log::set_max_level(LevelFilter::Info))
            .unwrap();
    }
}

impl Log for ThreadLocalScanLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() == Level::Error
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            LOCAL_LOGGER.with(|log_buffer| {
                log_buffer.borrow_mut().push(format!("{}", record.args()));
            })
        }
    }

    fn flush(&self) {}
}

fn execute_swf(file: DirEntry) {
    let base_path = Path::new(file.path()).parent().unwrap();
    let (mut executor, channel) = NullExecutor::new();
    let movie = SwfMovie::from_path(file.path(), None).unwrap();
    let frame_time = 1000.0 / movie.frame_rate().to_f64();
    let player = Player::new(
        Box::new(NullRenderer::new()),
        Box::new(NullAudioBackend::new()),
        Box::new(NullNavigatorBackend::with_base_path(base_path, channel)),
        Box::new(MemoryStorageBackend::default()),
        Box::new(NullLocaleBackend::new()),
        Box::new(NullVideoBackend::new()),
        Box::new(ScanLogBackend::new()),
        Box::new(NullUiBackend::new()),
    )
    .unwrap();

    player.lock().unwrap().set_root_movie(Arc::new(movie));
    player
        .lock()
        .unwrap()
        .set_max_execution_duration(Duration::from_secs(300));
    player.lock().unwrap().run_frame();
    player.lock().unwrap().update_timers(frame_time);
    //executor.poll_all().unwrap();
}

fn scan_file(file: DirEntry, name: String) -> FileResults {
    let start = Instant::now();

    LOCAL_LOGGER.with(|log_buffer| {
        log_buffer.borrow_mut().truncate(0);
    });

    let mut progress = Progress::Nothing;

    let data = match std::fs::read(file.path()) {
        Ok(data) => data,
        Err(e) => {
            return {
                FileResults {
                    progress,
                    testing_time: start.elapsed().as_millis(),
                    name,
                    error: Some(format!("File error: {}", e.to_string())),
                    vm_type: None,
                }
            }
        }
    };

    progress = Progress::Read;

    let swf_buf = match decompress_swf(&data[..]) {
        Ok(swf_buf) => swf_buf,
        Err(e) => {
            return FileResults {
                progress,
                testing_time: start.elapsed().as_millis(),
                name,
                error: Some(e.to_string()),
                vm_type: None,
            }
        }
    };

    progress = Progress::Decompressed;

    let vm_type = match catch_unwind(|| parse_swf(&swf_buf)) {
        Ok(swf) => match swf {
            Ok(swf) => {
                let mut vm_type = Some(AvmType::Avm1);
                if let Some(Tag::FileAttributes(fa)) = swf.tags.first() {
                    if fa.contains(FileAttributes::IS_ACTION_SCRIPT_3) {
                        vm_type = Some(AvmType::Avm2);
                    }
                }

                vm_type
            }
            Err(e) => {
                return FileResults {
                    progress,
                    testing_time: start.elapsed().as_millis(),
                    name,
                    error: Some(format!("Parse error: {}", e.to_string())),
                    vm_type: None,
                }
            }
        },
        Err(e) => match e.downcast::<String>() {
            Ok(e) => {
                return FileResults {
                    progress,
                    testing_time: start.elapsed().as_millis(),
                    name,
                    error: Some(format!("PANIC: {}", e.to_string())),
                    vm_type: None,
                }
            }
            Err(_) => {
                return FileResults {
                    progress,
                    testing_time: start.elapsed().as_millis(),
                    name,
                    error: Some("PANIC".to_string()),
                    vm_type: None,
                }
            }
        },
    };

    progress = Progress::Parsed;

    //Run one frame of the movie in Ruffle.
    if let Err(e) = catch_unwind(|| execute_swf(file)) {
        match e.downcast::<String>() {
            Ok(e) => {
                return FileResults {
                    progress,
                    testing_time: start.elapsed().as_millis(),
                    name,
                    error: Some(format!("PANIC: {}", e.to_string())),
                    vm_type: None,
                }
            }
            Err(_) => {
                return FileResults {
                    progress,
                    testing_time: start.elapsed().as_millis(),
                    name,
                    error: Some("PANIC".to_string()),
                    vm_type: None,
                }
            }
        }
    }

    progress = Progress::Executed;

    let errors = LOCAL_LOGGER.with(|log_buffer| log_buffer.borrow_mut().join("\n"));
    if !errors.is_empty() {
        return FileResults {
            progress,
            testing_time: start.elapsed().as_millis(),
            name,
            error: Some(errors),
            vm_type,
        };
    }

    progress = Progress::Completed;

    FileResults {
        progress,
        testing_time: start.elapsed().as_millis(),
        name,
        error: None,
        vm_type,
    }
}

/// Parallel-to-serial iterator bridge trait
///
/// Proposed in and copied from https://github.com/rayon-rs/rayon/issues/858
pub trait SerBridge<T>
where
    T: Send + 'static,
    Self: ParallelIterator<Item = T> + 'static,
{
    fn ser_bridge(self) -> SerBridgeImpl<T> {
        SerBridgeImpl::new(self)
    }
}

impl<PI, T> SerBridge<T> for PI
where
    T: Send + 'static,
    PI: ParallelIterator<Item = T> + 'static,
{
}

/// Parallel-to-serial iterator bridge
///
/// Proposed in and copied from https://github.com/rayon-rs/rayon/issues/858
pub struct SerBridgeImpl<T> {
    rx: crossbeam_channel::Receiver<T>,
}

impl<T: Send + 'static> SerBridgeImpl<T> {
    pub fn new<PI>(par_iterable: impl IntoParallelIterator<Item = T, Iter = PI>) -> Self
    where
        PI: ParallelIterator<Item = T> + 'static,
    {
        let par_iter = par_iterable.into_par_iter();
        let (tx, rx) = crossbeam_channel::bounded(0);
        std::thread::spawn(move || {
            let _ = par_iter.try_for_each(|item| tx.send(item));
        });
        SerBridgeImpl { rx }
    }
}

impl<T> Iterator for SerBridgeImpl<T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        self.rx.recv().ok()
    }
}

fn main() -> Result<(), std::io::Error> {
    ThreadLocalScanLogger::init();

    ThreadPoolBuilder::new()
        .stack_size(16 * 1024 * 1024)
        .build()
        .unwrap()
        .install(|| {
            let opt = Opt::parse();
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
                    let result = scan_file(file, name.clone());

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
        })
}
