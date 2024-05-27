#![deny(clippy::unwrap_used)]
#![allow(clippy::empty_docs)] //False positive in rustc 1.78 beta

//! Ruffle web frontend.
mod audio;
mod external_interface;
mod input;
mod log_adapter;
mod navigator;
mod storage;
mod ui;

use external_interface::{external_to_js_value, js_to_external_value, JavascriptInterface};
use input::{web_key_to_codepoint, web_to_ruffle_key_code, web_to_ruffle_text_control};
use js_sys::{Error as JsError, Promise, Uint8Array};
use ruffle_core::backend::navigator::OpenURLMode;
use ruffle_core::backend::ui::FontDefinition;
use ruffle_core::compatibility_rules::CompatibilityRules;
use ruffle_core::config::{Letterbox, NetworkingAccessMode};
use ruffle_core::context::UpdateContext;
use ruffle_core::events::{MouseButton, MouseWheelDelta, TextControlCode};
use ruffle_core::tag_utils::SwfMovie;
use ruffle_core::{swf, DefaultFont};
use ruffle_core::{
    Color, Player, PlayerBuilder, PlayerEvent, PlayerRuntime, SandboxType, StageAlign,
    StageScaleMode, StaticCallstack, ViewportDimensions,
};
use ruffle_render::quality::StageQuality;
use ruffle_video_software::backend::SoftwareVideoBackend;
use ruffle_web_common::JsResult;
use serde::{Deserialize, Serialize};
use slotmap::{new_key_type, SlotMap};
use std::rc::Rc;
use std::str::FromStr;
use std::sync::Once;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{cell::RefCell, error::Error, num::NonZeroI32};
use tracing_subscriber::layer::{Layered, SubscriberExt};
use tracing_subscriber::registry::Registry;
use tracing_wasm::{WASMLayer, WASMLayerConfigBuilder};
use url::Url;
use wasm_bindgen::convert::FromWasmAbi;
use wasm_bindgen::prelude::*;
use web_sys::{
    AddEventListenerOptions, ClipboardEvent, Element, Event, EventTarget, HtmlCanvasElement,
    HtmlElement, KeyboardEvent, PointerEvent, WheelEvent, Window,
};

static RUFFLE_GLOBAL_PANIC: Once = Once::new();

new_key_type! {
    /// An opaque handle to a `RuffleInstance` inside the pool.
    ///
    /// This type is exported to JS, and is used to interact with the library.
    #[wasm_bindgen]
    pub struct RuffleHandle;
}

thread_local! {
    /// We store the actual instances of the ruffle core in a static pool.
    /// This gives us a clear boundary between the JS side and Rust side, avoiding
    /// issues with lifetimes and type parameters (which cannot be exported with wasm-bindgen).
    static INSTANCES: RefCell<SlotMap<RuffleHandle, RefCell<RuffleInstance>>> = RefCell::new(SlotMap::with_key());

    static CURRENT_CONTEXT: RefCell<Option<*mut UpdateContext<'static, 'static>>> = const { RefCell::new(None) };
}

type AnimationHandler = Closure<dyn FnMut(f64)>;

struct JsCallback<E> {
    target: EventTarget,
    name: &'static str,
    is_capture: bool,
    closure: Closure<dyn FnMut(E)>,
}

impl<E: FromWasmAbi + 'static> JsCallback<E> {
    pub fn register<T: AsRef<EventTarget>, C: FnMut(E) + 'static>(
        target: &T,
        name: &'static str,
        is_capture: bool,
        closure: C,
    ) -> Self {
        let target = target.as_ref();
        let closure = Closure::new(closure);

        target
            .add_event_listener_with_callback_and_add_event_listener_options(
                name,
                closure.as_ref().unchecked_ref(),
                AddEventListenerOptions::new()
                    .passive(false)
                    .capture(is_capture),
            )
            .warn_on_error();

        Self {
            target: target.clone(),
            name,
            is_capture,
            closure,
        }
    }
}

impl<E> Drop for JsCallback<E> {
    fn drop(&mut self) {
        self.target
            .remove_event_listener_with_callback_and_bool(
                self.name,
                self.closure.as_ref().unchecked_ref(),
                self.is_capture,
            )
            .warn_on_error()
    }
}

struct RuffleInstance {
    core: Arc<Mutex<Player>>,
    callstack: Option<StaticCallstack>,
    js_player: JavascriptPlayer,
    canvas: HtmlCanvasElement,
    canvas_width: i32,
    canvas_height: i32,
    device_pixel_ratio: f64,
    window: Window,
    timestamp: Option<f64>,
    animation_handler: Option<AnimationHandler>, // requestAnimationFrame callback
    animation_handler_id: Option<NonZeroI32>,    // requestAnimationFrame id
    mouse_move_callback: Option<JsCallback<PointerEvent>>,
    mouse_enter_callback: Option<JsCallback<PointerEvent>>,
    mouse_leave_callback: Option<JsCallback<PointerEvent>>,
    mouse_down_callback: Option<JsCallback<PointerEvent>>,
    player_mouse_down_callback: Option<JsCallback<PointerEvent>>,
    window_mouse_down_callback: Option<JsCallback<PointerEvent>>,
    mouse_up_callback: Option<JsCallback<PointerEvent>>,
    mouse_wheel_callback: Option<JsCallback<WheelEvent>>,
    key_down_callback: Option<JsCallback<KeyboardEvent>>,
    key_up_callback: Option<JsCallback<KeyboardEvent>>,
    paste_callback: Option<JsCallback<ClipboardEvent>>,
    unload_callback: Option<JsCallback<Event>>,
    has_focus: bool,
    trace_observer: Rc<RefCell<JsValue>>,
    log_subscriber: Arc<Layered<WASMLayer, Registry>>,
}

