use crate::cli::Opt;
use crate::custom_event::RuffleEvent;
use crate::executor::GlutinAsyncExecutor;
use crate::gui::MovieView;
use crate::RENDER_INFO;
use anyhow::anyhow;
use ruffle_core::backend::audio::AudioBackend;
use ruffle_core::{Player, PlayerBuilder};
use ruffle_render::backend::RenderBackend;
use ruffle_render_wgpu::backend::WgpuRenderBackend;
use ruffle_render_wgpu::descriptors::Descriptors;
use std::rc::Rc;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;
use url::Url;
use winit::event_loop::EventLoopProxy;
use winit::window::Window;

pub struct PlayerController {
    player: Arc<Mutex<Player>>,
    executor: Arc<Mutex<GlutinAsyncExecutor>>,
}

impl PlayerController {
    pub fn new(
        opt: &Opt,
        event_loop: EventLoopProxy<RuffleEvent>,
        movie_url: Option<Url>,
        window: Rc<Window>,
        descriptors: Arc<Descriptors>,
        movie_view: MovieView,
    ) -> Self {
        let mut builder = PlayerBuilder::new();

        match crate::audio::CpalAudioBackend::new() {
            Ok(mut audio) => {
                audio.set_volume(opt.volume);
                builder = builder.with_audio(audio);
            }
            Err(e) => {
                tracing::error!("Unable to create audio device: {}", e);
            }
        };

        let (executor, channel) = GlutinAsyncExecutor::new(event_loop.clone());
        let navigator = crate::navigator::ExternalNavigatorBackend::new(
            opt.base.to_owned().unwrap_or(
                movie_url.unwrap_or_else(|| Url::parse("file:///empty").expect("Dummy Url")),
            ),
            channel,
            event_loop,
            opt.proxy.clone(),
            opt.upgrade_to_https,
            opt.open_url_mode,
        );

        if cfg!(feature = "software_video") {
            builder =
                builder.with_video(ruffle_video_software::backend::SoftwareVideoBackend::new());
        }

        let renderer = WgpuRenderBackend::new(descriptors, movie_view)
            .map_err(|e| anyhow!(e.to_string()))
            .expect("Couldn't create wgpu rendering backend");
        RENDER_INFO.with(|i| *i.borrow_mut() = Some(renderer.debug_info().to_string()));

        builder = builder
            .with_navigator(navigator)
            .with_renderer(renderer)
            .with_storage(
                crate::storage::DiskStorageBackend::new().expect("Couldn't create storage backend"),
            )
            .with_ui(crate::ui::DesktopUiBackend::new(window).expect("Couldn't create ui backend"))
            .with_autoplay(true)
            .with_letterbox(opt.letterbox)
            .with_max_execution_duration(Duration::from_secs_f64(opt.max_execution_duration))
            .with_quality(opt.quality)
            .with_warn_on_unsupported_content(!opt.dont_warn_on_unsupported_content)
            .with_scale_mode(opt.scale, opt.force_scale)
            .with_fullscreen(opt.fullscreen)
            .with_load_behavior(opt.load_behavior)
            .with_spoofed_url(opt.spoof_url.clone().map(|url| url.to_string()))
            .with_player_version(opt.player_version)
            .with_frame_rate(opt.frame_rate);

        Self {
            player: builder.build(),
            executor,
        }
    }

    pub fn get(&self) -> Option<MutexGuard<Player>> {
        // We don't want to return None when the lock fails to grab as that's a fatal error, not a lack of player
        Some(
            self.player
                .try_lock()
                .expect("Player lock must be available"),
        )
    }

    pub fn poll(&self) {
        self.executor
            .lock()
            .expect("active executor reference")
            .poll_all()
    }
}
