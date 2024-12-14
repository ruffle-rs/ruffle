use anyhow::{anyhow, Result};
use clap::Parser;
use image::RgbaImage;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use ruffle_core::limits::ExecutionLimit;
use ruffle_core::tag_utils::SwfMovie;
use ruffle_core::PlayerBuilder;
use ruffle_render_wgpu::backend::{request_adapter_and_device, WgpuRenderBackend};
use ruffle_render_wgpu::clap::{GraphicsBackend, PowerPreference};
use ruffle_render_wgpu::descriptors::Descriptors;
use ruffle_render_wgpu::target::TextureTarget;
use ruffle_render_wgpu::wgpu;
use std::fs::create_dir_all;
use std::io::{self, Write};
use std::panic::catch_unwind;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use walkdir::{DirEntry, WalkDir};

#[derive(Parser, Debug, Copy, Clone)]
struct SizeOpt {
    /// The amount to scale the page size with
    #[clap(long = "scale", default_value = "1.0")]
    scale: f64,

    /// Optionally override the output width
    #[clap(long = "width")]
    width: Option<u32>,

    /// Optionally override the output height
    #[clap(long = "height")]
    height: Option<u32>,
}

#[derive(Parser, Debug)]
#[clap(name = "Ruffle Exporter", author, version)]
struct Opt {
    /// The file or directory of files to export frames from
    #[clap(name = "swf")]
    swf: PathBuf,

    /// The file or directory (if multiple frames/files) to store the capture in.
    /// The default value will either be:
    /// - If given one swf and one frame, the name of the swf + ".png"
    /// - If given one swf and multiple frames, the name of the swf as a directory
    /// - If given multiple swfs, this field is required.
    #[clap(name = "output")]
    output_path: Option<PathBuf>,

    /// Number of frames to capture per file
    #[clap(short = 'f', long = "frames", default_value = "1")]
    frames: u32,

    /// Number of frames to skip
    #[clap(long = "skipframes", default_value = "0")]
    skipframes: u32,

    /// Don't show a progress bar
    #[clap(short, long, action)]
    silent: bool,

    #[clap(flatten)]
    size: SizeOpt,

    /// Type of graphics backend to use. Not all options may be supported by your current system.
    /// Default will attempt to pick the most supported graphics backend.
    #[clap(long, short, default_value = "default")]
    graphics: GraphicsBackend,

    /// Power preference for the graphics device used. High power usage tends to prefer dedicated GPUs,
    /// whereas a low power usage tends prefer integrated GPUs.
    #[clap(long, short, default_value = "high")]
    power: PowerPreference,

    /// Skip unsupported movie types (currently AVM 2)
    #[clap(long, action)]
    skip_unsupported: bool,
}

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
        if let Some(progress) = &progress {
            progress.set_message(format!(
                "{} frame {}",
                swf_path.file_stem().unwrap().to_string_lossy(),
                i
            ));
        }

        player.lock().unwrap().preload(&mut ExecutionLimit::none());

        player.lock().unwrap().run_frame();
        if i >= skipframes {
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
        }

        if let Some(progress) = &progress {
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

    let frames = take_screenshot(
        descriptors,
        &opt.swf,
        opt.frames,
        opt.skipframes,
        &progress,
        opt.size,
        opt.skip_unsupported,
    )?;

    if let Some(progress) = &progress {
        progress.set_message(opt.swf.file_stem().unwrap().to_string_lossy().into_owned());
    }

    if frames.len() == 1 {
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
        for (frame, image) in frames.iter().enumerate() {
            let mut path: PathBuf = (&output).into();
            path.push(format!("{frame}.png"));
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
        if let Ok(frames) = take_screenshot(
            descriptors.clone(),
            file.path(),
            opt.frames,
            opt.skipframes,
            &progress,
            opt.size,
            opt.skip_unsupported,
        ) {
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
                for (frame, image) in frames.iter().enumerate() {
                    let mut destination = parent.clone();
                    destination.push(format!("{frame}.png"));
                    image.save(&destination)?;
                }
            }
        }

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
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
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
