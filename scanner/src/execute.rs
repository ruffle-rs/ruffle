//! Child/executor process impls

use crate::cli_options::ExecuteReportOpt;
use crate::file_results::{AvmType, FileResults, Step};
use crate::logging::{ScanLogBackend, ThreadLocalScanLogger, LOCAL_LOGGER};
use ruffle_core::backend::navigator::{NullExecutor, NullNavigatorBackend};
use ruffle_core::limits::ExecutionLimit;
use ruffle_core::swf::{decompress_swf, parse_swf};
use ruffle_core::tag_utils::SwfMovie;
use ruffle_core::PlayerBuilder;
use sha2::{Digest, Sha256};
use std::io::{stdout, Write};
use std::panic::catch_unwind;
use std::path::Path;
use std::time::{Duration, Instant};

fn execute_swf(file: &Path) {
    let base_path = file.parent().unwrap();
    let executor = NullExecutor::new();
    let movie = SwfMovie::from_path(file, None).unwrap();
    let frame_time = 1000.0 / movie.frame_rate().to_f64();
    let player = PlayerBuilder::new()
        .with_log(ScanLogBackend::new())
        .with_navigator(NullNavigatorBackend::with_base_path(base_path, &executor))
        .with_max_execution_duration(Duration::from_secs(300))
        .with_movie(movie)
        .build();

    player.lock().unwrap().preload(&mut ExecutionLimit::none());

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

    let mut file_result = FileResults::new(&name);

    let stdout = stdout();
    let mut writer = csv::Writer::from_writer(stdout.lock());
    checkpoint(&mut file_result, &start, &mut writer)?;

    file_result.progress = Step::Read;

    let data = match std::fs::read(&file_path) {
        Ok(data) => data,
        Err(e) => {
            file_result.error = Some(format!("File error: {e}"));
            checkpoint(&mut file_result, &start, &mut writer)?;

            return Ok(());
        }
    };

    file_result.compressed_len = Some(data.len());

    let mut hash = Sha256::new();
    hash.update(&data[..]);

    file_result.hash = hash.finalize().to_vec();
    checkpoint(&mut file_result, &start, &mut writer)?;

    file_result.progress = Step::Decompress;

    let swf_buf = match decompress_swf(&data[..]) {
        Ok(swf_buf) => swf_buf,
        Err(e) => {
            file_result.error = Some(e.to_string());
            checkpoint(&mut file_result, &start, &mut writer)?;

            return Ok(());
        }
    };

    checkpoint(&mut file_result, &start, &mut writer)?;
    file_result.progress = Step::Parse;

    match catch_unwind(|| parse_swf(&swf_buf)) {
        Ok(swf) => match swf {
            Ok(swf) => {
                let stage_size = swf.header.stage_size();
                let stage_width = (stage_size.x_max - stage_size.x_min).to_pixels();
                let stage_height = (stage_size.y_max - stage_size.y_min).to_pixels();

                file_result.uncompressed_len = Some(swf.header.uncompressed_len());
                file_result.compression = Some(swf.header.compression().into());
                file_result.version = Some(swf.header.version());
                file_result.stage_size = Some(format!("{stage_width}x{stage_height}"));
                file_result.frame_rate = Some(swf.header.frame_rate().into());
                file_result.num_frames = Some(swf.header.num_frames());
                file_result.use_direct_blit = Some(swf.header.use_direct_blit());
                file_result.use_gpu = Some(swf.header.use_gpu());
                file_result.use_network_sandbox = Some(swf.header.use_network_sandbox());
                file_result.vm_type = Some(match swf.header.is_action_script_3() {
                    true => AvmType::Avm2,
                    false => AvmType::Avm1,
                });
            }
            Err(e) => {
                file_result.error = Some(format!("Parse error: {e}"));
                checkpoint(&mut file_result, &start, &mut writer)?;
            }
        },
        Err(e) => match e.downcast::<String>() {
            Ok(e) => {
                file_result.error = Some(format!("PANIC: {e}"));
                checkpoint(&mut file_result, &start, &mut writer)?;
            }
            Err(_) => {
                file_result.error = Some("PANIC".to_string());
                checkpoint(&mut file_result, &start, &mut writer)?;
            }
        },
    };

    checkpoint(&mut file_result, &start, &mut writer)?;
    file_result.progress = Step::Execute;

    //Run one frame of the movie in Ruffle.
    if let Err(e) = catch_unwind(|| execute_swf(&file_path)) {
        match e.downcast::<String>() {
            Ok(e) => {
                file_result.error = Some(format!("PANIC: {e}"));
                checkpoint(&mut file_result, &start, &mut writer)?;
            }
            Err(_) => {
                file_result.error = Some("PANIC".to_string());
                checkpoint(&mut file_result, &start, &mut writer)?;
            }
        }
    }

    let errors = LOCAL_LOGGER.with(|log_buffer| {
        log_buffer.borrow_mut().dedup();

        log_buffer.borrow_mut().join("\n")
    });
    if !errors.is_empty() {
        file_result.error = Some(errors);
    } else {
        file_result.progress = Step::Complete;
    }

    checkpoint(&mut file_result, &start, &mut writer)?;

    Ok(())
}
