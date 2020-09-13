use clap::Clap;
use futures::executor::block_on;
use image::RgbaImage;
use indicatif::{ProgressBar, ProgressStyle};
use ruffle_core::backend::audio::NullAudioBackend;
use ruffle_core::backend::input::NullInputBackend;
use ruffle_core::backend::locale::NullLocaleBackend;
use ruffle_core::backend::log::NullLogBackend;
use ruffle_core::backend::navigator::NullNavigatorBackend;
use ruffle_core::backend::render::RenderBackend;
use ruffle_core::backend::storage::MemoryStorageBackend;
use ruffle_core::tag_utils::SwfMovie;
use ruffle_core::Player;
use ruffle_render_svg::SvgRenderBackend;
use ruffle_render_wgpu::clap::{GraphicsBackend, PowerPreference};
use ruffle_render_wgpu::target::TextureTarget;
use ruffle_render_wgpu::{wgpu, Descriptors, WgpuRenderBackend};
use std::error::Error;
use std::fs::{create_dir_all, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use walkdir::{DirEntry, WalkDir};

#[derive(Clap, Debug, Copy, Clone, Eq, PartialEq, Hash)]
enum Format {
    Png,
    Svg,
}

#[derive(Clap, Debug, Copy, Clone)]
struct SizeOpt {
    /// The amount to scale the page size with
    #[clap(long = "scale", default_value = "1.0")]
    scale: f32,

    /// Optionally override the output width
    #[clap(long = "width")]
    width: Option<u32>,

    /// Optionally override the output height
    #[clap(long = "height")]
    height: Option<u32>,
}

#[derive(Clap, Debug)]
#[clap(name = "Ruffle Exporter", author, version)]
struct Opt {
    /// The file or directory of files to export frames from
    #[clap(name = "swf", parse(from_os_str))]
    swf: PathBuf,

    /// The file or directory (if multiple frames/files) to store the capture in.
    /// The default value will either be:
    /// - If given one swf and one frame, the name of the swf + ".png"
    /// - If given one swf and multiple frames, the name of the swf as a directory
    /// - If given multiple swfs, this field is required.
    #[clap(name = "output", parse(from_os_str))]
    output_path: Option<PathBuf>,

    /// Number of frames to capture per file
    #[clap(short = 'f', long = "frames", default_value = "1")]
    frames: u32,

    /// Number of frames to skip
    #[clap(long = "skipframes", default_value = "0")]
    skipframes: u32,

    /// Don't show a progress bar
    #[clap(short, long)]
    silent: bool,

    /// Export file format
    #[clap(long, arg_enum, default_value = "png")]
    format: Format,

    #[clap(flatten)]
    size: SizeOpt,

    /// Type of graphics backend to use. Not all options may be supported by your current system.
    /// Default will attempt to pick the most supported graphics backend.
    #[clap(
        long,
        short,
        case_insensitive = true,
        default_value = "default",
        arg_enum
    )]
    graphics: GraphicsBackend,

    /// Power preference for the graphics device used. High power usage tends to prefer dedicated GPUs,
    /// whereas a low power usage tends prefer integrated GPUs.
    #[clap(long, short, case_insensitive = true, default_value = "high", arg_enum)]
    power: PowerPreference,

    /// Location to store a wgpu trace output
    #[clap(long, parse(from_os_str))]
    #[cfg(feature = "render_trace")]
    trace_path: Option<PathBuf>,
}

enum CaptureDescriptors {
    Png { descriptors: Descriptors },
    Svg,
}

enum FileData {
    Png(RgbaImage),
    Svg(String),
}