#[wasm_bindgen(raw_module = "./ruffle-player")]
extern "C" {
    #[wasm_bindgen(extends = EventTarget)]
    #[derive(Clone)]
    pub type JavascriptPlayer;

    #[wasm_bindgen(method, js_name = "onCallbackAvailable")]
    fn on_callback_available(this: &JavascriptPlayer, name: &str);

    #[wasm_bindgen(method, js_name = "getObjectId")]
    fn get_object_id(this: &JavascriptPlayer) -> Option<String>;

    #[wasm_bindgen(method, catch, js_name = "onFSCommand")]
    fn on_fs_command(this: &JavascriptPlayer, command: &str, args: &str) -> Result<bool, JsValue>;

    #[wasm_bindgen(method)]
    fn panic(this: &JavascriptPlayer, error: &JsError);

    #[wasm_bindgen(method, js_name = "displayRootMovieDownloadFailedMessage")]
    fn display_root_movie_download_failed_message(this: &JavascriptPlayer, invalid_swf: bool);

    #[wasm_bindgen(method, js_name = "displayMessage")]
    fn display_message(this: &JavascriptPlayer, message: &str);

    #[wasm_bindgen(method, getter, js_name = "isFullscreen")]
    fn is_fullscreen(this: &JavascriptPlayer) -> bool;

    #[wasm_bindgen(catch, method, js_name = "setFullscreen")]
    fn set_fullscreen(this: &JavascriptPlayer, is_full: bool) -> Result<(), JsValue>;

    #[wasm_bindgen(method, js_name = "setMetadata")]
    fn set_metadata(this: &JavascriptPlayer, metadata: JsValue);

    #[wasm_bindgen(method, js_name = "openVirtualKeyboard")]
    fn open_virtual_keyboard(this: &JavascriptPlayer);

    #[wasm_bindgen(method, js_name = "isVirtualKeyboardFocused")]
    fn is_virtual_keyboard_focused(this: &JavascriptPlayer) -> bool;

    #[wasm_bindgen(method, js_name = "displayUnsupportedVideo")]
    fn display_unsupported_video(this: &JavascriptPlayer, url: &str);
}

fn deserialize_color<'de, D>(deserializer: D) -> Result<Option<Color>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let color: Option<String> = serde::Deserialize::deserialize(deserializer)?;
    let color = match color {
        Some(color) => color,
        None => return Ok(None),
    };

    // Parse classic HTML hex color (XXXXXX or #XXXXXX), attempting to match browser behavior.
    // Optional leading #.
    let color = color.strip_prefix('#').unwrap_or(&color);

    // Fail if less than 6 digits.
    let color = match color.get(..6) {
        Some(color) => color,
        None => return Ok(None),
    };

    let rgb = color.chars().fold(0, |acc, c| {
        // Each char represents 4-bits. Invalid hex digit is allowed (converts to 0).
        let digit = c.to_digit(16).unwrap_or_default();
        (acc << 4) | digit
    });
    Ok(Some(Color::from_rgb(rgb, 255)))
}

fn deserialize_log_level<'de, D>(deserializer: D) -> Result<tracing::Level, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;
    let value: String = serde::Deserialize::deserialize(deserializer)?;
    tracing::Level::from_str(&value).map_err(Error::custom)
}

fn deserialize_player_runtime<'de, D>(deserializer: D) -> Result<PlayerRuntime, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value: String = serde::Deserialize::deserialize(deserializer)?;
    Ok(match value.as_str() {
        "air" => PlayerRuntime::AIR,
        "flashPlayer" => PlayerRuntime::FlashPlayer,
        _ => Default::default(),
    })
}

fn deserialize_duration<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;

    struct DurationVisitor;

    impl<'de> serde::de::Visitor<'de> for DurationVisitor {
        type Value = Duration;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str(
                "Either a non-negative number (indicating seconds) or a {secs: number, nanos: number}."
            )
        }

        fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
        where
            E: Error,
        {
            Ok(Duration::from_secs_f64(if value < 1 {
                1.0
            } else {
                value as f64
            }))
        }

        fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
        where
            E: Error,
        {
            Ok(Duration::from_secs_f64(if value.is_nan() || value < 1.0 {
                1.0
            } else {
                value
            }))
        }

        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::MapAccess<'de>,
        {
            let mut secs = None;
            let mut nanos = None;
            while let Some(key) = map.next_key::<String>()? {
                let key_s = key.as_str();

                match key_s {
                    "secs" => {
                        if secs.is_some() {
                            return Err(Error::duplicate_field("secs"));
                        }
                        secs = Some(map.next_value()?);
                    }
                    "nanos" => {
                        if nanos.is_some() {
                            return Err(Error::duplicate_field("nanos"));
                        }
                        nanos = Some(map.next_value()?);
                    }
                    _ => return Err(Error::unknown_field(key_s, &["secs", "nanos"])),
                }
            }
            let secs = secs.ok_or_else(|| Error::missing_field("secs"))?;
            let nanos = nanos.ok_or_else(|| Error::missing_field("nanos"))?;
            Ok(Duration::new(secs, nanos))
        }
    }

    deserializer.deserialize_any(DurationVisitor)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SocketProxy {
    host: String,
    port: u16,

    proxy_url: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Config {
    allow_script_access: bool,

    #[serde(deserialize_with = "deserialize_color")]
    background_color: Option<Color>,

    letterbox: Letterbox,

    upgrade_to_https: bool,

    compatibility_rules: bool,

    #[serde(rename = "base")]
    base_url: Option<String>,

    #[serde(rename = "menu")]
    show_menu: bool,

    allow_fullscreen: bool,

    salign: Option<String>,

    force_align: bool,

    quality: Option<String>,

    scale: Option<String>,

    force_scale: bool,

    frame_rate: Option<f64>,

    wmode: Option<String>,

    #[serde(deserialize_with = "deserialize_log_level")]
    log_level: tracing::Level,

    #[serde(deserialize_with = "deserialize_duration")]
    max_execution_duration: Duration,

    player_version: Option<u8>,

    preferred_renderer: Option<String>,

    open_url_mode: OpenURLMode,

    allow_networking: NetworkingAccessMode,

    socket_proxy: Vec<SocketProxy>,

    credential_allow_list: Vec<String>,

    #[serde(deserialize_with = "deserialize_player_runtime")]
    player_runtime: PlayerRuntime,
}

/// Metadata about the playing SWF file to be passed back to JavaScript.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct MovieMetadata {
    width: f64,
    height: f64,
    frame_rate: f32,
    num_frames: u16,
    swf_version: u8,
    background_color: Option<String>,
    is_action_script_3: bool,
    #[serde(rename = "uncompressedLength")]
    uncompressed_len: i32,
}

