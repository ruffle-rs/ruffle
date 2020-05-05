use futures::executor::block_on;
use ruffle_core::backend::audio::NullAudioBackend;
use ruffle_core::backend::input::NullInputBackend;
use ruffle_core::backend::navigator::NullNavigatorBackend;
use ruffle_core::tag_utils::SwfMovie;
use ruffle_core::Player;
use ruffle_render_wgpu::target::TextureTarget;
use ruffle_render_wgpu::WgpuRenderBackend;
use std::fs::create_dir_all;
use std::path::{Path, PathBuf};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
struct Opt {
    /// The swf file to export frames from
    #[structopt(name = "swf", parse(from_os_str))]
    swf: PathBuf,

    /// The file or directory (if multiple frames) to store the capture in
    #[structopt(name = "output", parse(from_os_str))]
    output_path: Option<PathBuf>,

    /// Number of frames to capture
    #[structopt(short = "f", long = "frames", default_value = "1")]
    frames: u32,
}

fn take_screenshot<P: AsRef<Path>>(
    swf_path: P,
    output: P,
    frames: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    let movie = SwfMovie::from_path(swf_path)?;

    let adapter = block_on(wgpu::Adapter::request(
        &wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::Default,
            compatible_surface: None,
        },
        wgpu::BackendBit::PRIMARY,
    ))
    .unwrap();

    let (device, queue) = block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        extensions: wgpu::Extensions {
            anisotropic_filtering: false,
        },
        limits: wgpu::Limits::default(),
    }));
    let target = TextureTarget::new(&device, (movie.width(), movie.height()));
    let player = Player::new(
        Box::new(WgpuRenderBackend::new(device, queue, target)?),
        Box::new(NullAudioBackend::new()),
        Box::new(NullNavigatorBackend::new()),
        Box::new(NullInputBackend::new()),
        movie,
    )?;

    for i in 0..frames {
        player.lock().unwrap().run_frame();
        player.lock().unwrap().render();

        let mut player = player.lock().unwrap();
        let renderer = player
            .renderer_mut()
            .downcast_mut::<WgpuRenderBackend<TextureTarget>>()
            .unwrap();
        let target = renderer.target();
        if let Some(image) = target.capture(renderer.device()) {
            if frames > 1 {
                let mut path = PathBuf::from(output.as_ref());
                path.push(format!("frame_{}.png", i));
                image.save(&path)?;
            } else {
                image.save(&output)?;
            }
        }
    }

    Ok(())
}

fn main() {
    let opt: Opt = Opt::from_args();
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
    match take_screenshot(opt.swf.clone(), output.clone(), opt.frames) {
        Ok(_) => {
            if opt.frames == 1 {
                println!(
                    "Saved first frame of {} to {}",
                    opt.swf.to_string_lossy(),
                    output.to_string_lossy()
                );
            } else {
                println!(
                    "Saved first {} frames of {} to {}",
                    opt.frames,
                    opt.swf.to_string_lossy(),
                    output.to_string_lossy()
                );
            }
        }
        Err(e) => {
            println!("Couldn't capture swf: {}", e);
            std::process::exit(1);
        }
    }
}
