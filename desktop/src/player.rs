use crate::backends::{
    CpalAudioBackend, DesktopExternalInterfaceProvider, DesktopFSCommandProvider, DesktopUiBackend,
    DiskStorageBackend, ExternalNavigatorBackend,
};
use crate::cli::Opt;
use crate::custom_event::RuffleEvent;
use crate::executor::WinitAsyncExecutor;
use crate::gui::MovieView;
use crate::{CALLSTACK, RENDER_INFO, SWF_INFO};
use anyhow::anyhow;
use ruffle_core::backend::navigator::{OpenURLMode, SocketMode};
use ruffle_core::config::Letterbox;
use ruffle_core::{
    LoadBehavior, Player, PlayerBuilder, PlayerEvent, PlayerRuntime, StageAlign, StageScaleMode,
};
use ruffle_render::backend::RenderBackend;
use ruffle_render::quality::StageQuality;
use ruffle_render_wgpu::backend::WgpuRenderBackend;
use ruffle_render_wgpu::descriptors::Descriptors;
use std::collections::HashSet;
use std::rc::Rc;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;
use url::Url;
use winit::event_loop::EventLoopProxy;
use winit::window::Window;

/// Options used when creating a Player (& passed through to a PlayerBuilder).
/// These may be primed by command line arguments.
#[derive(Debug, Clone)]
pub struct PlayerOptions {
    pub parameters: Vec<(String, String)>,
    pub max_execution_duration: f64,
    pub base: Option<Url>,
    pub quality: StageQuality,
    pub align: StageAlign,
    pub force_align: bool,
    pub scale: StageScaleMode,
    pub volume: f32,
    pub force_scale: bool,
    pub proxy: Option<Url>,
    pub socket_allowed: HashSet<String>,
    pub tcp_connections: SocketMode,
    pub upgrade_to_https: bool,
    pub fullscreen: bool,
    pub load_behavior: LoadBehavior,
    pub letterbox: Letterbox,
    pub spoof_url: Option<Url>,
    pub player_version: u8,
    pub player_runtime: PlayerRuntime,
    pub frame_rate: Option<f64>,
    pub open_url_mode: OpenURLMode,
    pub dummy_external_interface: bool,
}

impl From<&Opt> for PlayerOptions {
    fn from(value: &Opt) -> Self {
        Self {
            parameters: value.parameters().collect(),
            max_execution_duration: value.max_execution_duration,
            base: value.base.clone(),
            quality: value.quality,
            align: value.align.unwrap_or_default(),
            force_align: value.force_align,
            scale: value.scale,
            volume: value.volume,
            force_scale: value.force_scale,
            proxy: value.proxy.clone(),
            upgrade_to_https: value.upgrade_to_https,
            fullscreen: value.fullscreen,
            load_behavior: value.load_behavior,
            letterbox: value.letterbox,
            spoof_url: value.spoof_url.clone(),
            player_version: value.player_version.unwrap_or(32),
            player_runtime: value.player_runtime,
            frame_rate: value.frame_rate,
            open_url_mode: value.open_url_mode,
            dummy_external_interface: value.dummy_external_interface,
            socket_allowed: HashSet::from_iter(value.socket_allow.iter().cloned()),
            tcp_connections: value.tcp_connections,
        }
    }
}

/// Represents a current Player and any associated state with that player,
/// which may be lost when this Player is closed (dropped)
struct ActivePlayer {
    player: Arc<Mutex<Player>>,
    executor: Arc<Mutex<WinitAsyncExecutor>>,
}

