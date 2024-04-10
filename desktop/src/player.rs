use crate::backends::{
    CpalAudioBackend, DesktopExternalInterfaceProvider, DesktopFSCommandProvider, DesktopUiBackend,
    ExternalNavigatorBackend, RfdNavigatorInterface,
};
use crate::custom_event::RuffleEvent;
use crate::gui::MovieView;
use crate::preferences::GlobalPreferences;
use crate::{CALLSTACK, RENDER_INFO, SWF_INFO};
use anyhow::anyhow;
use ruffle_core::backend::navigator::{OpenURLMode, SocketMode};
use ruffle_core::config::Letterbox;
use ruffle_core::events::{GamepadButton, KeyCode};
use ruffle_core::{
    DefaultFont, LoadBehavior, Player, PlayerBuilder, PlayerEvent, PlayerRuntime, StageAlign,
    StageScaleMode,
};
use ruffle_frontend_utils::backends::executor::{AsyncExecutor, PollRequester};
use ruffle_frontend_utils::bundle::source::BundleSourceError;
use ruffle_frontend_utils::bundle::{Bundle, BundleError};
use ruffle_frontend_utils::content::PlayingContent;
use ruffle_render::backend::RenderBackend;
use ruffle_render::quality::StageQuality;
use ruffle_render_wgpu::backend::WgpuRenderBackend;
use ruffle_render_wgpu::descriptors::Descriptors;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::path::PathBuf;
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
    pub force_scale: bool,
    pub proxy: Option<Url>,
    pub socket_allowed: HashSet<String>,
    pub tcp_connections: SocketMode,
    pub upgrade_to_https: bool,
    pub fullscreen: bool,
    pub load_behavior: LoadBehavior,
    pub save_directory: PathBuf,
    pub letterbox: Letterbox,
    pub spoof_url: Option<Url>,
    pub player_version: u8,
    pub player_runtime: PlayerRuntime,
    pub frame_rate: Option<f64>,
    pub open_url_mode: OpenURLMode,
    pub dummy_external_interface: bool,
    pub gamepad_button_mapping: HashMap<GamepadButton, KeyCode>,
    pub avm2_optimizer_enabled: bool,
}

impl From<&GlobalPreferences> for PlayerOptions {
    fn from(value: &GlobalPreferences) -> Self {
        Self {
            parameters: value.cli.parameters().collect(),
            max_execution_duration: value.cli.max_execution_duration,
            base: value.cli.base.clone(),
            quality: value.cli.quality,
            align: value.cli.align.unwrap_or_default(),
            force_align: value.cli.force_align,
            scale: value.cli.scale,
            force_scale: value.cli.force_scale,
            proxy: value.cli.proxy.clone(),
            upgrade_to_https: value.cli.upgrade_to_https,
            fullscreen: value.cli.fullscreen,
            load_behavior: value.cli.load_behavior,
            save_directory: value.cli.save_directory.clone(),
            letterbox: value.cli.letterbox,
            spoof_url: value.cli.spoof_url.clone(),
            player_version: value.cli.player_version.unwrap_or(32),
            player_runtime: value.cli.player_runtime,
            frame_rate: value.cli.frame_rate,
            open_url_mode: value.cli.open_url_mode,
            dummy_external_interface: value.cli.dummy_external_interface,
            socket_allowed: HashSet::from_iter(value.cli.socket_allow.iter().cloned()),
            tcp_connections: value.cli.tcp_connections,
            gamepad_button_mapping: HashMap::from_iter(value.cli.gamepad_button.iter().cloned()),
            avm2_optimizer_enabled: !value.cli.no_avm2_optimizer,
        }
    }
}

#[derive(Clone)]
struct WinitWaker(EventLoopProxy<RuffleEvent>);

impl PollRequester for WinitWaker {
    fn request_poll(&self) {
        if self.0.send_event(RuffleEvent::TaskPoll).is_err() {
            tracing::error!("Couldn't request poll - event loop is closed");
        }
    }
}

/// Represents a current Player and any associated state with that player,
/// which may be lost when this Player is closed (dropped)
struct ActivePlayer {
    player: Arc<Mutex<Player>>,
    executor: Arc<AsyncExecutor<WinitWaker>>,
}