fn take_screenshot(
    descriptors: CaptureDescriptors,
    swf_path: &Path,
    frames: u32,
    skipframes: u32,
    progress: &Option<ProgressBar>,
    size: SizeOpt,
) -> Result<(CaptureDescriptors, Vec<FileData>), Box<dyn std::error::Error>> {
    let movie = SwfMovie::from_path(&swf_path)?;

    let width = size.width.unwrap_or_else(|| movie.width());
    let width = (width as f32 * size.scale).round() as u32;

    let height = size.height.unwrap_or_else(|| movie.height());
    let height = (height as f32 * size.scale).round() as u32;
    let is_png = matches!(descriptors, CaptureDescriptors::Png {..});
    let backend: Box<dyn RenderBackend> = match descriptors {
        CaptureDescriptors::Png { descriptors } => {
            let target = TextureTarget::new(&descriptors.device, (width, height));
            Box::new(WgpuRenderBackend::new(descriptors, target)?)
        }
        CaptureDescriptors::Svg => Box::new(SvgRenderBackend::new(width, height)),
    };
    let player = Player::new(
        backend,
        Box::new(NullAudioBackend::new()),
        Box::new(NullNavigatorBackend::new()),
        Box::new(NullInputBackend::new()),
        Box::new(MemoryStorageBackend::default()),
        Box::new(NullLocaleBackend::new()),
        Box::new(NullLogBackend::new()),
    )?;

    player
        .lock()
        .unwrap()
        .set_viewport_dimensions(width, height);
    player.lock().unwrap().set_root_movie(Arc::new(movie));

    let mut result = Vec::new();
    let totalframes = frames + skipframes;

    for i in 0..totalframes {
        if let Some(progress) = &progress {
            progress.set_message(&format!(
                "{} frame {}",
                swf_path.file_stem().unwrap().to_string_lossy(),
                i
            ));
        }
        player.lock().unwrap().run_frame();
        if i >= skipframes {
            player.lock().unwrap().render();
            let mut player = player.lock().unwrap();
            if is_png {
                let renderer = player
                    .renderer_mut()
                    .downcast_mut::<WgpuRenderBackend<TextureTarget>>()
                    .unwrap();
                let target = renderer.target();
                if let Some(image) = target.capture(renderer.device()) {
                    result.push(FileData::Png(image));
                } else {
                    return Err(format!("Unable to capture frame {} of {:?}", i, swf_path).into());
                }
            } else {
                let renderer = player
                    .renderer_mut()
                    .downcast_mut::<SvgRenderBackend>()
                    .unwrap();
                result.push(FileData::Svg(renderer.to_string()));
            }
        }

        if let Some(progress) = &progress {
            progress.inc(1);
        }
    }

    if is_png {
        let descriptors = Arc::try_unwrap(player)
            .ok()
            .unwrap()
            .into_inner()?
            .destroy()
            .downcast::<WgpuRenderBackend<TextureTarget>>()
            .ok()
            .unwrap()
            .descriptors();
        Ok((
            CaptureDescriptors::Png {
                descriptors: descriptors.into(),
            },
            result,
        ))
    } else {
        Ok((CaptureDescriptors::Svg, result))
    }
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
                progress.set_message(&format!("Searching for swf files... {}", results.len()));
            }
        }
    }

    if let Some(progress) = &progress {
        progress.finish_with_message(&format!("Found {} swf files to export", results.len()));
    }

    results
}