#[wasm_bindgen]
impl RuffleHandle {
    #[allow(clippy::new_ret_no_self)]
    #[wasm_bindgen(constructor)]
    pub fn new(parent: HtmlElement, js_player: JavascriptPlayer, config: JsValue) -> Promise {
        wasm_bindgen_futures::future_to_promise(async move {
            let config: Config = serde_wasm_bindgen::from_value(config)
                .map_err(|e| format!("Error parsing config: {e}"))?;

            if RUFFLE_GLOBAL_PANIC.is_completed() {
                // If an actual panic happened, then we can't trust the state it left us in.
                // Prevent future players from loading so that they can inform the user about the error.
                return Err("Ruffle is panicking!".into());
            }
            set_panic_hook();

            let ruffle = Self::new_internal(parent, js_player, config)
                .await
                .map_err(|err| JsValue::from(format!("Error creating player: {}", err)))?;
            Ok(JsValue::from(ruffle))
        })
    }

    /// Stream an arbitrary movie file from (presumably) the Internet.
    ///
    /// This method should only be called once per player.
    ///
    /// `parameters` are *extra* parameters to set on the LoaderInfo -
    /// parameters from `movie_url` query parameters will be automatically added.
    pub fn stream_from(&mut self, movie_url: String, parameters: JsValue) -> Result<(), JsValue> {
        let _ = self.with_core_mut(|core| {
            let parameters_to_load = parse_movie_parameters(&parameters);

            let ruffle = *self;
            let on_metadata = move |swf_header: &ruffle_core::swf::HeaderExt| {
                ruffle.on_metadata(swf_header);
            };

            core.fetch_root_movie(movie_url, parameters_to_load, Box::new(on_metadata));
        });
        Ok(())
    }

    /// Play an arbitrary movie on this instance.
    ///
    /// This method should only be called once per player.
    pub fn load_data(
        &mut self,
        swf_data: Uint8Array,
        parameters: JsValue,
        swf_name: String,
    ) -> Result<(), JsValue> {
        let window = web_sys::window().ok_or("Expected window")?;
        let mut url = Url::from_str(&window.location().href()?)
            .map_err(|e| format!("Error creating url: {e}"))?;
        url.set_query(None);
        url.set_fragment(None);
        if let Ok(mut segments) = url.path_segments_mut() {
            segments.pop();
            segments.push(&swf_name);
        }

        let mut movie =
            SwfMovie::from_data(&swf_data.to_vec(), url.to_string(), None).map_err(|e| {
                let _ = self.with_core_mut(|core| {
                    core.ui_mut()
                        .display_root_movie_download_failed_message(true);
                });
                format!("Error loading movie: {e}")
            })?;
        movie.append_parameters(parse_movie_parameters(&parameters));

        self.on_metadata(movie.header());

        let _ = self.with_core_mut(move |core| {
            core.update(|uc| {
                uc.set_root_movie(movie);
            });
        });

        Ok(())
    }

    pub fn play(&mut self) {
        let _ = self.with_core_mut(|core| {
            core.set_is_playing(true);
        });
    }

    pub fn pause(&mut self) {
        let _ = self.with_core_mut(|core| {
            core.set_is_playing(false);
        });
    }

    pub fn is_playing(&mut self) -> bool {
        self.with_core(|core| core.is_playing()).unwrap_or_default()
    }

    pub fn volume(&self) -> f32 {
        self.with_core(|core| core.volume()).unwrap_or_default()
    }

    pub fn set_volume(&mut self, value: f32) {
        let _ = self.with_core_mut(|core| core.set_volume(value));
    }

    pub fn renderer_debug_info(&self) -> JsValue {
        self.with_core(|core| JsValue::from_str(&core.renderer().debug_info()))
            .unwrap_or(JsValue::NULL)
    }

    pub fn renderer_name(&self) -> JsValue {
        self.with_core(|core| JsValue::from_str(core.renderer().name()))
            .unwrap_or(JsValue::NULL)
    }

    // after the context menu is closed, remember to call `clear_custom_menu_items`!
    pub fn prepare_context_menu(&mut self) -> JsValue {
        self.with_core_mut(|core| {
            let info = core.prepare_context_menu();
            serde_wasm_bindgen::to_value(&info).unwrap_or(JsValue::UNDEFINED)
        })
        .unwrap_or(JsValue::UNDEFINED)
    }

    pub fn run_context_menu_callback(&mut self, index: usize) {
        let _ = self.with_core_mut(|core| core.run_context_menu_callback(index));
    }

    pub fn set_fullscreen(&mut self, is_fullscreen: bool) {
        let _ = self.with_core_mut(|core| core.set_fullscreen(is_fullscreen));
    }

    pub fn clear_custom_menu_items(&mut self) {
        let _ = self.with_core_mut(Player::clear_custom_menu_items);
    }

    pub fn destroy(&mut self) {
        // Remove instance from the active list.
        let _ = self.remove_instance();
        // Instance is dropped at this point.
    }

