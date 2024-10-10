use crate::external_interface::JavascriptInterface;
use crate::navigator::WebNavigatorBackend;
use crate::{
    audio, log_adapter, storage, ui, JavascriptPlayer, RuffleHandle, SocketProxy,
    RUFFLE_GLOBAL_PANIC,
};
use js_sys::Promise;
use ruffle_core::backend::audio::{AudioBackend, NullAudioBackend};
use ruffle_core::backend::navigator::OpenURLMode;
use ruffle_core::backend::storage::{MemoryStorageBackend, StorageBackend};
use ruffle_core::backend::ui::FontDefinition;
use ruffle_core::compatibility_rules::CompatibilityRules;
use ruffle_core::config::{Letterbox, NetworkingAccessMode};
use ruffle_core::ttf_parser;
use ruffle_core::{
    swf, Color, DefaultFont, Player, PlayerBuilder, PlayerRuntime, StageAlign, StageScaleMode,
};
use ruffle_render::backend::RenderBackend;
use ruffle_render::quality::StageQuality;
use ruffle_video_external::backend::ExternalVideoBackend;
use ruffle_web_common::JsResult;
use std::cell::RefCell;
use std::collections::HashMap;
use std::error::Error;
use std::rc::Rc;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tracing_subscriber::layer::{Layered, SubscriberExt};
use tracing_subscriber::Registry;
use tracing_wasm::{WASMLayer, WASMLayerConfigBuilder};
use wasm_bindgen::prelude::*;
use web_sys::{HtmlCanvasElement, HtmlElement};

#[wasm_bindgen(inspectable)]
#[derive(Debug, Clone)]
pub struct RuffleInstanceBuilder {
    pub(crate) allow_script_access: bool,
    pub(crate) background_color: Option<Color>,
    pub(crate) letterbox: Letterbox,
    pub(crate) upgrade_to_https: bool,
    pub(crate) compatibility_rules: CompatibilityRules,
    pub(crate) base_url: Option<String>,
    pub(crate) show_menu: bool,
    pub(crate) allow_fullscreen: bool,
    pub(crate) stage_align: StageAlign,
    pub(crate) force_align: bool,
    pub(crate) quality: StageQuality,
    pub(crate) scale: StageScaleMode,
    pub(crate) force_scale: bool,
    pub(crate) frame_rate: Option<f64>,
    pub(crate) wmode: Option<String>, // TODO: Enumify? `Player` is working in strings here too...
    pub(crate) log_level: tracing::Level,
    pub(crate) max_execution_duration: Duration,
    pub(crate) player_version: Option<u8>,
    pub(crate) preferred_renderer: Option<String>, // TODO: Enumify?
    pub(crate) open_url_mode: OpenURLMode,
    pub(crate) allow_networking: NetworkingAccessMode,
    pub(crate) socket_proxy: Vec<SocketProxy>,
    pub(crate) credential_allow_list: Vec<String>,
    pub(crate) player_runtime: PlayerRuntime,
    pub(crate) volume: f32,
    pub(crate) default_fonts: HashMap<DefaultFont, Vec<String>>,
    pub(crate) custom_fonts: Vec<(String, Vec<u8>)>,
}

impl Default for RuffleInstanceBuilder {
    fn default() -> Self {
        // Anything available in `BaseLoadOptions` should match the default we list in the docs there.
        // Some options may be variable (eg allowScriptAccess based on URL) -
        // those should be always overriding these values in JS

        Self {
            allow_script_access: false,
            background_color: None,
            letterbox: Letterbox::Fullscreen,
            upgrade_to_https: true,
            compatibility_rules: CompatibilityRules::default(),
            base_url: None,
            show_menu: true,
            allow_fullscreen: false,
            stage_align: StageAlign::empty(),
            force_align: false,
            quality: StageQuality::High,
            scale: StageScaleMode::ShowAll,
            force_scale: false,
            frame_rate: None,
            wmode: None,
            log_level: tracing::Level::ERROR,
            max_execution_duration: Duration::from_secs_f64(15.0),
            player_version: None,
            preferred_renderer: None,
            open_url_mode: OpenURLMode::Allow,
            allow_networking: NetworkingAccessMode::All,
            socket_proxy: vec![],
            credential_allow_list: vec![],
            player_runtime: PlayerRuntime::FlashPlayer,
            volume: 1.0,
            default_fonts: HashMap::new(),
            custom_fonts: vec![],
        }
    }
}

