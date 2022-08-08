use anyhow::{anyhow, Result};
use clap::Parser;
use image::RgbaImage;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use ruffle_core::tag_utils::SwfMovie;
use ruffle_core::PlayerBuilder;
use ruffle_render_wgpu::clap::{GraphicsBackend, PowerPreference};
use ruffle_render_wgpu::target::TextureTarget;
use ruffle_render_wgpu::{wgpu, Descriptors, WgpuRenderBackend};
use std::fs::create_dir_all;
use std::panic::catch_unwind;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use walkdir::{DirEntry, WalkDir};

#[derive(Parser, Debug, Copy, Clone)]
struct SizeOpt {
    /// The amount to scale the page size with
    #[clap(long = "scale", default_value = "1.0", value_parser)]
    scale: f64,

    /// Optionally override the output width
    #[clap(long = "width", value_parser)]
    width: Option<u32>,

    /// Optionally override the output height
    #[clap(long = "height", value_parser)]
    height: Option<u32>,
}

#[derive(Parser, Debug)]
#[clap(name = "Ruffle Exporter", author, version)]
struct Opt {
    /// The file or directory of files to export frames from
    #[clap(name = "swf", value_parser)]
    swf: PathBuf,

    /// The file or directory (if multiple frames/files) to store the capture in.
    /// The default value will either be:
    /// - If given one swf and one frame, the name of the swf + ".png"
    /// - If given one swf and multiple frames, the name of the swf as a directory
    /// - If given multiple swfs, this field is required.
    #[clap(name = "output", value_parser)]
    output_path: Option<PathBuf>,

    /// Number of frames to capture per file
    #[clap(short = 'f', long = "frames", default_value = "1", value_parser)]
    frames: u32,

    /// Number of frames to skip
    #[clap(long = "skipframes", default_value = "0", value_parser)]
    skipframes: u32,

    /// Don't show a progress bar
    #[clap(short, long, action)]
    silent: bool,

    #[clap(flatten)]
    size: SizeOpt,

    /// Type of graphics backend to use. Not all options may be supported by your current system.
    /// Default will attempt to pick the most supported graphics backend.
    #[clap(long, short, default_value = "default", arg_enum, value_parser)]
    graphics: GraphicsBackend,

    /// Power preference for the graphics device used. High power usage tends to prefer dedicated GPUs,
    /// whereas a low power usage tends prefer integrated GPUs.
    #[clap(long, short, default_value = "high", arg_enum, value_parser)]
    power: PowerPreference,

    /// Location to store a wgpu trace output
    #[clap(long, value_parser)]
    #[cfg(feature = "render_trace")]
    trace_path: Option<PathBuf>,
}

fn take_screenshot(
    descriptors: Arc<Descriptors>,
    swf_path: &Path,
    frames: u32,
    skipframes: u32,
    progress: &Option<ProgressBar>,
    size: SizeOpt,
) -> Result<Vec<RgbaImage>> {
    let movie = SwfMovie::from_path(&swf_path, None).map_err(|e| anyhow!(e.to_string()))?;

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

    let target = TextureTarget::new(&descriptors.device, (width, height));
    let player = PlayerBuilder::new()
        .with_renderer(
            WgpuRenderBackend::new(descriptors, target).map_err(|e| anyhow!(e.to_string()))?,
        )
        .with_software_video()
        .with_movie(movie)
        .with_viewport_dimensions(width, height, size.scale as f64)
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
        player.lock().unwrap().run_frame();
        if i >= skipframes {
            match catch_unwind(|| {
                player.lock().unwrap().render();
                let mut player = player.lock().unwrap();
                let renderer = player
                    .renderer_mut()
                    .downcast_mut::<WgpuRenderBackend<TextureTarget>>()
                    .unwrap();
                renderer.capture_frame()
            }) {
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
    )?;

    if let Some(progress) = &progress {
        progress.set_message(opt.swf.file_stem().unwrap().to_string_lossy().into_owned());
    }

    if frames.len() == 1 {
        frames.get(0).unwrap().save(&output)?;
    } else {
        for (frame, image) in frames.iter().enumerate() {
            let mut path: PathBuf = (&output).into();
            path.push(format!("{}.png", frame));
            image.save(&path)?;
        }
    }

    let message = if frames.len() == 1 {
        format!(
            "Saved first frame of {} to {}",
            opt.swf.to_string_lossy(),
            output.to_string_lossy()
        )
    } else {
        format!(
            "Saved first {} frames of {} to {}",
            frames.len(),
            opt.swf.to_string_lossy(),
            output.to_string_lossy()
        )
    };

    if let Some(progress) = progress {
        progress.finish_with_message(message);
    } else {
        println!("{}", message);
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
        let frames = take_screenshot(
            descriptors.clone(),
            file.path(),
            opt.frames,
            opt.skipframes,
            &progress,
            opt.size,
        )?;

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

        if frames.len() == 1 {
            let mut destination: PathBuf = (&output).into();
            relative_path.set_extension("png");
            destination.push(relative_path);
            if let Some(parent) = destination.parent() {
                let _ = create_dir_all(parent);
            }
            frames.get(0).unwrap().save(&destination)?;
        } else {
            let mut parent: PathBuf = (&output).into();
            relative_path.set_extension("");
            parent.push(&relative_path);
            let _ = create_dir_all(&parent);
            for (frame, image) in frames.iter().enumerate() {
                let mut destination = parent.clone();
                destination.push(format!("{}.png", frame));
                image.save(&destination)?;
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
        println!("{}", message);
    }

    Ok(())
}

#[cfg(feature = "render_trace")]
fn trace_path(opt: &Opt) -> Option<&Path> {
    if let Some(path) = &opt.trace_path {
        let _ = std::fs::create_dir_all(path);
        Some(path)
    } else {
        None
    }
}

#[cfg(not(feature = "render_trace"))]
fn trace_path(_opt: &Opt) -> Option<&Path> {
    None
}

fn main() -> Result<()> {
    let opt: Opt = Opt::parse();
    let instance = wgpu::Instance::new(opt.graphics.into());
    let descriptors = Arc::new(
        futures::executor::block_on(WgpuRenderBackend::<TextureTarget>::build_descriptors(
            opt.graphics.into(),
            instance,
            None,
            opt.power.into(),
            trace_path(&opt),
        ))
        .map_err(|e| anyhow!(e.to_string()))?,
    );

    if opt.swf.is_file() {
        capture_single_swf(descriptors, &opt)?;
    } else if opt.output_path.is_some() {
        capture_multiple_swfs(descriptors, &opt)?;
    } else {
        return Err(anyhow!(
            "Output directory is required when exporting multiple files."
        ));
    }

    Ok(())
}