impl ActivePlayer {
    pub fn new(
        opt: &PlayerOptions,
        event_loop: EventLoopProxy<RuffleEvent>,
        movie_url: &Url,
        window: Rc<Window>,
        descriptors: Arc<Descriptors>,
        movie_view: MovieView,
    ) -> Self {
        let mut builder = PlayerBuilder::new();

        match CpalAudioBackend::new() {
            Ok(audio) => {
                builder = builder.with_audio(audio);
            }
            Err(e) => {
                tracing::error!("Unable to create audio device: {}", e);
            }
        };

        let (executor, channel) = WinitAsyncExecutor::new(event_loop.clone());
        let navigator = ExternalNavigatorBackend::new(
            opt.base.to_owned().unwrap_or_else(|| movie_url.clone()),
            channel,
            event_loop.clone(),
            opt.proxy.clone(),
            opt.upgrade_to_https,
            opt.open_url_mode,
            opt.socket_allowed.clone(),
            opt.tcp_connections,
        );

        if cfg!(feature = "software_video") {
            builder =
                builder.with_video(ruffle_video_software::backend::SoftwareVideoBackend::new());
        }

        let renderer = WgpuRenderBackend::new(descriptors, movie_view)
            .map_err(|e| anyhow!(e.to_string()))
            .expect("Couldn't create wgpu rendering backend");
        RENDER_INFO.with(|i| *i.borrow_mut() = Some(renderer.debug_info().to_string()));

        if opt.dummy_external_interface {
            builder = builder.with_external_interface(Box::new(DesktopExternalInterfaceProvider {
                spoof_url: opt.spoof_url.clone(),
            }));
        }

        let max_execution_duration = if opt.max_execution_duration == f64::INFINITY {
            Duration::MAX
        } else {
            Duration::from_secs_f64(opt.max_execution_duration)
        };

        builder = builder
            .with_navigator(navigator)
            .with_renderer(renderer)
            .with_storage(DiskStorageBackend::new().expect("Couldn't create storage backend"))
            .with_fs_commands(Box::new(DesktopFSCommandProvider {
                event_loop: event_loop.clone(),
                window: window.clone(),
            }))
            .with_ui(
                DesktopUiBackend::new(window.clone(), opt.open_url_mode)
                    .expect("Couldn't create ui backend"),
            )
            .with_autoplay(true)
            .with_letterbox(opt.letterbox)
            .with_max_execution_duration(max_execution_duration)
            .with_quality(opt.quality)
            .with_align(opt.align, opt.force_align)
            .with_scale_mode(opt.scale, opt.force_scale)
            .with_fullscreen(opt.fullscreen)
            .with_load_behavior(opt.load_behavior)
            .with_spoofed_url(opt.spoof_url.clone().map(|url| url.to_string()))
            .with_page_url(opt.spoof_url.clone().map(|url| url.to_string()))
            .with_player_version(Some(opt.player_version))
            .with_player_runtime(opt.player_runtime)
            .with_frame_rate(opt.frame_rate);
        let player = builder.build();

        let name = movie_url
            .path_segments()
            .and_then(|segments| segments.last())
            .unwrap_or_else(|| movie_url.as_str())
            .to_string();

        window.set_title(&format!("Ruffle - {name}"));

        SWF_INFO.with(|i| *i.borrow_mut() = Some(name.clone()));

        let on_metadata = move |swf_header: &ruffle_core::swf::HeaderExt| {
            let _ = event_loop.send_event(RuffleEvent::OnMetadata(swf_header.clone()));
        };

        {
            let mut player_lock = player.lock().expect("Player lock must be available");
            CALLSTACK.with(|callstack| {
                *callstack.borrow_mut() = Some(player_lock.callstack());
            });
            player_lock.fetch_root_movie(
                movie_url.to_string(),
                opt.parameters.to_owned(),
                Box::new(on_metadata),
            );
        }

        Self { player, executor }
    }
}

/// Owner of a Ruffle Player (via ActivePlayer),
/// responsible for either creating, destroying or communicating with that player.
pub struct PlayerController {
    player: Option<ActivePlayer>,
    event_loop: EventLoopProxy<RuffleEvent>,
    window: Rc<Window>,
    descriptors: Arc<Descriptors>,
}

impl PlayerController {
    pub fn new(
        event_loop: EventLoopProxy<RuffleEvent>,
        window: Rc<Window>,
        descriptors: Arc<Descriptors>,
    ) -> Self {
        Self {
            player: None,
            event_loop,
            window,
            descriptors,
        }
    }

    pub fn create(&mut self, opt: &PlayerOptions, movie_url: &Url, movie_view: MovieView) {
        self.player = Some(ActivePlayer::new(
            opt,
            self.event_loop.clone(),
            movie_url,
            self.window.clone(),
            self.descriptors.clone(),
            movie_view,
        ));
    }

    pub fn destroy(&mut self) {
        self.player = None;
    }

    pub fn get(&self) -> Option<MutexGuard<Player>> {
        match &self.player {
            None => None,
            // We don't want to return None when the lock fails to grab as that's a fatal error, not a lack of player
            Some(player) => Some(
                player
                    .player
                    .try_lock()
                    .expect("Player lock must be available"),
            ),
        }
    }

    pub fn handle_event(&self, event: PlayerEvent) {
        if let Some(mut player) = self.get() {
            if player.is_playing() {
                player.handle_event(event);
            }
        }
    }

    pub fn poll(&self) {
        if let Some(player) = &self.player {
            player
                .executor
                .lock()
                .expect("Executor lock must be available")
                .poll_all()
        }
    }
}