#[wasm_bindgen]
impl RuffleInstanceBuilder {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self::default()
    }

    #[wasm_bindgen(js_name = "setAllowScriptAccess")]
    pub fn set_allow_script_access(&mut self, value: bool) {
        self.allow_script_access = value;
    }

    #[wasm_bindgen(js_name = "setBackgroundColor")]
    pub fn set_background_color(&mut self, value: Option<u32>) {
        self.background_color = value.map(|rgb| Color::from_rgb(rgb, 255));
    }

    #[wasm_bindgen(js_name = "setUpgradeToHttps")]
    pub fn set_upgrade_to_https(&mut self, value: bool) {
        self.upgrade_to_https = value;
    }

    #[wasm_bindgen(js_name = "setCompatibilityRules")]
    pub fn set_compatibility_rules(&mut self, value: bool) {
        self.compatibility_rules = if value {
            CompatibilityRules::default()
        } else {
            CompatibilityRules::empty()
        };
    }

    #[wasm_bindgen(js_name = "setLetterbox")]
    pub fn set_letterbox(&mut self, value: &str) {
        self.letterbox = match value {
            "off" => Letterbox::Off,
            "fullscreen" => Letterbox::Fullscreen,
            "on" => Letterbox::On,
            _ => return,
        };
    }

    #[wasm_bindgen(js_name = "setBaseUrl")]
    pub fn set_base_url(&mut self, value: Option<String>) {
        self.base_url = value;
    }

    #[wasm_bindgen(js_name = "setShowMenu")]
    pub fn set_show_menu(&mut self, value: bool) {
        self.show_menu = value;
    }

    #[wasm_bindgen(js_name = "setAllowFullscreen")]
    pub fn set_allow_fullscreen(&mut self, value: bool) {
        self.allow_fullscreen = value;
    }

    #[wasm_bindgen(js_name = "setStageAlign")]
    pub fn set_stage_align(&mut self, value: &str) {
        // [NA] This is weird. Do we really need this?

        // Chars get converted into flags.
        // This means "tbbtlbltblbrllrbltlrtbl" is valid, resulting in "TBLR".
        let mut align = StageAlign::default();
        for c in value.bytes().map(|c| c.to_ascii_uppercase()) {
            match c {
                b'T' => align.insert(StageAlign::TOP),
                b'B' => align.insert(StageAlign::BOTTOM),
                b'L' => align.insert(StageAlign::LEFT),
                b'R' => align.insert(StageAlign::RIGHT),
                _ => (),
            }
        }
        self.stage_align = align;
    }

    #[wasm_bindgen(js_name = "setForceAlign")]
    pub fn set_force_align(&mut self, value: bool) {
        self.force_align = value;
    }

    #[wasm_bindgen(js_name = "setQuality")]
    pub fn set_quality(&mut self, value: &str) {
        self.quality = match value {
            "low" => StageQuality::Low,
            "medium" => StageQuality::Medium,
            "high" => StageQuality::High,
            "best" => StageQuality::Best,
            "8x8" => StageQuality::High8x8,
            "8x8linear" => StageQuality::High8x8Linear,
            "16x16" => StageQuality::High16x16,
            "16x16linear" => StageQuality::High16x16Linear,
            _ => return,
        };
    }

    #[wasm_bindgen(js_name = "setScale")]
    pub fn set_scale(&mut self, value: &str) {
        self.scale = match value {
            "exactfit" => StageScaleMode::ExactFit,
            "noborder" => StageScaleMode::NoBorder,
            "noscale" => StageScaleMode::NoScale,
            "showall" => StageScaleMode::ShowAll,
            _ => return,
        };
    }

    #[wasm_bindgen(js_name = "setForceScale")]
    pub fn set_force_scale(&mut self, value: bool) {
        self.force_scale = value;
    }

    #[wasm_bindgen(js_name = "setFrameRate")]
    pub fn set_frame_rate(&mut self, value: Option<f64>) {
        self.frame_rate = value;
    }

    #[wasm_bindgen(js_name = "setWmode")]
    pub fn set_wmode(&mut self, value: Option<String>) {
        self.wmode = value;
    }

    #[wasm_bindgen(js_name = "setLogLevel")]
    pub fn set_log_level(&mut self, value: &str) {
        if let Ok(level) = tracing::Level::from_str(value) {
            self.log_level = level;
        }
    }

    #[wasm_bindgen(js_name = "setMaxExecutionDuration")]
    pub fn set_max_execution_duration(&mut self, value: f64) {
        self.max_execution_duration = Duration::from_secs_f64(value);
    }

    #[wasm_bindgen(js_name = "setPlayerVersion")]
    pub fn set_player_version(&mut self, value: Option<u8>) {
        self.player_version = value;
    }

    #[wasm_bindgen(js_name = "setPreferredRenderer")]
    pub fn set_preferred_renderer(&mut self, value: Option<String>) {
        self.preferred_renderer = value;
    }

    #[wasm_bindgen(js_name = "setOpenUrlMode")]
    pub fn set_open_url_mode(&mut self, value: &str) {
        self.open_url_mode = match value {
            "allow" => OpenURLMode::Allow,
            "confirm" => OpenURLMode::Confirm,
            "deny" => OpenURLMode::Deny,
            _ => return,
        };
    }

    #[wasm_bindgen(js_name = "setAllowNetworking")]
    pub fn set_allow_networking(&mut self, value: &str) {
        self.allow_networking = match value {
            "all" => NetworkingAccessMode::All,
            "internal" => NetworkingAccessMode::Internal,
            "none" => NetworkingAccessMode::None,
            _ => return,
        };
    }

    #[wasm_bindgen(js_name = "addSocketProxy")]
    pub fn add_socket_proxy(&mut self, host: String, port: u16, proxy_url: String) {
        self.socket_proxy.push(SocketProxy {
            host,
            port,
            proxy_url,
        })
    }

    #[wasm_bindgen(js_name = "setCredentialAllowList")]
    pub fn set_credential_allow_list(&mut self, value: Vec<String>) {
        self.credential_allow_list = value;
    }

    #[wasm_bindgen(js_name = "setPlayerRuntime")]
    pub fn set_player_runtime(&mut self, value: &str) {
        self.player_runtime = match value {
            "air" => PlayerRuntime::AIR,
            "flashPlayer" => PlayerRuntime::FlashPlayer,
            _ => return,
        };
    }

    #[wasm_bindgen(js_name = "setVolume")]
    pub fn set_volume(&mut self, value: f32) {
        self.volume = value;
    }

    #[wasm_bindgen(js_name = "addFont")]
    pub fn add_font(&mut self, font_name: String, data: Vec<u8>) {
        self.custom_fonts.push((font_name, data))
    }

    #[wasm_bindgen(js_name = "setDefaultFont")]
    pub fn set_default_font(&mut self, default_name: &str, fonts: Vec<JsValue>) {
        let default = match default_name {
            "sans" => DefaultFont::Sans,
            "serif" => DefaultFont::Serif,
            "typewriter" => DefaultFont::Typewriter,
            "japaneseGothic" => DefaultFont::JapaneseGothic,
            "japaneseGothicMono" => DefaultFont::JapaneseGothicMono,
            "japaneseMincho" => DefaultFont::JapaneseMincho,
            _ => return,
        };
        self.default_fonts.insert(
            default,
            fonts
                .into_iter()
                .flat_map(|value| value.as_string())
                .collect(),
        );
    }

    // TODO: This should be split into two methods that either load url or load data
    // Right now, that's done immediately afterwards in TS
    pub async fn build(&self, parent: HtmlElement, js_player: JavascriptPlayer) -> Promise {
        let copy = self.clone();
        wasm_bindgen_futures::future_to_promise(async move {
            if RUFFLE_GLOBAL_PANIC.is_completed() {
                // If an actual panic happened, then we can't trust the state it left us in.
                // Prevent future players from loading so that they can inform the user about the error.
                return Err("Ruffle is panicking!".into());
            }

            let ruffle = RuffleHandle::new_internal(parent, js_player, copy)
                .await
                .map_err(|err| JsValue::from(format!("Error creating player: {}", err)))?;
            Ok(JsValue::from(ruffle))
        })
    }
}

