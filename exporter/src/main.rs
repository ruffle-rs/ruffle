use futures::executor::block_on;
use image::RgbaImage;
use indicatif::{ProgressBar, ProgressStyle};
use ruffle_core::backend::audio::NullAudioBackend;
use ruffle_core::backend::input::NullInputBackend;
use ruffle_core::backend::navigator::NullNavigatorBackend;
use ruffle_core::tag_utils::SwfMovie;
use ruffle_core::Player;
use ruffle_render_wgpu::target::TextureTarget;
use ruffle_render_wgpu::WgpuRenderBackend;
use std::error::Error;
use std::fs::create_dir_all;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use structopt::StructOpt;
use walkdir::{DirEntry, WalkDir};
use ruffle_core::backend::storage::MemoryStorageBackend;

#[derive(StructOpt, Debug, Copy, Clone)]
struct SizeOpt {
    /// The amount to scale the page size with
    #[structopt(long = "scale", default_value = "1.0")]
    scale: f32,

    /// Optionaly override the output width
    #[structopt(long = "width")]
    width: Option<u32>,

    /// Optionaly override the output height
    #[structopt(long = "height")]
    height: Option<u32>,
}

#[derive(StructOpt, Debug)]
struct Opt {
    /// The file or directory of files to export frames from
    #[structopt(name = "swf", parse(from_os_str))]
    swf: PathBuf,

    /// The file or directory (if multiple frames/files) to store the capture in.
    /// The default value will either be:
    /// - If given one swf and one frame, the name of the swf + ".png"
    /// - If given one swf and multiple frames, the name of the swf as a directory
    /// - If given multiple swfs, this field is required.
    #[structopt(name = "output", parse(from_os_str))]
    output_path: Option<PathBuf>,

    /// Number of frames to capture per file
    #[structopt(short = "f", long = "frames", default_value = "1")]
    frames: u32,

    /// Number of frames to skip
    #[structopt(long = "skipframes", default_value = "0")]
    skipframes: u32,

    /// Don't show a progress bar
    #[structopt(short, long)]
    silent: bool,

    #[structopt(flatten)]
    size: SizeOpt,
}

fn take_screenshot(
    device: Rc<wgpu::Device>,
    queue: Rc<wgpu::Queue>,
    swf_path: &Path,
    frames: u32,
    skipframes: u32,
    progress: &Option<ProgressBar>,
    size: SizeOpt,
) -> Result<Vec<RgbaImage>, Box<dyn std::error::Error>> {
    let movie = SwfMovie::from_path(&swf_path)?;

    let width = size.width.unwrap_or_else(|| movie.width());
    let width = (width as f32 * size.scale).round() as u32;

    let height = size.height.unwrap_or_else(|| movie.height());
    let height = (height as f32 * size.scale).round() as u32;

    let target = TextureTarget::new(&device, (width, height));
    let player = Player::new(
        Box::new(WgpuRenderBackend::new(device, queue, target)?),
        Box::new(NullAudioBackend::new()),
        Box::new(NullNavigatorBackend::new()),
        Box::new(NullInputBackend::new()),
        movie,
        Box::new(MemoryStorageBackend::default())
    )?;

    player
        .lock()
        .unwrap()
        .set_viewport_dimensions(width, height);

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
            let renderer = player
                .renderer_mut()
                .downcast_mut::<WgpuRenderBackend<TextureTarget>>()
                .unwrap();
            let target = renderer.target();
            if let Some(image) = target.capture(renderer.device()) {
                result.push(image);
            } else {
                return Err(format!("Unable to capture frame {} of {:?}", i, swf_path).into());
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
                progress.set_message(&format!("Searching for swf files... {}", results.len()));
            }
        }
    }

    if let Some(progress) = &progress {
        progress.finish_with_message(&format!("Found {} swf files to export", results.len()));
    }

    results
}

fn capture_single_swf(
    device: Rc<wgpu::Device>,
    queue: Rc<wgpu::Queue>,
    opt: &Opt,
) -> Result<(), Box<dyn Error>> {
    let output = opt.output_path.clone().unwrap_or_else(|| {
        let mut result = PathBuf::new();
        if opt.frames == 1 {
            result.set_file_name(opt.swf.file_stem().unwrap());
            result.set_extension("png");
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

    let frames = take_screenshot(
        device,
        queue,
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
        frames.get(0).unwrap().save(&output)?;
    } else {
        for (frame, image) in frames.iter().enumerate() {
            let mut path = PathBuf::from(&output);
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
        progress.finish_with_message(&message);
    } else {
        println!("{}", message);
    }

    Ok(())
}

fn capture_multiple_swfs(
    device: Rc<wgpu::Device>,
    queue: Rc<wgpu::Queue>,
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
        let frames = take_screenshot(
            device.clone(),
            queue.clone(),
            &file.path(),
            opt.frames,
            opt.skipframes,
            &progress,
            opt.size,
        )?;

        if let Some(progress) = &progress {
            progress.set_message(&file.path().file_stem().unwrap().to_string_lossy());
        }

        let mut relative_path = file
            .path()
            .strip_prefix(&opt.swf)
            .unwrap_or_else(|_| &file.path())
            .to_path_buf();

        if frames.len() == 1 {
            let mut destination = PathBuf::from(&output);
            relative_path.set_extension("png");
            destination.push(relative_path);
            if let Some(parent) = destination.parent() {
                let _ = create_dir_all(parent);
            }
            frames.get(0).unwrap().save(&destination)?;
        } else {
            let mut parent = PathBuf::from(&output);
            relative_path.set_extension("");
            parent.push(&relative_path);
            let _ = create_dir_all(&parent);
            for (frame, image) in frames.iter().enumerate() {
                let mut destination = parent.clone();
                destination.push(format!("{}.png", frame));
                image.save(&destination)?;
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

fn main() -> Result<(), Box<dyn Error>> {
    let opt: Opt = Opt::from_args();
    let adapter = block_on(wgpu::Adapter::request(
        &wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::Default,
            compatible_surface: None,
        },
        wgpu::BackendBit::PRIMARY,
    ))
    .ok_or_else(|| {
        "This tool requires hardware acceleration, but no compatible graphics device was found."
    })?;

    let (device, queue) = block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        extensions: wgpu::Extensions {
            anisotropic_filtering: false,
        },
        limits: wgpu::Limits::default(),
    }));

    if opt.swf.is_file() {
        capture_single_swf(Rc::new(device), Rc::new(queue), &opt)?;
    } else if opt.output_path.is_some() {
        capture_multiple_swfs(Rc::new(device), Rc::new(queue), &opt)?;
    } else {
        return Err("Output directory is required when exporting multiple files.".into());
    }

    Ok(())
}
