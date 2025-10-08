use std::panic::catch_unwind;
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;

use image::RgbaImage;
use ruffle_core::Player;
use ruffle_core::PlayerBuilder;
use ruffle_core::limits::ExecutionLimit;
use ruffle_core::tag_utils::movie_from_path;
use ruffle_render_wgpu::backend::WgpuRenderBackend;
use ruffle_render_wgpu::descriptors::Descriptors;

use anyhow::Result;
use anyhow::anyhow;
use ruffle_render_wgpu::backend::request_adapter_and_device;
use ruffle_render_wgpu::target::TextureTarget;
use ruffle_render_wgpu::wgpu;

use crate::cli::FrameSelection;
use crate::cli::Opt;
use crate::cli::SizeOpt;
use crate::player_ext::PlayerExporterExt;

pub struct Exporter {
    descriptors: Arc<Descriptors>,
    size: SizeOpt,
    skipframes: u32,
    frames: FrameSelection,
    force_play: bool,
}

impl Exporter {
    pub fn new(opt: &Opt) -> Result<Self> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: opt.graphics.into(),
            ..Default::default()
        });
        let (adapter, device, queue) = futures::executor::block_on(request_adapter_and_device(
            opt.graphics.into(),
            &instance,
            None,
            opt.power.into(),
        ))
        .map_err(|e| anyhow!(e.to_string()))?;

        let descriptors = Arc::new(Descriptors::new(instance, adapter, device, queue));

        Ok(Self {
            descriptors,
            size: opt.size,
            skipframes: opt.skipframes,
            frames: opt.frames,
            force_play: opt.force_play,
        })
    }

    pub fn start_exporting_movie(&self, swf_path: &Path) -> Result<MovieExport> {
        let movie = movie_from_path(swf_path, None).map_err(|e| anyhow!(e.to_string()))?;

        let width = self
            .size
            .width
            .map(f64::from)
            .unwrap_or_else(|| movie.width().to_pixels());
        let width = (width * self.size.scale).round() as u32;

        let height = self
            .size
            .height
            .map(f64::from)
            .unwrap_or_else(|| movie.height().to_pixels());
        let height = (height * self.size.scale).round() as u32;

        let target = TextureTarget::new(&self.descriptors.device, (width, height))
            .map_err(|e| anyhow!(e.to_string()))?;
        let player = PlayerBuilder::new()
            .with_renderer(
                WgpuRenderBackend::new(self.descriptors.clone(), target)
                    .map_err(|e| anyhow!(e.to_string()))?,
            )
            .with_movie(movie)
            .with_viewport_dimensions(width, height, self.size.scale)
            .build();

        Ok(MovieExport {
            player,
            skipframes: self.skipframes,
            frames: self.frames,
            force_play: self.force_play,
        })
    }
}

pub struct MovieExport {
    player: Arc<Mutex<Player>>,
    skipframes: u32,
    frames: FrameSelection,
    force_play: bool,
}

impl MovieExport {
    pub fn total_frames(&self) -> u32 {
        self.frames.total_frames(&self.player, self.skipframes)
    }

    pub fn run_frame(&self) {
        if self.force_play {
            self.player.force_root_clip_play();
        }

        self.player
            .lock()
            .unwrap()
            .preload(&mut ExecutionLimit::none());

        self.player.lock().unwrap().run_frame();
    }

    pub fn capture_frame(&self) -> Result<RgbaImage> {
        let image = || {
            self.player.lock().unwrap().render();
            self.player.capture_frame()
        };
        match catch_unwind(image) {
            Ok(Some(image)) => Ok(image),
            Ok(None) => Err(anyhow!("No frame captured")),
            Err(e) => Err(anyhow!("{e:?}")),
        }
    }
}