impl RuffleInstanceBuilder {
    pub fn setup_fonts(&self, player: &mut Player) {
        for (font_name, bytes) in &self.custom_fonts {
            let bytes_slice = &bytes[..];
            if let Ok(face) = ttf_parser::Face::parse(bytes_slice, 0) {
                tracing::debug!("Loading font {font_name} as TTF/OTF/TTC/OTC font");

                // Check if font collection
                let number_of_fonts = ttf_parser::fonts_in_collection(bytes_slice).unwrap_or(1u32);

                Self::register_ttf_face_by_name(font_name, bytes.clone(), face, 0, player);

                // Register all remaining fonts in the collection if it is a collection
                for i in 1u32..number_of_fonts {
                    if let Ok(face) = ttf_parser::Face::parse(bytes_slice, i) {
                        Self::register_ttf_face_by_name(font_name, bytes.clone(), face, i, player);
                    } else {
                        tracing::warn!(
                            "Failed to parse font {font_name} at index {i} in font collection"
                        );
                    }
                }
            } else {
                tracing::debug!("Loading font {font_name} as SWF font");
                if let Ok(swf_stream) = swf::decompress_swf(&bytes[..]) {
                    if let Ok(swf) = swf::parse_swf(&swf_stream) {
                        let encoding = swf::SwfStr::encoding_for_version(swf.header.version());
                        for tag in swf.tags {
                            match tag {
                                swf::Tag::DefineFont(_font) => {
                                    tracing::warn!("DefineFont1 tag is not yet supported by Ruffle, inside font swf {font_name}");
                                }
                                swf::Tag::DefineFont2(font) => {
                                    tracing::debug!(
                                        "Loaded font {} from font swf {font_name}",
                                        font.name.to_str_lossy(encoding)
                                    );
                                    player.register_device_font(FontDefinition::SwfTag(
                                        *font, encoding,
                                    ));
                                }
                                swf::Tag::DefineFont4(font) => {
                                    let name = font.name.to_str_lossy(encoding);
                                    if let Some(data) = font.data {
                                        tracing::debug!(
                                            "Loaded font {name} from font swf {font_name}"
                                        );
                                        player.register_device_font(FontDefinition::FontFile {
                                            name: name.to_string(),
                                            is_bold: font.is_bold,
                                            is_italic: font.is_italic,
                                            data: data.to_vec(),
                                            index: 0,
                                        })
                                    } else {
                                        tracing::warn!(
                                            "Font {name} from font swf {font_name} contains no data"
                                        );
                                    }
                                }
                                _ => {}
                            }
                        }
                        continue;
                    }
                }
                tracing::warn!("Font source {font_name} was not recognised (not a valid SWF?)");
            }
        }

        for (default, names) in &self.default_fonts {
            player.set_default_font(*default, names.clone());
        }
    }

