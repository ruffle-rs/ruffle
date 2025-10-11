pub mod cli;
mod exporter;
mod player_ext;
mod progress;

use anyhow::{anyhow, Result};
use image::RgbaImage;
use indicatif::ProgressBar;
use rayon::prelude::*;
use std::fs::create_dir_all;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use walkdir::{DirEntry, WalkDir};

use crate::cli::{FrameSelection, Opt};
use crate::exporter::Exporter;
use crate::progress::ExporterProgress;

/// Captures a screenshot. The resulting image uses straight alpha
fn take_screenshot(
    exporter: &Exporter,
    swf_path: &Path,
    frames: FrameSelection, // TODO Figure out a way to get framecount before calling take_screenshot, so that we can have accurate progress bars when using --frames all
    skipframes: u32,
    progress: &ExporterProgress,
) -> Result<Vec<RgbaImage>> {
    let movie_export = exporter.start_exporting_movie(swf_path)?;

    let mut result = Vec::new();
    let totalframes = movie_export.total_frames();

    for i in 0..totalframes {
        progress.set_message(format!(
            "{} frame {}",
            swf_path.file_stem().unwrap().to_string_lossy(),
            i
        ));

        movie_export.run_frame();

        if i >= skipframes {
            match movie_export.capture_frame() {
                Ok(image) => result.push(image),
                Err(e) => {
                    return Err(anyhow!(
                        "Unable to capture frame {} of {:?}: {:?}",
                        i,
                        swf_path,
                        e
                    ))
                }
            }
        }

        if !matches!(frames, FrameSelection::All) {
            progress.inc(1);
        }
    }
    Ok(result)
}

fn find_files(root: &Path, with_progress: bool) -> Vec<DirEntry> {
    let progress = if with_progress {
        Some(ProgressBar::new_spinner())
    } else {
        None
    };
    let mut results = Vec::new();

    for entry in WalkDir::new(root)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let f_name = entry.file_name().to_string_lossy();

        if f_name.ends_with(".swf") {
            results.push(entry);
            if let Some(progress) = &progress {
                progress.set_message(format!("Searching for swf files... {}", results.len()));
            }
        }
    }

    if let Some(progress) = &progress {
        progress.finish_with_message(format!("Found {} swf files to export", results.len()));
    }

    results
}

fn capture_single_swf(exporter: &Exporter, opt: &Opt) -> Result<()> {
    let is_single_frame = opt.frames.is_single_frame();
    let output = opt.output_path.clone().unwrap_or_else(|| {
        let mut result = PathBuf::new();
        result.set_file_name(opt.swf.file_stem().unwrap());
        if is_single_frame {
            result.set_extension("png");
        }
        result
    });

    if !is_single_frame {
        let _ = create_dir_all(&output);
    }

    let progress = ExporterProgress::new(opt, 1);

    let frames = take_screenshot(exporter, &opt.swf, opt.frames, opt.skipframes, &progress)?;

    progress.set_message(opt.swf.file_stem().unwrap().to_string_lossy().into_owned());

    if is_single_frame {
        let image = frames.first().unwrap();
        if opt.output_path == Some(PathBuf::from("-")) {
            let mut bytes: Vec<u8> = Vec::new();
            image
                .write_to(&mut io::Cursor::new(&mut bytes), image::ImageFormat::Png)
                .expect("Encoding failed");
            io::stdout()
                .write_all(bytes.as_slice())
                .expect("Writing to stdout failed");
        } else {
            image.save(&output)?;
        }
    } else {
        let digits = frames.len().to_string().len();
        for (frame, image) in frames.iter().enumerate() {
            let mut path: PathBuf = (&output).into();
            path.push(format!("{frame:0digits$}.png"));
            image.save(&path)?;
        }
    }

    let message = if frames.len() == 1 {
        if !opt.silent {
            Some(format!(
                "Saved first frame of {} to {}",
                opt.swf.to_string_lossy(),
                output.to_string_lossy()
            ))
        } else {
            None
        }
    } else {
        Some(format!(
            "Saved first {} frames of {} to {}",
            frames.len(),
            opt.swf.to_string_lossy(),
            output.to_string_lossy()
        ))
    };

    if let Some(message) = message {
        progress.finish_with_message(message);
    }

    Ok(())
}

fn capture_multiple_swfs(exporter: &Exporter, opt: &Opt) -> Result<()> {
    let output = opt.output_path.clone().unwrap();
    let files = find_files(&opt.swf, !opt.silent);

    let progress = ExporterProgress::new(opt, files.len() as u64);

    files.par_iter().try_for_each(|file| -> Result<()> {
        progress.set_message(
            file.path()
                .file_stem()
                .unwrap()
                .to_string_lossy()
                .into_owned(),
        );
        if let Ok(frames) =
            take_screenshot(exporter, file.path(), opt.frames, opt.skipframes, &progress)
        {
            let mut relative_path = file
                .path()
                .strip_prefix(&opt.swf)
                .unwrap_or_else(|_| file.path())
                .to_path_buf();

            if frames.len() == 1 {
                let mut destination: PathBuf = (&output).into();
                relative_path.set_extension("png");
                destination.push(relative_path);
                if let Some(parent) = destination.parent() {
                    let _ = create_dir_all(parent);
                }
                frames.first().unwrap().save(&destination)?;
            } else {
                let mut parent: PathBuf = (&output).into();
                relative_path.set_extension("");
                parent.push(&relative_path);
                let _ = create_dir_all(&parent);
                let digits = frames.len().to_string().len();
                for (frame, image) in frames.iter().enumerate() {
                    let mut destination = parent.clone();
                    destination.push(format!("{frame:0digits$}.png"));
                    image.save(&destination)?;
                }
            }
        }

        Ok(())
    })?;

    let message = match opt.frames {
        FrameSelection::Count(n) if n.get() == 1 => format!(
            "Saved first frame of {} files to {}",
            files.len(),
            output.to_string_lossy()
        ),
        FrameSelection::All => format!(
            "Saved all frames of {} files to {}",
            files.len(),
            output.to_string_lossy()
        ),
        FrameSelection::Count(n) => format!(
            "Saved first {} frames of {} files to {}",
            n,
            files.len(),
            output.to_string_lossy()
        ),
    };

    progress.finish_with_message(message);

    Ok(())
}

pub fn run_main(opt: Opt) -> Result<()> {
    let exporter = Exporter::new(&opt)?;

    if opt.swf.is_file() {
        capture_single_swf(&exporter, &opt)?;
    } else if !opt.swf.is_dir() {
        return Err(anyhow!(
            "Not a file or directory: {}",
            opt.swf.to_string_lossy()
        ));
    } else if opt.output_path.is_some() {
        capture_multiple_swfs(&exporter, &opt)?;
    } else {
        return Err(anyhow!(
            "Output directory is required when exporting multiple files."
        ));
    }

    Ok(())
}