    pub fn add_font(&mut self, font_name: &str, data: Uint8Array) {
        let _ = self.with_core_mut(|core| {
            let bytes: Vec<u8> = data.to_vec();
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
                                core.register_device_font(FontDefinition::SwfTag(*font, encoding));
                            }
                            swf::Tag::DefineFont4(font) => {
                                let name = font.name.to_str_lossy(encoding);
                                if let Some(data) = font.data {
                                    tracing::debug!("Loaded font {name} from font swf {font_name}");
                                    core.register_device_font(FontDefinition::FontFile { name: name.to_string(), is_bold: font.is_bold, is_italic: font.is_bold, data: data.to_vec(), index: 0 })
                                } else {
                                    tracing::warn!("Font {name} from font swf {font_name} contains no data");
                                }
                            }
                            _ => {}
                        }
                    }
                    return;
                }
            }

            tracing::warn!("Font source {font_name} was not recognised (not a valid SWF?)");
        });
    }

    pub fn set_default_font(&mut self, default_name: &str, fonts: Vec<JsValue>) {
        let _ = self.with_core_mut(|core| {
            let default = match default_name {
                "sans" => DefaultFont::Sans,
                "serif" => DefaultFont::Serif,
                "typewriter" => DefaultFont::Typewriter,
                name => {
                    tracing::error!("Unknown default font name '{name}'");
                    return;
                }
            };
            core.set_default_font(
                default,
                fonts
                    .into_iter()
                    .flat_map(|value| value.as_string())
                    .collect(),
            );
        });
    }

    #[allow(clippy::boxed_local)] // for js_bind
    pub fn call_exposed_callback(&self, name: &str, args: Box<[JsValue]>) -> JsValue {
        let args: Vec<_> = args.iter().map(js_to_external_value).collect();

        // Re-entrant callbacks need to return through the hole that was punched through for them
        // We record the context of external functions, and then if we get an internal callback
        // during the same call we'll reuse that.
        // This is unsafe by nature. I don't know any safe way to do this.
        if let Some(context) = CURRENT_CONTEXT.with(|v| *v.borrow()) {
            unsafe {
                if let Some(callback) = (*context).external_interface.get_callback(name) {
                    return external_to_js_value(callback.call(&mut *context, name, args));
                }
            }
        }

        self.with_core_mut(|core| external_to_js_value(core.call_internal_interface(name, args)))
            .unwrap_or(JsValue::UNDEFINED)
    }

    pub fn set_trace_observer(&self, observer: JsValue) {
        let _ = self.with_instance(|instance| {
            *instance.trace_observer.borrow_mut() = observer;
        });
    }

    /// Returns the web AudioContext used by this player.
    /// Returns `None` if the audio backend does not use Web Audio.
    pub fn audio_context(&self) -> Option<web_sys::AudioContext> {
        self.with_core_mut(|core| {
            core.audio()
                .downcast_ref::<audio::WebAudioBackend>()
                .map(|audio| audio.audio_context().clone())
        })
        .unwrap_or_default()
    }

    /// Returns whether the `simd128` target feature was enabled at build time.
    /// This is intended to discriminate between the two WebAssembly module
    /// versions, one of which uses WebAssembly extensions, and the other one
    /// being "vanilla". `simd128` is used as proxy for most extensions, since
    /// no other WebAssembly target feature is exposed to `cfg!`.
    pub fn is_wasm_simd_used() -> bool {
        cfg!(target_feature = "simd128")
    }
}