impl ActivePlayer {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        opt: &PlayerOptions,
        event_loop: EventLoopProxy<RuffleEvent>,
        movie_url: &Url,
        window: Rc<Window>,
        descriptors: Arc<Descriptors>,
        movie_view: MovieView,
        font_database: Rc<fontdb::Database>,
        preferences: GlobalPreferences,
    ) -> Self {
        let mut builder = PlayerBuilder::new();

        match CpalAudioBackend::new(&preferences) {
            Ok(audio) => {
                builder = builder.with_audio(audio);
            }
            Err(e) => {
                tracing::error!("Unable to create audio device: {}", e);
            }
        };

        let mut content = PlayingContent::DirectFile(movie_url.clone());
        if movie_url.scheme() == "file" {
            if let Ok(path) = movie_url.to_file_path() {
                match Bundle::from_path(&path) {
                    Ok(bundle) => {
                        if bundle.warnings().is_empty() {
                            tracing::info!("Opening bundle at {path:?}");
                        } else {
                            // TODO: Show warnings to user (toast?)
                            tracing::warn!("Opening bundle at {path:?} with warnings");
                            for warning in bundle.warnings() {
                                tracing::warn!("{warning}");
                            }
                        }
                        content = PlayingContent::Bundle(movie_url.clone(), bundle);
                    }
                    Err(BundleError::BundleDoesntExist)
                    | Err(BundleError::InvalidSource(BundleSourceError::UnknownSource)) => {
                        // Do nothing and carry on opening it as a swf - this likely isn't a bundle at all
                    }
                    Err(e) => {
                        // TODO: Visible popup when a bundle (or regular file) fails to open
                        tracing::error!("Couldn't open bundle at {path:?}: {e}");
                    }
                }
            }
        }

        let (executor, future_spawner) = AsyncExecutor::new(WinitWaker(event_loop.clone()));
        let movie_url = content.initial_swf_url().clone();
        let readable_name = content.name();
        let navigator = ExternalNavigatorBackend::new(
            opt.base.to_owned().unwrap_or_else(|| movie_url.clone()),
            future_spawner,
            opt.proxy.clone(),
            opt.upgrade_to_https,
            opt.open_url_mode,
            opt.socket_allowed.clone(),
            opt.tcp_connections,
            Rc::new(content),
            RfdNavigatorInterface,
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

        if !opt.gamepad_button_mapping.is_empty() {
            builder = builder.with_gamepad_button_mapping(opt.gamepad_button_mapping.clone());
        }

        builder = builder
            .with_navigator(navigator)
            .with_renderer(renderer)
            .with_storage(preferences.storage_backend().create_backend(opt))
            .with_fs_commands(Box::new(DesktopFSCommandProvider {
                event_loop: event_loop.clone(),
                window: window.clone(),
            }))
            .with_ui(
                DesktopUiBackend::new(
                    window.clone(),
                    opt.open_url_mode,
                    font_database,
                    preferences,
                )
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
            .with_frame_rate(opt.frame_rate)
            .with_avm2_optimizer_enabled(opt.avm2_optimizer_enabled);
        let player = builder.build();

        window.set_title(&format!("Ruffle - {readable_name}"));

        SWF_INFO.with(|i| *i.borrow_mut() = Some(readable_name));

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

            player_lock.set_default_font(
                DefaultFont::Serif,
                vec![
                    "Times New Roman".into(),
                    "Tinos".into(),
                    "Liberation Serif".into(),
                    "DejaVu Serif".into(),
                ],
            );
            player_lock.set_default_font(
                DefaultFont::Sans,
                vec![
                    "Arial".into(),
                    "Arimo".into(),
                    "Liberation Sans".into(),
                    "DejaVu Sans".into(),
                ],
            );
            player_lock.set_default_font(
                DefaultFont::Typewriter,
                vec![
                    "Courier New".into(),
                    "Cousine".into(),
                    "Liberation Mono".into(),
                    "DejaVu Sans Mono".into(),
                ],
            );
            player_lock.set_default_font(
                DefaultFont::JapaneseGothic,
                vec![
                    "ヒラギノ角ゴ Pro W3".into(), // Mac with Japanese environment
                    "MS UI Gothic".into(),        // Windows
                    "Noto Sans CJK JP".into(),    // Linux
                    "Arial Unicode MS".into(),    // Mac fallback
                ],
            );
            player_lock.set_default_font(
                DefaultFont::JapaneseGothicMono,
                vec![
                    "Osaka－等幅".into(),      // Mac with Japanese environment
                    "MS Gothic".into(),        // Windows
                    "Noto Sans CJK JP".into(), // Linux
                    "Arial Unicode MS".into(), // Mac fallback
                ],
            );
            player_lock.set_default_font(
                DefaultFont::JapaneseMincho,
                vec![
                    "ヒラギノ明朝 Pro W3".into(), // Mac with Japanese environment
                    "MS PMincho".into(),          // Windows
                    "Noto Sans CJK JP".into(),    // Linux
                    "Arial Unicode MS".into(),    // Mac fallback
                ],
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
    font_database: Rc<fontdb::Database>,
    preferences: GlobalPreferences,
}

impl PlayerController {
    pub fn new(
        event_loop: EventLoopProxy<RuffleEvent>,
        window: Rc<Window>,
        descriptors: Arc<Descriptors>,
        font_database: fontdb::Database,
        preferences: GlobalPreferences,
    ) -> Self {
        Self {
            player: None,
            event_loop,
            window,
            descriptors,
            font_database: Rc::new(font_database),
            preferences,
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
            self.font_database.clone(),
            self.preferences.clone(),
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
            player.executor.poll_all()
        }
    }
}
