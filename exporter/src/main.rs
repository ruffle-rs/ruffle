mod options;

use anyhow::{anyhow, Result};
use clap::Parser;
use image::RgbaImage;
use indicatif::{ProgressBar, ProgressStyle};
use options::{Opt, SizeOpt};
use rayon::prelude::*;
use ruffle_core::limits::ExecutionLimit;
use ruffle_core::tag_utils::SwfMovie;
use ruffle_core::PlayerBuilder;
use ruffle_render_wgpu::backend::{request_adapter_and_device, WgpuRenderBackend};
use ruffle_render_wgpu::descriptors::Descriptors;
use ruffle_render_wgpu::target::TextureTarget;
use ruffle_render_wgpu::wgpu;
use std::fs::create_dir_all;
use std::io::{self, Write};
use std::panic::catch_unwind;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use walkdir::{DirEntry, WalkDir};

/// Captures a screenshot. The resulting image uses straight alpha
fn take_screenshot(
    descriptors: Arc<Descriptors>,
    swf_path: &Path,
    frames: u32,
    skipframes: u32,
    progress: &Option<ProgressBar>,
    size: SizeOpt,
    skip_unsupported: bool,
) -> Result<Vec<RgbaImage>> {
    let movie = SwfMovie::from_path(swf_path, None).map_err(|e| anyhow!(e.to_string()))?;

    if movie.is_action_script_3() && skip_unsupported {
        return Err(anyhow!("Skipping unsupported movie"));
    }

    let width = size
        .width
        .map(f64::from)
        .unwrap_or_else(|| movie.width().to_pixels());
    let width = (width * size.scale).round() as u32;

    let height = size
        .height
        .map(f64::from)
        .unwrap_or_else(|| movie.height().to_pixels());
    let height = (height * size.scale).round() as u32;

    let target = TextureTarget::new(&descriptors.device, (width, height))
        .map_err(|e| anyhow!(e.to_string()))?;
    let player = PlayerBuilder::new()
        .with_renderer(
            WgpuRenderBackend::new(descriptors, target).map_err(|e| anyhow!(e.to_string()))?,
        )
        .with_movie(movie)
        .with_viewport_dimensions(width, height, size.scale)
        .build();

    let mut result = Vec::new();
    let totalframes = frames + skipframes;

    for i in 0..totalframes {
        player.lock().unwrap().preload(&mut ExecutionLimit::none());
        player.lock().unwrap().run_frame();

        if i >= skipframes {
            if let Some(progress) = &progress {
                progress.set_message(format!(
                    "{} frame {}",
                    swf_path.file_stem().unwrap().to_string_lossy(),
                    i + 1
                ));
            }

            let image = || {
                player.lock().unwrap().render();
                let mut player = player.lock().unwrap();
                let renderer = player
                    .renderer_mut()
                    .downcast_mut::<WgpuRenderBackend<TextureTarget>>()
                    .unwrap();
                renderer.capture_frame()
            };

            match catch_unwind(image) {
                Ok(Some(image)) => result.push(image),
                Ok(None) => return Err(anyhow!("Unable to capture frame {} of {:?}", i, swf_path)),
                Err(e) => {
                    return Err(anyhow!(
                        "Unable to capture frame {} of {:?}: {:?}",
                        i,
                        swf_path,
                        e
                    ))
                }
            }

            if let Some(progress) = &progress {
                progress.inc(1);
            }
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

fn write_single_frame(frames: &[RgbaImage], opt: &Opt, output: &Path) -> Result<()> {
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
        image.save(output)?;
    }

    Ok(())
}

fn write_multiple_frames(
    frames: &[image::ImageBuffer<image::Rgba<u8>, Vec<u8>>],
    output: &Path,
    skipframes: usize,
) -> Result<(), anyhow::Error> {
    for (frame, image) in frames.iter().enumerate() {
        let adjusted_frame = frame + skipframes;
        let mut path: PathBuf = (output).into();
        path.push(format!("{adjusted_frame}.png"));
        image.save(&path)?;
    }
    Ok(())
}

fn capture_swf(
    descriptors: Arc<Descriptors>,
    opt: &Opt,
    progress: &Option<ProgressBar>,
    input: &Path,
    output: &Path,
) -> Result<usize> {
    let mut frames_written = 0;

    while frames_written != opt.frames {
        let frame_difference: i32 =
            if opt.frame_cache > 0 && (opt.frames - frames_written) < opt.frame_cache {
                opt.frames as i32 - frames_written as i32
            } else {
                0
            };

        let frame_amount = if frame_difference > 0 {
            frame_difference as u32
        } else if opt.frame_cache == 0 || opt.frame_cache > opt.frames {
            opt.frames
        } else {
            opt.frame_cache
        };

        let frames = take_screenshot(
            descriptors.clone(),
            input,
            frame_amount,
            frames_written + opt.skipframes,
            progress,
            opt.size,
            opt.skip_unsupported,
        )?;

        if opt.frames == 1 {
            write_single_frame(&frames, opt, output)?;
        } else {
            write_multiple_frames(&frames, output, frames_written.try_into()?)?;
        }

        if frame_difference > 0 {
            frames_written += frame_difference as u32;
        } else if opt.frame_cache == 0 {
            frames_written = opt.frames;
        } else {
            frames_written += opt.frame_cache;
        }
    }

    Ok(frames_written as usize)
}

fn capture_single_swf(descriptors: Arc<Descriptors>, opt: &Opt) -> Result<()> {
    let output = opt.output_path.clone().unwrap_or_else(|| {
        let mut result = PathBuf::new();
        result.set_file_name(opt.swf.file_stem().unwrap());
        if opt.frames == 1 {
            result.set_extension("png");
        }
        result
    });

    if opt.frames > 1 {
        let _ = create_dir_all(&output);
    }

    let progress = if !opt.silent {
        let progress = ProgressBar::new(opt.frames as u64);
        progress.set_style(
            ProgressStyle::with_template(
                "[{elapsed_precise}] {bar:40.cyan/blue} [{eta_precise}] {pos:>7}/{len:7} {msg}",
            )
            .unwrap()
            .progress_chars("##-"),
        );
        Some(progress)
    } else {
        None
    };

    let frames_len = capture_swf(descriptors.clone(), opt, &progress, &opt.swf, &output)?;

    let message = if frames_len == 1 {
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
            frames_len,
            opt.swf.to_string_lossy(),
            output.to_string_lossy()
        ))
    };

    if let Some(message) = message {
        if let Some(progress) = progress {
            progress.finish_with_message(message);
        } else {
            println!("{message}");
        }
    }

    Ok(())
}