impl RuffleHandle {
    async fn new_internal(
        parent: HtmlElement,
        js_player: JavascriptPlayer,
        config: Config,
    ) -> Result<Self, Box<dyn Error>> {
        // Redirect Log to Tracing if it isn't already
        let _ = tracing_log::LogTracer::builder()
            // wgpu crates are extremely verbose
            .ignore_crate("wgpu_hal")
            .ignore_crate("wgpu_core")
            .init();
        let log_subscriber = Arc::new(
            Registry::default().with(WASMLayer::new(
                WASMLayerConfigBuilder::new()
                    .set_report_logs_in_timings(cfg!(feature = "profiling"))
                    .set_max_level(config.log_level)
                    .build(),
            )),
        );
        let _subscriber = tracing::subscriber::set_default(log_subscriber.clone());
        let allow_script_access = config.allow_script_access;
        let allow_networking = config.allow_networking;

        let window = web_sys::window().ok_or("Expected window")?;
        let document = window.document().ok_or("Expected document")?;

        let (mut builder, canvas) =
            create_renderer(PlayerBuilder::new(), &document, &config).await?;

        parent
            .append_child(&canvas.clone().into())
            .into_js_result()?;

        if let Ok(audio) = audio::WebAudioBackend::new(log_subscriber.clone()) {
            builder = builder.with_audio(audio);
        } else {
            tracing::error!("Unable to create audio backend. No audio will be played.");
        }
        builder = builder.with_navigator(navigator::WebNavigatorBackend::new(
            allow_script_access,
            allow_networking,
            config.upgrade_to_https,
            config.base_url,
            log_subscriber.clone(),
            config.open_url_mode,
            config.socket_proxy,
            config.credential_allow_list,
        ));

        match window.local_storage() {
            Ok(Some(s)) => {
                builder = builder.with_storage(Box::new(storage::LocalStorageBackend::new(s)));
            }
            err => {
                tracing::warn!("Unable to use localStorage: {:?}\nData will not save.", err);
            }
        };

        let default_quality = if ruffle_web_common::is_mobile_or_tablet() {
            tracing::info!("Running on a mobile device; defaulting to low quality");
            StageQuality::Low
        } else {
            StageQuality::High
        };

        // Create the external interface.
        if allow_script_access && allow_networking == NetworkingAccessMode::All {
            let interface = Box::new(JavascriptInterface::new(js_player.clone()));
            builder = builder
                .with_external_interface(interface.clone())
                .with_fs_commands(interface);
        }

        let trace_observer = Rc::new(RefCell::new(JsValue::UNDEFINED));
        let core = builder
            .with_log(log_adapter::WebLogBackend::new(trace_observer.clone()))
            .with_ui(ui::WebUiBackend::new(js_player.clone(), &canvas))
            .with_video(SoftwareVideoBackend::new())
            .with_letterbox(config.letterbox)
            .with_max_execution_duration(config.max_execution_duration)
            .with_player_version(config.player_version)
            .with_player_runtime(config.player_runtime)
            .with_compatibility_rules(if config.compatibility_rules {
                CompatibilityRules::default()
            } else {
                CompatibilityRules::empty()
            })
            .with_quality(
                config
                    .quality
                    .and_then(|q| {
                        let quality = match q.to_ascii_lowercase().as_str() {
                            "low" => StageQuality::Low,
                            "medium" => StageQuality::Medium,
                            "high" => StageQuality::High,
                            "best" => StageQuality::Best,
                            "8x8" => StageQuality::High8x8,
                            "8x8linear" => StageQuality::High8x8Linear,
                            "16x16" => StageQuality::High16x16,
                            "16x16linear" => StageQuality::High16x16Linear,
                            _ => return None,
                        };

                        Some(quality)
                    })
                    .unwrap_or(default_quality),
            )
            .with_align(
                config
                    .salign
                    .map(|s| {
                        // Chars get converted into flags.
                        // This means "tbbtlbltblbrllrbltlrtbl" is valid, resulting in "TBLR".
                        let mut align = StageAlign::default();
                        for c in s.bytes().map(|c| c.to_ascii_uppercase()) {
                            match c {
                                b'T' => align.insert(StageAlign::TOP),
                                b'B' => align.insert(StageAlign::BOTTOM),
                                b'L' => align.insert(StageAlign::LEFT),
                                b'R' => align.insert(StageAlign::RIGHT),
                                _ => (),
                            }
                        }
                        align
                    })
                    .unwrap_or_default(),
                config.force_align,
            )
            .with_scale_mode(
                config
                    .scale
                    .and_then(|s| {
                        let scale_mode = match s.to_ascii_lowercase().as_str() {
                            "exactfit" => StageScaleMode::ExactFit,
                            "noborder" => StageScaleMode::NoBorder,
                            "noscale" => StageScaleMode::NoScale,
                            "showall" => StageScaleMode::ShowAll,
                            _ => return None,
                        };
                        Some(scale_mode)
                    })
                    .unwrap_or_default(),
                config.force_scale,
            )
            .with_frame_rate(config.frame_rate)
            // FIXME - should this be configurable?
            .with_sandbox_type(SandboxType::Remote)
            .with_page_url(window.location().href().ok())
            .build();

        let mut callstack = None;
        if let Ok(mut core) = core.try_lock() {
            // Set config parameters.
            core.set_background_color(config.background_color);
            core.set_show_menu(config.show_menu);
            core.set_allow_fullscreen(config.allow_fullscreen);
            core.set_window_mode(config.wmode.as_deref().unwrap_or("window"));
            callstack = Some(core.callstack());
        }

        // Create instance.
        let instance = RuffleInstance {
            core,
            callstack,
            js_player: js_player.clone(),
            canvas: canvas.clone(),
            canvas_width: 0, // Initialize canvas width and height to 0 to force an initial canvas resize.
            canvas_height: 0,
            device_pixel_ratio: window.device_pixel_ratio(),
            window: window.clone(),
            animation_handler: None,
            animation_handler_id: None,
            mouse_move_callback: None,
            mouse_enter_callback: None,
            mouse_leave_callback: None,
            mouse_down_callback: None,
            player_mouse_down_callback: None,
            window_mouse_down_callback: None,
            mouse_up_callback: None,
            mouse_wheel_callback: None,
            key_down_callback: None,
            key_up_callback: None,
            paste_callback: None,
            unload_callback: None,
            timestamp: None,
            has_focus: false,
            trace_observer,
            log_subscriber,
        };

        // Prevent touch-scrolling on canvas.
        canvas
            .style()
            .set_property("touch-action", "none")
            .warn_on_error();

        // Register the instance and create the animation frame closure.
        let mut ruffle = Self::add_instance(instance)?;

        // Create the animation frame closure.
        ruffle.with_instance_mut(|instance| {
            instance.animation_handler = Some(Closure::new(move |timestamp| {
                ruffle.tick(timestamp);
            }));

            // Create mouse move handler.
            instance.mouse_move_callback = Some(JsCallback::register(
                &canvas,
                "pointermove",
                false,
                move |js_event: PointerEvent| {
                    let _ = ruffle.with_instance(move |instance| {
                        let event = PlayerEvent::MouseMove {
                            x: f64::from(js_event.offset_x()) * instance.device_pixel_ratio,
                            y: f64::from(js_event.offset_y()) * instance.device_pixel_ratio,
                        };
                        let _ = instance.with_core_mut(|core| {
                            core.handle_event(event);
                        });
                        if instance.has_focus {
                            js_event.prevent_default();
                        }
                    });
                },
            ));

            // Create mouse enter handler.
            instance.mouse_enter_callback = Some(JsCallback::register(
                &canvas,
                "pointerenter",
                false,
                move |_js_event: PointerEvent| {
                    let _ = ruffle.with_instance(move |instance| {
                        let _ = instance.with_core_mut(|core| {
                            core.set_mouse_in_stage(true);
                        });
                    });
                },
            ));

            // Create mouse leave handler.
            instance.mouse_leave_callback = Some(JsCallback::register(
                &canvas,
                "pointerleave",
                false,
                move |_js_event: PointerEvent| {
                    let _ = ruffle.with_instance(move |instance| {
                        let _ = instance.with_core_mut(|core| {
                            core.set_mouse_in_stage(false);
                            core.handle_event(PlayerEvent::MouseLeave);
                        });
                    });
                },
            ));

            // Create mouse down handler.
            instance.mouse_down_callback = Some(JsCallback::register(
                &canvas,
                "pointerdown",
                false,
                move |js_event: PointerEvent| {
                    let _ = ruffle.with_instance(move |instance| {
                        if let Some(target) = js_event.current_target() {
                            let _ = target
                                .unchecked_ref::<Element>()
                                .set_pointer_capture(js_event.pointer_id());
                        }
                        let device_pixel_ratio = instance.device_pixel_ratio;
                        let event = PlayerEvent::MouseDown {
                            x: f64::from(js_event.offset_x()) * device_pixel_ratio,
                            y: f64::from(js_event.offset_y()) * device_pixel_ratio,
                            button: match js_event.button() {
                                0 => MouseButton::Left,
                                1 => MouseButton::Middle,
                                2 => MouseButton::Right,
                                _ => MouseButton::Unknown,
                            },
                        };
                        let _ = instance.with_core_mut(|core| {
                            core.handle_event(event);
                        });

                        js_event.prevent_default();
                    });
                },
            ));

            // Create player mouse down handler.
            instance.player_mouse_down_callback = Some(JsCallback::register(
                &js_player,
                "pointerdown",
                false,
                move |_js_event| {
                    let _ = ruffle.with_instance_mut(|instance| {
                        instance.has_focus = true;
                        // Ensure the parent window gets focus. This is necessary for events
                        // to be received when the player is inside a frame.
                        instance.window.focus().warn_on_error();
                    });
                },
            ));

            // Create window mouse down handler.
            instance.window_mouse_down_callback = Some(JsCallback::register(
                &window,
                "pointerdown",
                true,
                move |_js_event| {
                    let _ = ruffle.with_instance_mut(|instance| {
                        // If we actually clicked on the player, this will be reset to true
                        // after the event bubbles down to the player.
                        instance.has_focus = false;
                    });
                },
            ));

            // Create mouse up handler.
            instance.mouse_up_callback = Some(JsCallback::register(
                &canvas,
                "pointerup",
                false,
                move |js_event: PointerEvent| {
                    let _ = ruffle.with_instance(|instance| {
                        if let Some(target) = js_event.current_target() {
                            let _ = target
                                .unchecked_ref::<Element>()
                                .release_pointer_capture(js_event.pointer_id());
                        }
                        let event = PlayerEvent::MouseUp {
                            x: f64::from(js_event.offset_x()) * instance.device_pixel_ratio,
                            y: f64::from(js_event.offset_y()) * instance.device_pixel_ratio,
                            button: match js_event.button() {
                                0 => MouseButton::Left,
                                1 => MouseButton::Middle,
                                2 => MouseButton::Right,
                                _ => MouseButton::Unknown,
                            },
                        };
                        let _ = instance.with_core_mut(|core| {
                            core.handle_event(event);
                        });

                        if instance.has_focus {
                            js_event.prevent_default();
                        }
                    });
                },
            ));

            // Create mouse wheel handler.
            instance.mouse_wheel_callback = Some(JsCallback::register(
                &canvas,
                "wheel",
                false,
                move |js_event: WheelEvent| {
                    let _ = ruffle.with_instance(|instance| {
                        let delta = match js_event.delta_mode() {
                            WheelEvent::DOM_DELTA_LINE => {
                                MouseWheelDelta::Lines(-js_event.delta_y())
                            }
                            WheelEvent::DOM_DELTA_PIXEL => {
                                MouseWheelDelta::Pixels(-js_event.delta_y())
                            }
                            _ => return,
                        };
                        let _ = instance.with_core_mut(|core| {
                            core.handle_event(PlayerEvent::MouseWheel { delta });
                            if core.should_prevent_scrolling() {
                                js_event.prevent_default();
                            }
                        });
                    });
                },
            ));

            // Create keydown event handler.
            instance.key_down_callback = Some(JsCallback::register(
                &window,
                "keydown",
                false,
                move |js_event: KeyboardEvent| {
                    let _ = ruffle.with_instance(|instance| {
                        if instance.has_focus {
                            let mut paste_event = false;
                            let _ = instance.with_core_mut(|core| {
                                let key_code = web_to_ruffle_key_code(&js_event.code());
                                let key_char = web_key_to_codepoint(&js_event.key());
                                let is_ctrl_cmd = js_event.ctrl_key() || js_event.meta_key();
                                core.handle_event(PlayerEvent::KeyDown { key_code, key_char });

                                if let Some(control_code) = web_to_ruffle_text_control(
                                    &js_event.key(),
                                    is_ctrl_cmd,
                                    js_event.shift_key(),
                                ) {
                                    paste_event = control_code == TextControlCode::Paste;
                                    // The JS paste event fires separately and the clipboard text is not available until then,
                                    // so we need to wait before handling it
                                    if !paste_event {
                                        core.handle_event(PlayerEvent::TextControl {
                                            code: control_code,
                                        });
                                    }
                                } else if let Some(codepoint) = key_char {
                                    core.handle_event(PlayerEvent::TextInput { codepoint });
                                }
                            });

                            // Don't prevent the JS paste event from firing
                            if !paste_event {
                                js_event.prevent_default();
                            }
                        }
                    });
                },
            ));

            instance.paste_callback = Some(JsCallback::register(
                &window,
                "paste",
                false,
                move |js_event: ClipboardEvent| {
                    let _ = ruffle.with_instance(|instance| {
                        if instance.has_focus {
                            let _ = instance.with_core_mut(|core| {
                                let clipboard_content =
                                    if let Some(content) = js_event.clipboard_data() {
                                        content.get_data("text/plain").unwrap_or_default()
                                    } else {
                                        "".into()
                                    };
                                core.ui_mut().set_clipboard_content(clipboard_content);
                                core.handle_event(PlayerEvent::TextControl {
                                    code: TextControlCode::Paste,
                                });
                            });
                            js_event.prevent_default();
                        }
                    });
                },
            ));

            // Create keyup event handler.
            instance.key_up_callback = Some(JsCallback::register(
                &window,
                "keyup",
                false,
                move |js_event: KeyboardEvent| {
                    let _ = ruffle.with_instance(|instance| {
                        if instance.has_focus {
                            let _ = instance.with_core_mut(|core| {
                                let key_code = web_to_ruffle_key_code(&js_event.code());
                                let key_char = web_key_to_codepoint(&js_event.key());
                                core.handle_event(PlayerEvent::KeyUp { key_code, key_char });
                            });
                            js_event.prevent_default();
                        }
                    });
                },
            ));

            instance.unload_callback =
                Some(JsCallback::register(&window, "unload", false, move |_| {
                    let _ = ruffle.with_core_mut(|core| {
                        core.flush_shared_objects();
                    });
                }));
        })?;

        // Set initial timestamp and do initial tick to start animation loop.
        ruffle.tick(0.0);

        Ok(ruffle)
    }