    #[inline]
    fn register_ttf_face_by_name(
        url: &String,
        bytes: Vec<u8>,
        face: ttf_parser::Face<'_>,
        index: u32,
        player: &mut Player,
    ) {
        let full_name = face
            .names()
            .into_iter()
            .find(|name| name.name_id == ttf_parser::name_id::FULL_NAME)
            .and_then(|name| name.to_string());

        let name = if let Some(full_name) = full_name {
            full_name
        } else {
            tracing::warn!("Font {url} at index {index} has no full name, using URL as font name");
            url.to_string()
        };

        player.register_device_font(FontDefinition::FontFile {
            name: name.to_string(),
            is_bold: face.is_bold(),
            is_italic: face.is_italic(),
            data: bytes,
            index,
        });
    }

    pub fn create_log_subscriber(&self) -> Arc<Layered<WASMLayer, Registry>> {
        let layer = WASMLayer::new(
            WASMLayerConfigBuilder::new()
                .set_report_logs_in_timings(cfg!(feature = "profiling"))
                .set_max_level(self.log_level)
                .build(),
        );
        Arc::new(tracing_subscriber::registry().with(layer))
    }

    pub async fn create_renderer(
        &self,
    ) -> Result<(Box<dyn RenderBackend>, HtmlCanvasElement), Box<dyn Error>> {
        let window = web_sys::window().ok_or("Expected window")?;
        let document = window.document().ok_or("Expected document")?;
        #[cfg(not(any(
            feature = "canvas",
            feature = "webgl",
            feature = "webgpu",
            feature = "wgpu-webgl"
        )))]
        std::compile_error!("You must enable one of the render backend features (e.g., webgl).");

        let _is_transparent = self.wmode.as_deref() == Some("transparent");

        let mut renderer_list = vec!["wgpu-webgl", "webgpu", "webgl", "canvas"];
        if let Some(preferred_renderer) = &self.preferred_renderer {
            if let Some(pos) = renderer_list.iter().position(|&r| r == preferred_renderer) {
                renderer_list.remove(pos);
                renderer_list.insert(0, preferred_renderer.as_str());
            } else {
                tracing::error!("Unrecognized renderer name: {}", preferred_renderer);
            }
        }

        // Try to create a backend, falling through to the next backend on failure.
        // We must recreate the canvas each attempt, as only a single context may be created per canvas
        // with `getContext`.
        for renderer in renderer_list {
            match renderer {
                #[cfg(all(feature = "webgpu", target_family = "wasm"))]
                "webgpu" => {
                    // Check that we have access to WebGPU (navigator.gpu should exist).
                    if web_sys::window()
                        .ok_or(JsValue::FALSE)
                        .and_then(|window| {
                            js_sys::Reflect::has(&window.navigator(), &JsValue::from_str("gpu"))
                        })
                        .unwrap_or_default()
                    {
                        tracing::info!("Creating wgpu webgpu renderer...");
                        let canvas: HtmlCanvasElement = document
                            .create_element("canvas")
                            .into_js_result()?
                            .dyn_into()
                            .map_err(|_| "Expected HtmlCanvasElement")?;

                        match ruffle_render_wgpu::backend::WgpuRenderBackend::for_canvas(
                            canvas.clone(),
                            true,
                        )
                        .await
                        {
                            Ok(renderer) => {
                                return Ok((Box::new(renderer), canvas));
                            }
                            Err(error) => {
                                tracing::error!("Error creating wgpu webgpu renderer: {}", error)
                            }
                        }
                    }
                }
                #[cfg(all(feature = "wgpu-webgl", target_family = "wasm"))]
                "wgpu-webgl" => {
                    tracing::info!("Creating wgpu webgl renderer...");
                    let canvas: HtmlCanvasElement = document
                        .create_element("canvas")
                        .into_js_result()?
                        .dyn_into()
                        .map_err(|_| "Expected HtmlCanvasElement")?;

                    match ruffle_render_wgpu::backend::WgpuRenderBackend::for_canvas(
                        canvas.clone(),
                        false,
                    )
                    .await
                    {
                        Ok(renderer) => {
                            return Ok((Box::new(renderer), canvas));
                        }
                        Err(error) => {
                            tracing::error!("Error creating wgpu webgl renderer: {}", error)
                        }
                    }
                }
                #[cfg(feature = "webgl")]
                "webgl" => {
                    tracing::info!("Creating WebGL renderer...");
                    let canvas: HtmlCanvasElement = document
                        .create_element("canvas")
                        .into_js_result()?
                        .dyn_into()
                        .map_err(|_| "Expected HtmlCanvasElement")?;
                    match ruffle_render_webgl::WebGlRenderBackend::new(
                        &canvas,
                        _is_transparent,
                        self.quality,
                    ) {
                        Ok(renderer) => {
                            return Ok((Box::new(renderer), canvas));
                        }
                        Err(error) => {
                            tracing::error!("Error creating WebGL renderer: {}", error)
                        }
                    }
                }
                #[cfg(feature = "canvas")]
                "canvas" => {
                    tracing::info!("Creating Canvas renderer...");
                    let canvas: HtmlCanvasElement = document
                        .create_element("canvas")
                        .into_js_result()?
                        .dyn_into()
                        .map_err(|_| "Expected HtmlCanvasElement")?;
                    match ruffle_render_canvas::WebCanvasRenderBackend::new(
                        &canvas,
                        _is_transparent,
                    ) {
                        Ok(renderer) => {
                            return Ok((Box::new(renderer), canvas));
                        }
                        Err(error) => tracing::error!("Error creating canvas renderer: {}", error),
                    }
                }
                _ => {}
            }
        }
        Err("Unable to create renderer".into())
    }

    pub fn create_audio_backend(
        &self,
        log_subscriber: Arc<Layered<WASMLayer, Registry>>,
    ) -> Box<dyn AudioBackend> {
        if let Ok(audio) = audio::WebAudioBackend::new(log_subscriber.clone()) {
            Box::new(audio)
        } else {
            tracing::error!("Unable to create audio backend. No audio will be played.");
            Box::new(NullAudioBackend::new())
        }
    }

    pub fn create_navigator(
        &self,
        log_subscriber: Arc<Layered<WASMLayer, Registry>>,
    ) -> WebNavigatorBackend {
        WebNavigatorBackend::new(
            self.allow_script_access,
            self.allow_networking,
            self.upgrade_to_https,
            self.base_url.clone(),
            log_subscriber.clone(),
            self.open_url_mode,
            self.socket_proxy.clone(),
            self.credential_allow_list.clone(),
        )
    }

    pub fn create_storage_backend(&self) -> Box<dyn StorageBackend> {
        match web_sys::window().expect("window()").local_storage() {
            Ok(Some(s)) => Box::new(storage::LocalStorageBackend::new(s)),
            err => {
                tracing::warn!("Unable to use localStorage: {:?}\nData will not save.", err);
                Box::new(MemoryStorageBackend::new())
            }
        }
    }

    pub async fn create_player(
        &self,
        js_player: JavascriptPlayer,
        log_subscriber: Arc<Layered<WASMLayer, Registry>>,
    ) -> Result<BuiltPlayer, Box<dyn Error>> {
        let window = web_sys::window().ok_or("Expected window")?;

        let (renderer, canvas) = self.create_renderer().await?;

        let mut builder = PlayerBuilder::new()
            .with_boxed_renderer(renderer)
            .with_boxed_audio(self.create_audio_backend(log_subscriber.clone()))
            .with_navigator(self.create_navigator(log_subscriber.clone()))
            .with_storage(self.create_storage_backend());

        // Create the external interface.
        if self.allow_script_access && self.allow_networking == NetworkingAccessMode::All {
            let interface = Box::new(JavascriptInterface::new(js_player.clone()));
            builder = builder
                .with_external_interface(interface.clone())
                .with_fs_commands(interface);
        }

        let trace_observer = Rc::new(RefCell::new(JsValue::UNDEFINED));
        let core = builder
            .with_log(log_adapter::WebLogBackend::new(trace_observer.clone()))
            .with_ui(ui::WebUiBackend::new(js_player.clone(), &canvas))
            .with_video(ExternalVideoBackend::new())
            .with_letterbox(self.letterbox)
            .with_max_execution_duration(self.max_execution_duration)
            .with_player_version(self.player_version)
            .with_player_runtime(self.player_runtime)
            .with_compatibility_rules(self.compatibility_rules.clone())
            .with_quality(self.quality)
            .with_align(self.stage_align, self.force_align)
            .with_scale_mode(self.scale, self.force_scale)
            .with_frame_rate(self.frame_rate)
            .with_page_url(window.location().href().ok())
            .build();

        let player_weak = Arc::downgrade(&core);

        {
            let mut core = core
                .lock()
                .expect("Failed to lock player after construction");
            core.navigator_mut()
                .downcast_mut::<WebNavigatorBackend>()
                .expect("Expected WebNavigatorBackend")
                .set_player(player_weak);
            // Set config parameters.
            core.set_volume(self.volume);
            core.set_background_color(self.background_color);
            core.set_show_menu(self.show_menu);
            core.set_allow_fullscreen(self.allow_fullscreen);
            core.set_window_mode(self.wmode.as_deref().unwrap_or("window"));
            self.setup_fonts(&mut core);
        }

        Ok(BuiltPlayer {
            core,
            canvas,
            trace_observer,
        })
    }
}

pub struct BuiltPlayer {
    pub core: Arc<Mutex<Player>>,
    pub canvas: HtmlCanvasElement,
    pub trace_observer: Rc<RefCell<JsValue>>,
}