#[allow(clippy::branches_sharing_code)]
fn capture_multiple_swfs(descriptors: Arc<Descriptors>, opt: &Opt) -> Result<()> {
    let output = opt.output_path.clone().unwrap();
    let files = find_files(&opt.swf, !opt.silent);

    let progress = if !opt.silent {
        let progress = ProgressBar::new((files.len() as u64) * (opt.frames as u64));
        progress.set_style(
            ProgressStyle::with_template(
                "[{elapsed_precise}] {bar:40.cyan/blue} [{eta_precise}] {pos:>7}/{len:7} {msg}",
            )
            .unwrap()
            .progress_chars("##-"),
        );
        Some(progress)
    } else {
        None
    };

    files.par_iter().try_for_each(|file| -> Result<()> {
        if let Some(progress) = &progress {
            progress.set_message(
                file.path()
                    .file_stem()
                    .unwrap()
                    .to_string_lossy()
                    .into_owned(),
            );
        }

        let mut relative_path = file
            .path()
            .strip_prefix(&opt.swf)
            .unwrap_or_else(|_| file.path())
            .to_path_buf();

        let output_path: PathBuf = if opt.frames == 1 {
            let mut destination: PathBuf = (&output).into();
            relative_path.set_extension("png");
            destination.push(relative_path);
            if let Some(parent) = destination.parent() {
                let _ = create_dir_all(parent);
            }
            destination
        } else {
            let mut parent: PathBuf = (&output).into();
            relative_path.set_extension("");
            parent.push(&relative_path);
            let _ = create_dir_all(&parent);
            parent
        };

        capture_swf(
            descriptors.clone(),
            opt,
            &progress,
            file.path(),
            &output_path,
        )?;

        Ok(())
    })?;

    let message = if opt.frames == 1 {
        format!(
            "Saved first frame of {} files to {}",
            files.len(),
            output.to_string_lossy()
        )
    } else {
        format!(
            "Saved first {} frames of {} files to {}",
            opt.frames,
            files.len(),
            output.to_string_lossy()
        )
    };

    if let Some(progress) = progress {
        progress.finish_with_message(message);
    } else {
        println!("{message}");
    }

    Ok(())
}

fn trace_path(_opt: &Opt) -> Option<&Path> {
    None
}

fn main() -> Result<()> {
    let opt: Opt = Opt::parse();
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: opt.graphics.into(),
        ..Default::default()
    });
    let (adapter, device, queue) = futures::executor::block_on(request_adapter_and_device(
        opt.graphics.into(),
        &instance,
        None,
        opt.power.into(),
        trace_path(&opt),
    ))
    .map_err(|e| anyhow!(e.to_string()))?;

    let descriptors = Arc::new(Descriptors::new(instance, adapter, device, queue));

    if opt.swf.is_file() {
        capture_single_swf(descriptors, &opt)?;
    } else if !opt.swf.is_dir() {
        return Err(anyhow!("Given path is not a file or directory."));
    } else if opt.output_path.is_some() {
        capture_multiple_swfs(descriptors, &opt)?;
    } else {
        return Err(anyhow!(
            "Output directory is required when exporting multiple files."
        ));
    }

    Ok(())
}