    /// Registers a new Ruffle instance and returns the handle to the instance.
    fn add_instance(instance: RuffleInstance) -> Result<Self, RuffleInstanceError> {
        INSTANCES.try_with(|instances| {
            let mut instances = instances.try_borrow_mut()?;
            let ruffle = instances.insert(RefCell::new(instance));
            Ok(ruffle)
        })?
    }

    /// Unregisters a Ruffle instance, and returns the removed instance.
    fn remove_instance(&self) -> Result<RuffleInstance, RuffleInstanceError> {
        INSTANCES.try_with(|instances| {
            let mut instances = instances.try_borrow_mut()?;
            if let Some(instance) = instances.remove(*self) {
                Ok(instance.into_inner())
            } else {
                Err(RuffleInstanceError::InstanceNotFound)
            }
        })?
    }

    /// Runs the given function on this Ruffle instance.
    fn with_instance<F, O>(&self, f: F) -> Result<O, RuffleInstanceError>
    where
        F: FnOnce(&RuffleInstance) -> O,
    {
        let ret = INSTANCES
            .try_with(|instances| {
                let instances = instances.try_borrow()?;
                if let Some(instance) = instances.get(*self) {
                    let instance = instance.try_borrow()?;
                    let _subscriber =
                        tracing::subscriber::set_default(instance.log_subscriber.clone());
                    Ok(f(&instance))
                } else {
                    Err(RuffleInstanceError::InstanceNotFound)
                }
            })
            .map_err(RuffleInstanceError::from)
            .and_then(std::convert::identity);
        if let Err(e) = &ret {
            tracing::error!("{}", e);
        }
        ret
    }

