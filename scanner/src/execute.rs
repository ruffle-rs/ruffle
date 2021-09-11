//! Child/executor process impls

use crate::cli_options::ExecuteReportOpt;
use crate::file_results::{AvmType, FileResults, Progress};
use crate::logging::{ScanLogBackend, ThreadLocalScanLogger, LOCAL_LOGGER};
use ruffle_core::backend::audio::NullAudioBackend;
use ruffle_core::backend::locale::NullLocaleBackend;
use ruffle_core::backend::navigator::{NullExecutor, NullNavigatorBackend};
use ruffle_core::backend::render::NullRenderer;
use ruffle_core::backend::storage::MemoryStorageBackend;
use ruffle_core::backend::ui::NullUiBackend;
use ruffle_core::backend::video::NullVideoBackend;
use ruffle_core::swf::{decompress_swf, parse_swf};
use ruffle_core::tag_utils::SwfMovie;
use ruffle_core::Player;
use sha2::{Digest, Sha256};
use swf::{FileAttributes, Tag};

use std::path::Path;

use std::panic::catch_unwind;

use std::io::{stdout, Write};
use std::sync::Arc;
use std::time::{Duration, Instant};

fn execute_swf(file: &Path) {
    let base_path = file.parent().unwrap();
    let (_executor, channel) = NullExecutor::new();
    let movie = SwfMovie::from_path(file, None).unwrap();
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

fn checkpoint<W: Write>(
    file_result: &mut FileResults,
    start: &Instant,
    writer: &mut csv::Writer<W>,
) -> Result<(), std::io::Error> {
    let has_error = file_result.error.is_some();

    file_result.testing_time = start.elapsed().as_millis();
    writer.serialize(file_result).unwrap();

    if has_error {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Error encountered, test terminated",
        ))
    } else {
        Ok(())
    }
}

pub fn execute_report_main(execute_report_opt: ExecuteReportOpt) -> Result<(), std::io::Error> {
    ThreadLocalScanLogger::init();

    let start = Instant::now();
    let file_path = execute_report_opt.input_path;
    let name = file_path
        .file_name()
        .expect("Valid file name in input path")
        .to_string_lossy()
        .into_owned();

    LOCAL_LOGGER.with(|log_buffer| {
        log_buffer.borrow_mut().truncate(0);
    });

    let mut file_result = FileResults {
        progress: Progress::Nothing,
        hash: vec![],
        testing_time: start.elapsed().as_millis(),
        name,
        error: None,
        vm_type: None,
    };
    let stdout = stdout();
    let mut writer = csv::Writer::from_writer(stdout.lock());
    checkpoint(&mut file_result, &start, &mut writer)?;

    let data = match std::fs::read(&file_path) {
        Ok(data) => data,
        Err(e) => {
            file_result.error = Some(format!("File error: {}", e.to_string()));
            checkpoint(&mut file_result, &start, &mut writer)?;

            return Ok(());
        }
    };

    let mut hash = Sha256::new();
    hash.update(&data[..]);

    file_result.hash = hash.finalize().to_vec();
    file_result.progress = Progress::Read;
    checkpoint(&mut file_result, &start, &mut writer)?;

    let swf_buf = match decompress_swf(&data[..]) {
        Ok(swf_buf) => swf_buf,
        Err(e) => {
            file_result.error = Some(e.to_string());
            checkpoint(&mut file_result, &start, &mut writer)?;

            return Ok(());
        }
    };

    file_result.progress = Progress::Decompressed;
    checkpoint(&mut file_result, &start, &mut writer)?;

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
                file_result.error = Some(format!("Parse error: {}", e.to_string()));
                checkpoint(&mut file_result, &start, &mut writer)?;

                return Ok(());
            }
        },
        Err(e) => match e.downcast::<String>() {
            Ok(e) => {
                file_result.error = Some(format!("PANIC: {}", e.to_string()));
                checkpoint(&mut file_result, &start, &mut writer)?;

                return Ok(());
            }
            Err(_) => {
                file_result.error = Some("PANIC".to_string());
                checkpoint(&mut file_result, &start, &mut writer)?;

                return Ok(());
            }
        },
    };

    file_result.vm_type = vm_type;
    file_result.progress = Progress::Parsed;
    checkpoint(&mut file_result, &start, &mut writer)?;

    //Run one frame of the movie in Ruffle.
    if let Err(e) = catch_unwind(|| execute_swf(&file_path)) {
        match e.downcast::<String>() {
            Ok(e) => {
                file_result.error = Some(format!("PANIC: {}", e.to_string()));
                checkpoint(&mut file_result, &start, &mut writer)?;
            }
            Err(_) => {
                file_result.error = Some("PANIC".to_string());
                checkpoint(&mut file_result, &start, &mut writer)?;
            }
        }
    }

    file_result.progress = Progress::Executed;

    let errors = LOCAL_LOGGER.with(|log_buffer| {
        log_buffer.borrow_mut().dedup();

        log_buffer.borrow_mut().join("\n")
    });
    if !errors.is_empty() {
        file_result.error = Some(errors);
        checkpoint(&mut file_result, &start, &mut writer)?;
    }

    file_result.progress = Progress::Completed;
    checkpoint(&mut file_result, &start, &mut writer)?;

    Ok(())
}