fn capture_single_swf(descriptors: CaptureDescriptors, opt: &Opt) -> Result<(), Box<dyn Error>> {
    let output = opt.output_path.clone().unwrap_or_else(|| {
        let mut result = PathBuf::new();
        if opt.frames == 1 {
            result.set_file_name(opt.swf.file_stem().unwrap());
            result.set_extension(if matches!(descriptors, CaptureDescriptors::Png {..}) {
                "png"
            } else {
                "svg"
            });
        } else {
            result.set_file_name(opt.swf.file_stem().unwrap());
        }
        result
    });

    if opt.frames > 1 {
        let _ = create_dir_all(&output);
    }

    let progress = if !opt.silent {
        let progress = ProgressBar::new(opt.frames as u64);
        progress.set_style(
            ProgressStyle::default_bar()
                .template(
                    "[{elapsed_precise}] {bar:40.cyan/blue} [{eta_precise}] {pos:>7}/{len:7} {msg}",
                )
                .progress_chars("##-"),
        );
        Some(progress)
    } else {
        None
    };

    let (_, frames) = take_screenshot(
        descriptors,
        &opt.swf,
        opt.frames,
        opt.skipframes,
        &progress,
        opt.size,
    )?;

    if let Some(progress) = &progress {
        progress.set_message(&opt.swf.file_stem().unwrap().to_string_lossy());
    }

    if frames.len() == 1 {
        match &frames[0] {
            FileData::Png(image) => image.save(&output)?,
            FileData::Svg(data) => File::create(&output)?.write_all(data.as_bytes())?,
        }
    } else {
        for (frame, image) in frames.iter().enumerate() {
            match image {
                FileData::Png(image) => image.save(output.join(format!("{}.png", frame)))?,
                FileData::Svg(data) => File::create(output.join(format!("{}.svg", frame)))?
                    .write_all(data.as_bytes())?,
            }
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
        progress.finish_with_message(&message);
    } else {
        println!("{}", message);
    }

    Ok(())
}

fn capture_multiple_swfs(
    mut descriptors: CaptureDescriptors,
    opt: &Opt,
) -> Result<(), Box<dyn Error>> {
    let output = opt.output_path.clone().unwrap();
    let files = find_files(&opt.swf, !opt.silent);

    let progress = if !opt.silent {
        let progress = ProgressBar::new((files.len() as u64) * (opt.frames as u64));
        progress.set_style(
            ProgressStyle::default_bar()
                .template(
                    "[{elapsed_precise}] {bar:40.cyan/blue} [{eta_precise}] {pos:>7}/{len:7} {msg}",
                )
                .progress_chars("##-"),
        );
        Some(progress)
    } else {
        None
    };

    for file in &files {
        let (new_descriptors, frames) = take_screenshot(
            descriptors,
            &file.path(),
            opt.frames,
            opt.skipframes,
            &progress,
            opt.size,
        )?;
        descriptors = new_descriptors;

        if let Some(progress) = &progress {
            progress.set_message(&file.path().file_stem().unwrap().to_string_lossy());
        }

        let mut relative_path = file
            .path()
            .strip_prefix(&opt.swf)
            .unwrap_or_else(|_| &file.path())
            .to_path_buf();

        if frames.len() == 1 {
            let _ = create_dir_all(&output);
            let mut destination = output.join(relative_path);
            match &frames[0] {
                FileData::Png(image) => {
                    destination.set_extension("png");
                    image.save(&destination)?
                }
                FileData::Svg(data) => {
                    destination.set_extension("svg");
                    File::create(&destination)?.write_all(data.as_bytes())?
                }
            }
        } else {
            let mut parent = PathBuf::from(&output);
            relative_path.set_extension("");
            parent.push(&relative_path);
            let _ = create_dir_all(&parent);
            for (frame, image) in frames.iter().enumerate() {
                match image {
                    FileData::Png(image) => image.save(parent.join(format!("{}.png", frame)))?,
                    FileData::Svg(data) => File::create(parent.join(format!("{}.svg", frame)))?
                        .write_all(data.as_bytes())?,
                }
            }
        }
    }

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
        progress.finish_with_message(&message);
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

fn main() -> Result<(), Box<dyn Error>> {
    let opt: Opt = Opt::parse();
    let descriptors = match opt.format {
        Format::Png => {
            let instance = wgpu::Instance::new(opt.graphics.into());
            let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
                 power_preference: opt.power.into(),
                 compatible_surface: None,
             }))
                 .ok_or(
                     "This tool requires hardware acceleration, but no compatible graphics device was found."
                 )?;

            let (device, queue) = block_on(adapter.request_device(
                &wgpu::DeviceDescriptor {
                    features: Default::default(),
                    limits: wgpu::Limits::default(),
                    shader_validation: false,
                },
                trace_path(&opt),
            ))?;
            let descriptors = Descriptors::new(device, queue)?;
            CaptureDescriptors::Png {
                descriptors: descriptors.into(),
            }
        }
        Format::Svg => CaptureDescriptors::Svg,
    };

    if opt.swf.is_file() {
        capture_single_swf(descriptors, &opt)?;
    } else if opt.output_path.is_some() {
        capture_multiple_swfs(descriptors, &opt)?;
    } else {
        return Err("Output directory is required when exporting multiple files.".into());
    }

    Ok(())
}