    /// Runs the given function on this Ruffle instance.
    fn with_instance_mut<F, O>(&self, f: F) -> Result<O, RuffleInstanceError>
    where
        F: FnOnce(&mut RuffleInstance) -> O,
    {
        let ret = INSTANCES
            .try_with(|instances| {
                let instances = instances.try_borrow()?;
                if let Some(instance) = instances.get(*self) {
                    let mut instance = instance.try_borrow_mut()?;
                    let _subscriber =
                        tracing::subscriber::set_default(instance.log_subscriber.clone());
                    Ok(f(&mut instance))
                } else {
                    Err(RuffleInstanceError::InstanceNotFound)
                }
            })
            .map_err(RuffleInstanceError::from)
            .and_then(std::convert::identity);
        if let Err(e) = &ret {
            tracing::error!("{}", e);
        }
        ret
    }

    /// Runs the given function on this instance's `Player`.
    fn with_core<F, O>(&self, f: F) -> Result<O, RuffleInstanceError>
    where
        F: FnOnce(&ruffle_core::Player) -> O,
    {
        let ret = INSTANCES
            .try_with(|instances| {
                let instances = instances.try_borrow()?;
                if let Some(instance) = instances.get(*self) {
                    let instance = instance.try_borrow()?;
                    let _subscriber =
                        tracing::subscriber::set_default(instance.log_subscriber.clone());
                    // This clone lets us drop the instance borrow to avoid potential double-borrows.
                    let core = instance.core.clone();
                    drop(instance);
                    drop(instances);
                    let core = core
                        .try_lock()
                        .map_err(|_| RuffleInstanceError::TryLockError)?;
                    Ok(f(&core))
                } else {
                    Err(RuffleInstanceError::InstanceNotFound)
                }
            })
            .map_err(RuffleInstanceError::from)
            .and_then(std::convert::identity);
        if let Err(e) = &ret {
            tracing::error!("{}", e);
        }
        ret
    }

    /// Runs the given function on this instance's `Player`.
    fn with_core_mut<F, O>(&self, f: F) -> Result<O, RuffleInstanceError>
    where
        F: FnOnce(&mut ruffle_core::Player) -> O,
    {
        let ret = INSTANCES
            .try_with(|instances| {
                let instances = instances.try_borrow()?;
                if let Some(instance) = instances.get(*self) {
                    let instance = instance.try_borrow()?;
                    let _subscriber =
                        tracing::subscriber::set_default(instance.log_subscriber.clone());
                    // This clone lets us drop the instance to avoid potential double-borrows.
                    let core = instance.core.clone();
                    drop(instance);
                    drop(instances);
                    let mut core = core
                        .try_lock()
                        .map_err(|_| RuffleInstanceError::TryLockError)?;
                    Ok(f(&mut core))
                } else {
                    Err(RuffleInstanceError::InstanceNotFound)
                }
            })
            .map_err(RuffleInstanceError::from)
            .and_then(std::convert::identity);
        if let Err(e) = &ret {
            tracing::error!("{}", e);
        }
        ret
    }

    fn tick(&mut self, timestamp: f64) {
        let mut dt = 0.0;
        let mut new_dimensions = None;
        let _ = self.with_instance_mut(|instance| {
            // Check for canvas resize.
            let canvas_width = instance.canvas.client_width();
            let canvas_height = instance.canvas.client_height();
            let device_pixel_ratio = instance.window.device_pixel_ratio(); // Changes via user zooming.
            if instance.canvas_width != canvas_width
                || instance.canvas_height != canvas_height
                || (instance.device_pixel_ratio - device_pixel_ratio).abs() >= f64::EPSILON
            {
                // If a canvas resizes, its drawing context will get scaled. You must reset
                // the width and height attributes of the canvas element to recreate the context.
                // (NOT the CSS width/height!)
                instance.canvas_width = canvas_width;
                instance.canvas_height = canvas_height;
                instance.device_pixel_ratio = device_pixel_ratio;

                // The actual viewport is scaled by DPI, bigger than CSS pixels.
                let viewport_width = (f64::from(canvas_width) * device_pixel_ratio) as u32;
                let viewport_height = (f64::from(canvas_height) * device_pixel_ratio) as u32;

                new_dimensions = Some((
                    instance.canvas.clone(),
                    viewport_width,
                    viewport_height,
                    device_pixel_ratio,
                ));
            }

            // Request next animation frame.
            if let Some(handler) = &instance.animation_handler {
                let id = instance
                    .window
                    .request_animation_frame(handler.as_ref().unchecked_ref())
                    .unwrap_or_default();
                instance.animation_handler_id = NonZeroI32::new(id);
            } else {
                instance.animation_handler_id = None;
            }

            // Calculate the elapsed time since the last tick.
            dt = instance
                .timestamp
                .map_or(0.0, |prev_timestamp| timestamp - prev_timestamp);

            // Store the timestamp of the last tick.
            instance.timestamp = Some(timestamp);
        });

        // Tick the Ruffle core.
        let _ = self.with_core_mut(|core| {
            if let Some((ref canvas, viewport_width, viewport_height, device_pixel_ratio)) =
                new_dimensions
            {
                canvas.set_width(viewport_width);
                canvas.set_height(viewport_height);

                core.set_viewport_dimensions(ViewportDimensions {
                    width: viewport_width,
                    height: viewport_height,
                    scale_factor: device_pixel_ratio,
                });
            }

            core.tick(dt);

            // Render if the core signals a new frame, or if we resized.
            if core.needs_render() || new_dimensions.is_some() {
                core.render();
            }
        });
    }

    fn on_metadata(&self, swf_header: &ruffle_core::swf::HeaderExt) {
        let _ = self.with_instance(|instance| {
            // Convert the background color to an HTML hex color ("#FFFFFF").
            let background_color = swf_header
                .background_color()
                .map(|color| format!("#{:06X}", color.to_rgb()));
            let metadata = MovieMetadata {
                width: swf_header.stage_size().width().to_pixels(),
                height: swf_header.stage_size().height().to_pixels(),
                frame_rate: swf_header.frame_rate().to_f32(),
                num_frames: swf_header.num_frames(),
                uncompressed_len: swf_header.uncompressed_len(),
                swf_version: swf_header.version(),
                background_color,
                is_action_script_3: swf_header.is_action_script_3(),
            };

            if let Ok(value) = serde_wasm_bindgen::to_value(&metadata) {
                instance.js_player.set_metadata(value);
            }
        });
    }
}

impl RuffleInstance {
    #[allow(dead_code)]
    fn with_core<F, O>(&self, f: F) -> Result<O, RuffleInstanceError>
    where
        F: FnOnce(&ruffle_core::Player) -> O,
    {
        let ret = self
            .core
            .try_lock()
            .map(|core| f(&core))
            .map_err(|_| RuffleInstanceError::TryLockError);
        if let Err(e) = &ret {
            tracing::error!("{}", e);
        }
        ret
    }

    fn with_core_mut<F, O>(&self, f: F) -> Result<O, RuffleInstanceError>
    where
        F: FnOnce(&mut ruffle_core::Player) -> O,
    {
        let ret = self
            .core
            .try_lock()
            .map(|mut core| f(&mut core))
            .map_err(|_| RuffleInstanceError::TryLockError);
        if let Err(e) = &ret {
            tracing::error!("{}", e);
        }
        ret
    }
}

impl Drop for RuffleInstance {
    fn drop(&mut self) {
        self.canvas.remove();

        // Stop all audio playing from the instance.
        let _ = self.with_core_mut(|core| {
            core.audio_mut().stop_all_sounds();
            core.flush_shared_objects();
        });

        // Cancel the animation handler, if it's still active.
        if let Some(id) = self.animation_handler_id {
            self.window
                .cancel_animation_frame(id.into())
                .warn_on_error();
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum RuffleInstanceError {
    #[error("Unable to access INSTANCES threadlocal")]
    ThreadLocalAccessError(#[from] std::thread::AccessError),
    #[error("Unable to mutably borrow Ruffle instance")]
    CannotBorrow(#[from] std::cell::BorrowError),
    #[error("Unable to borrow Ruffle instance")]
    CannotBorrowMut(#[from] std::cell::BorrowMutError),
    #[error("Unable to lock Ruffle core")]
    TryLockError,
    #[error("Ruffle Instance ID does not exist")]
    InstanceNotFound,
}

async fn create_renderer(
    builder: PlayerBuilder,
    document: &web_sys::Document,
    config: &Config,
) -> Result<(PlayerBuilder, HtmlCanvasElement), Box<dyn Error>> {
    #[cfg(not(any(
        feature = "canvas",
        feature = "webgl",
        feature = "webgpu",
        feature = "wgpu-webgl"
    )))]
    std::compile_error!("You must enable one of the render backend features (e.g., webgl).");

    let _is_transparent = config.wmode.as_deref() == Some("transparent");

    let mut renderer_list = vec!["wgpu-webgl", "webgpu", "webgl", "canvas"];
    if let Some(preferred_renderer) = &config.preferred_renderer {
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
                            return Ok((builder.with_renderer(renderer), canvas));
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
                        return Ok((builder.with_renderer(renderer), canvas));
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
                match ruffle_render_webgl::WebGlRenderBackend::new(&canvas, _is_transparent) {
                    Ok(renderer) => {
                        return Ok((builder.with_renderer(renderer), canvas));
                    }
                    Err(error) => tracing::error!("Error creating WebGL renderer: {}", error),
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
                match ruffle_render_canvas::WebCanvasRenderBackend::new(&canvas, _is_transparent) {
                    Ok(renderer) => {
                        return Ok((builder.with_renderer(renderer), canvas));
                    }
                    Err(error) => tracing::error!("Error creating canvas renderer: {}", error),
                }
            }
            _ => {}
        }
    }
    Err("Unable to create renderer".into())
}

fn set_panic_hook() {
    static HOOK_HAS_BEEN_SET: Once = Once::new();
    HOOK_HAS_BEEN_SET.call_once(|| {
        std::panic::set_hook(Box::new(|info| {
            RUFFLE_GLOBAL_PANIC.call_once(|| {
                console_error_panic_hook::hook(info);

                let _ = INSTANCES.try_with(|instances| {
                    let mut players = Vec::new();

                    // We have to be super cautious to not panic here, and not hold any borrows for
                    // longer than we need to.
                    // We grab all of the JsPlayers out from the list and release our hold, as they
                    // may call back to destroy() - which will mutably borrow instances.

                    if let Ok(instances) = instances.try_borrow() {
                        for (_, instance) in instances.iter() {
                            if let Ok((player, Some(callstack))) = instance
                                .try_borrow()
                                .map(|i| (i.js_player.clone(), i.callstack.clone()))
                            {
                                players.push((player, callstack));
                            }
                        }
                    }
                    for (player, callstack) in players {
                        let error = JsError::new(&info.to_string());
                        callstack.avm2(|callstack| {
                            let _ = js_sys::Reflect::set(
                                &error,
                                &"avmStack".into(),
                                &callstack.to_string().into(),
                            );
                        });
                        player.panic(&error);
                    }
                });
            });
        }));
    });
}

fn parse_movie_parameters(input: &JsValue) -> Vec<(String, String)> {
    let mut params = Vec::new();
    if let Ok(keys) = js_sys::Reflect::own_keys(input) {
        for key in keys.values().into_iter().flatten() {
            if let Ok(value) = js_sys::Reflect::get(input, &key) {
                if let (Some(key), Some(value)) = (key.as_string(), value.as_string()) {
                    params.push((key, value))
                }
            }
        }
    }
    params
}
