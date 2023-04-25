#![deny(clippy::unwrap_used)]

//! Ruffle web frontend.
mod audio;
mod log_adapter;
mod navigator;
mod storage;
mod ui;

use generational_arena::{Arena, Index};
use js_sys::{Array, Error as JsError, Function, Object, Promise, Uint8Array};
use ruffle_core::backend::navigator::OpenURLMode;
use ruffle_core::compatibility_rules::CompatibilityRules;
use ruffle_core::config::{Letterbox, NetworkingAccessMode};
use ruffle_core::context::UpdateContext;
use ruffle_core::events::{KeyCode, MouseButton, MouseWheelDelta};
use ruffle_core::external::{
    ExternalInterfaceMethod, ExternalInterfaceProvider, Value as ExternalValue, Value,
};
use ruffle_core::tag_utils::SwfMovie;
use ruffle_core::{
    Color, Player, PlayerBuilder, PlayerEvent, SandboxType, StageScaleMode, StaticCallstack,
    ViewportDimensions,
};
use ruffle_render::quality::StageQuality;
use ruffle_video_software::backend::SoftwareVideoBackend;
use ruffle_web_common::JsResult;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::str::FromStr;
use std::sync::Once;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{cell::RefCell, error::Error, num::NonZeroI32};
use tracing_subscriber::layer::{Layered, SubscriberExt};
use tracing_subscriber::registry::Registry;
use tracing_wasm::{WASMLayer, WASMLayerConfigBuilder};
use url::Url;
use wasm_bindgen::{prelude::*, JsCast, JsValue};
use web_sys::{
    AddEventListenerOptions, Element, Event, EventTarget, HtmlCanvasElement, HtmlElement,
    KeyboardEvent, PointerEvent, WheelEvent, Window,
};

static RUFFLE_GLOBAL_PANIC: Once = Once::new();

thread_local! {
    /// We store the actual instances of the ruffle core in a static pool.
    /// This gives us a clear boundary between the JS side and Rust side, avoiding
    /// issues with lifetimes and type parameters (which cannot be exported with wasm-bindgen).
    static INSTANCES: RefCell<Arena<RefCell<RuffleInstance>>> = RefCell::new(Arena::new());

    static CURRENT_CONTEXT: RefCell<Option<*mut UpdateContext<'static, 'static>>> = RefCell::new(None);
}

type AnimationHandler = Closure<dyn FnMut(f64)>;

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
    #[allow(dead_code)]
    mouse_move_callback: Option<Closure<dyn FnMut(PointerEvent)>>,
    mouse_enter_callback: Option<Closure<dyn FnMut(PointerEvent)>>,
    mouse_leave_callback: Option<Closure<dyn FnMut(PointerEvent)>>,
    mouse_down_callback: Option<Closure<dyn FnMut(PointerEvent)>>,
    player_mouse_down_callback: Option<Closure<dyn FnMut(PointerEvent)>>,
    window_mouse_down_callback: Option<Closure<dyn FnMut(PointerEvent)>>,
    mouse_up_callback: Option<Closure<dyn FnMut(PointerEvent)>>,
    mouse_wheel_callback: Option<Closure<dyn FnMut(WheelEvent)>>,
    key_down_callback: Option<Closure<dyn FnMut(KeyboardEvent)>>,
    key_up_callback: Option<Closure<dyn FnMut(KeyboardEvent)>>,
    unload_callback: Option<Closure<dyn FnMut(Event)>>,
    has_focus: bool,
    trace_observer: Arc<RefCell<JsValue>>,
    log_subscriber: Arc<Layered<WASMLayer, Registry>>,
}

#[wasm_bindgen(raw_module = "./ruffle-player")]
extern "C" {
    #[wasm_bindgen(extends = EventTarget)]
    #[derive(Clone)]
    pub type JavascriptPlayer;

    #[wasm_bindgen(method, js_name = "onCallbackAvailable")]
    fn on_callback_available(this: &JavascriptPlayer, name: &str);

    #[wasm_bindgen(method, catch, js_name = "onFSCommand")]
    fn on_fs_command(this: &JavascriptPlayer, command: &str, args: &str) -> Result<bool, JsValue>;

    #[wasm_bindgen(method)]
    fn panic(this: &JavascriptPlayer, error: &JsError);

    #[wasm_bindgen(method, js_name = "displayUnsupportedMessage")]
    fn display_unsupported_message(this: &JavascriptPlayer);

    #[wasm_bindgen(method, js_name = "displayRootMovieDownloadFailedMessage")]
    fn display_root_movie_download_failed_message(this: &JavascriptPlayer);

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
}

struct JavascriptInterface {
    js_player: JavascriptPlayer,
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

    salign: Option<String>,

    quality: Option<String>,

    scale: Option<String>,

    force_scale: bool,

    frame_rate: Option<f64>,

    wmode: Option<String>,

    warn_on_unsupported_content: bool,

    #[serde(deserialize_with = "deserialize_log_level")]
    log_level: tracing::Level,

    #[serde(deserialize_with = "deserialize_duration")]
    max_execution_duration: Duration,

    player_version: Option<u8>,

    preferred_renderer: Option<String>,

    open_url_mode: OpenURLMode,

    allow_networking: NetworkingAccessMode,
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
    uncompressed_len: u32,
}

/// An opaque handle to a `RuffleInstance` inside the pool.
///
/// This type is exported to JS, and is used to interact with the library.
#[wasm_bindgen]
#[derive(Clone, Copy)]
pub struct Ruffle(Index);

#[wasm_bindgen]
impl Ruffle {
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
            set_panic_handler();

            let ruffle = Ruffle::new_internal(parent, js_player, config)
                .await
                .map_err(|err| JsValue::from(format!("Error creating player: {}", err)))?;
            Ok(JsValue::from(ruffle))
        })
    }

    /// Stream an arbitrary movie file from (presumably) the Internet.
    ///
    /// This method should only be called once per player.
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

        let mut movie = SwfMovie::from_data(&swf_data.to_vec(), url.to_string(), None)
            .map_err(|e| format!("Error loading movie: {e}"))?;
        movie.append_parameters(parse_movie_parameters(&parameters));

        self.on_metadata(movie.header());

        let _ = self.with_core_mut(move |core| {
            core.set_root_movie(movie);
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
        if let Ok(mut instance) = self.remove_instance() {
            instance.canvas.remove();

            // Stop all audio playing from the instance.
            let _ = instance.with_core_mut(|core| {
                core.audio_mut().stop_all_sounds();
                core.flush_shared_objects();
            });

            // Clean up all event listeners.
            if let Some(mouse_move_callback) = instance.mouse_move_callback.take() {
                instance
                    .canvas
                    .remove_event_listener_with_callback(
                        "pointermove",
                        mouse_move_callback.as_ref().unchecked_ref(),
                    )
                    .warn_on_error();
            }
            if let Some(mouse_enter_callback) = instance.mouse_enter_callback.take() {
                instance
                    .canvas
                    .remove_event_listener_with_callback(
                        "pointerenter",
                        mouse_enter_callback.as_ref().unchecked_ref(),
                    )
                    .warn_on_error();
            }
            if let Some(mouse_leave_callback) = instance.mouse_leave_callback.take() {
                instance
                    .canvas
                    .remove_event_listener_with_callback(
                        "pointerleave",
                        mouse_leave_callback.as_ref().unchecked_ref(),
                    )
                    .warn_on_error();
            }
            if let Some(mouse_down_callback) = instance.mouse_down_callback.take() {
                instance
                    .canvas
                    .remove_event_listener_with_callback(
                        "pointerdown",
                        mouse_down_callback.as_ref().unchecked_ref(),
                    )
                    .warn_on_error();
            }
            if let Some(player_mouse_down_callback) = instance.player_mouse_down_callback.take() {
                instance
                    .js_player
                    .remove_event_listener_with_callback(
                        "pointerdown",
                        player_mouse_down_callback.as_ref().unchecked_ref(),
                    )
                    .warn_on_error();
            }
            if let Some(window_mouse_down_callback) = instance.window_mouse_down_callback.take() {
                instance
                    .window
                    .remove_event_listener_with_callback_and_bool(
                        "pointerdown",
                        window_mouse_down_callback.as_ref().unchecked_ref(),
                        true,
                    )
                    .warn_on_error();
            }
            if let Some(mouse_up_callback) = instance.mouse_up_callback.take() {
                instance
                    .canvas
                    .remove_event_listener_with_callback(
                        "pointerup",
                        mouse_up_callback.as_ref().unchecked_ref(),
                    )
                    .warn_on_error();
            }
            if let Some(mouse_wheel_callback) = instance.mouse_wheel_callback.take() {
                instance
                    .canvas
                    .remove_event_listener_with_callback(
                        "wheel",
                        mouse_wheel_callback.as_ref().unchecked_ref(),
                    )
                    .warn_on_error();
            }
            if let Some(key_down_callback) = instance.key_down_callback.take() {
                instance
                    .window
                    .remove_event_listener_with_callback(
                        "keydown",
                        key_down_callback.as_ref().unchecked_ref(),
                    )
                    .warn_on_error();
            }
            if let Some(key_up_callback) = instance.key_up_callback.take() {
                instance
                    .window
                    .remove_event_listener_with_callback(
                        "keyup",
                        key_up_callback.as_ref().unchecked_ref(),
                    )
                    .warn_on_error();
            }
            if let Some(unload_callback) = instance.unload_callback.take() {
                instance
                    .window
                    .remove_event_listener_with_callback(
                        "unload",
                        unload_callback.as_ref().unchecked_ref(),
                    )
                    .warn_on_error();
            }

            // Cancel the animation handler, if it's still active.
            if let Some(id) = instance.animation_handler_id {
                instance
                    .window
                    .cancel_animation_frame(id.into())
                    .warn_on_error();
            }
        }

        // Player is dropped at this point.
    }

    #[allow(clippy::boxed_local)] // for js_bind
    pub fn call_exposed_callback(&self, name: &str, args: Box<[JsValue]>) -> JsValue {
        let args: Vec<ExternalValue> = args.iter().map(js_to_external_value).collect();

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

impl Ruffle {
    async fn new_internal(
        parent: HtmlElement,
        js_player: JavascriptPlayer,
        config: Config,
    ) -> Result<Ruffle, Box<dyn Error>> {
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
        ));

        match window.local_storage() {
            Ok(Some(s)) => {
                builder = builder.with_storage(storage::LocalStorageBackend::new(s));
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

        let trace_observer = Arc::new(RefCell::new(JsValue::UNDEFINED));
        let core = builder
            .with_log(log_adapter::WebLogBackend::new(trace_observer.clone()))
            .with_ui(ui::WebUiBackend::new(js_player.clone(), &canvas))
            .with_video(SoftwareVideoBackend::new())
            .with_letterbox(config.letterbox)
            .with_max_execution_duration(config.max_execution_duration)
            .with_warn_on_unsupported_content(config.warn_on_unsupported_content)
            .with_player_version(config.player_version)
            .with_compatibility_rules(if config.compatibility_rules {
                CompatibilityRules::default()
            } else {
                CompatibilityRules::empty()
            })
            .with_quality(
                config
                    .quality
                    .and_then(|q| StageQuality::from_str(&q).ok())
                    .unwrap_or(default_quality),
            )
            .with_scale_mode(
                config
                    .scale
                    .and_then(|s| StageScaleMode::from_str(&s).ok())
                    .unwrap_or(StageScaleMode::ShowAll),
                config.force_scale,
            )
            .with_frame_rate(config.frame_rate)
            // FIXME - should this be configurable?
            .with_sandbox_type(SandboxType::Remote)
            .build();

        let mut callstack = None;
        if let Ok(mut core) = core.try_lock() {
            // Set config parameters.
            core.set_background_color(config.background_color);
            core.set_show_menu(config.show_menu);
            core.set_stage_align(config.salign.as_deref().unwrap_or(""));
            core.set_window_mode(config.wmode.as_deref().unwrap_or("window"));

            // Create the external interface.
            if allow_script_access && allow_networking == NetworkingAccessMode::All {
                core.add_external_interface(Box::new(JavascriptInterface::new(js_player.clone())));
            }
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
        let mut ruffle = Ruffle::add_instance(instance)?;

        // Create the animation frame closure.
        ruffle.with_instance_mut(|instance| {
            instance.animation_handler = Some(Closure::new(move |timestamp| {
                ruffle.tick(timestamp);
            }));

            // Create mouse move handler.
            let mouse_move_callback = Closure::new(move |js_event: PointerEvent| {
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
            });

            canvas
                .add_event_listener_with_callback(
                    "pointermove",
                    mouse_move_callback.as_ref().unchecked_ref(),
                )
                .warn_on_error();
            instance.mouse_move_callback = Some(mouse_move_callback);

            // Create mouse enter handler.
            let mouse_enter_callback = Closure::new(move |_js_event: PointerEvent| {
                let _ = ruffle.with_instance(move |instance| {
                    let _ = instance.with_core_mut(|core| {
                        core.set_mouse_in_stage(true);
                    });
                });
            });

            canvas
                .add_event_listener_with_callback(
                    "pointerenter",
                    mouse_enter_callback.as_ref().unchecked_ref(),
                )
                .warn_on_error();
            instance.mouse_enter_callback = Some(mouse_enter_callback);

            // Create mouse leave handler.
            let mouse_leave_callback = Closure::new(move |_js_event: PointerEvent| {
                let _ = ruffle.with_instance(move |instance| {
                    let _ = instance.with_core_mut(|core| {
                        core.set_mouse_in_stage(false);
                        core.handle_event(PlayerEvent::MouseLeave);
                    });
                });
            });

            canvas
                .add_event_listener_with_callback(
                    "pointerleave",
                    mouse_leave_callback.as_ref().unchecked_ref(),
                )
                .warn_on_error();
            instance.mouse_leave_callback = Some(mouse_leave_callback);

            // Create mouse down handler.
            let mouse_down_callback = Closure::new(move |js_event: PointerEvent| {
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
            });

            canvas
                .add_event_listener_with_callback(
                    "pointerdown",
                    mouse_down_callback.as_ref().unchecked_ref(),
                )
                .warn_on_error();
            instance.mouse_down_callback = Some(mouse_down_callback);

            // Create player mouse down handler.
            let player_mouse_down_callback = Closure::new(move |_js_event| {
                let _ = ruffle.with_instance_mut(|instance| {
                    instance.has_focus = true;
                    // Ensure the parent window gets focus. This is necessary for events
                    // to be received when the player is inside a frame.
                    instance.window.focus().warn_on_error();
                });
            });

            js_player
                .add_event_listener_with_callback(
                    "pointerdown",
                    player_mouse_down_callback.as_ref().unchecked_ref(),
                )
                .warn_on_error();
            instance.player_mouse_down_callback = Some(player_mouse_down_callback);

            // Create window mouse down handler.
            let window_mouse_down_callback = Closure::new(move |_js_event| {
                let _ = ruffle.with_instance_mut(|instance| {
                    // If we actually clicked on the player, this will be reset to true
                    // after the event bubbles down to the player.
                    instance.has_focus = false;
                });
            });

            window
                .add_event_listener_with_callback_and_bool(
                    "pointerdown",
                    window_mouse_down_callback.as_ref().unchecked_ref(),
                    true, // Use capture so this first *before* the player mouse down handler.
                )
                .warn_on_error();
            instance.window_mouse_down_callback = Some(window_mouse_down_callback);

            // Create mouse up handler.
            let mouse_up_callback = Closure::new(move |js_event: PointerEvent| {
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
            });

            canvas
                .add_event_listener_with_callback(
                    "pointerup",
                    mouse_up_callback.as_ref().unchecked_ref(),
                )
                .warn_on_error();
            instance.mouse_up_callback = Some(mouse_up_callback);

            // Create mouse wheel handler.
            let mouse_wheel_callback = Closure::new(move |js_event: WheelEvent| {
                let _ = ruffle.with_instance(|instance| {
                    let delta = match js_event.delta_mode() {
                        WheelEvent::DOM_DELTA_LINE => MouseWheelDelta::Lines(-js_event.delta_y()),
                        WheelEvent::DOM_DELTA_PIXEL => MouseWheelDelta::Pixels(-js_event.delta_y()),
                        _ => return,
                    };
                    let _ = instance.with_core_mut(|core| {
                        core.handle_event(PlayerEvent::MouseWheel { delta });
                        if core.should_prevent_scrolling() {
                            js_event.prevent_default();
                        }
                    });
                });
            });

            canvas
                .add_event_listener_with_callback_and_add_event_listener_options(
                    "wheel",
                    mouse_wheel_callback.as_ref().unchecked_ref(),
                    AddEventListenerOptions::new().passive(false),
                )
                .warn_on_error();
            instance.mouse_wheel_callback = Some(mouse_wheel_callback);

            // Create keydown event handler.
            let key_down_callback = Closure::new(move |js_event: KeyboardEvent| {
                let _ = ruffle.with_instance(|instance| {
                    if instance.has_focus {
                        let _ = instance.with_core_mut(|core| {
                            let key_code = web_to_ruffle_key_code(&js_event.code());
                            let key_char = web_key_to_codepoint(&js_event.key());
                            core.handle_event(PlayerEvent::KeyDown { key_code, key_char });

                            if let Some(codepoint) = key_char {
                                core.handle_event(PlayerEvent::TextInput { codepoint });
                            }
                        });

                        js_event.prevent_default();
                    }
                });
            });

            window
                .add_event_listener_with_callback(
                    "keydown",
                    key_down_callback.as_ref().unchecked_ref(),
                )
                .warn_on_error();
            instance.key_down_callback = Some(key_down_callback);

            // Create keyup event handler.
            let key_up_callback = Closure::new(move |js_event: KeyboardEvent| {
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
            });

            window
                .add_event_listener_with_callback("keyup", key_up_callback.as_ref().unchecked_ref())
                .warn_on_error();
            instance.key_up_callback = Some(key_up_callback);

            let unload_callback = Closure::new(move |_| {
                let _ = ruffle.with_core_mut(|core| {
                    core.flush_shared_objects();
                });
            });

            window
                .add_event_listener_with_callback(
                    "unload",
                    unload_callback.as_ref().unchecked_ref(),
                )
                .warn_on_error();
            instance.unload_callback = Some(unload_callback);
        })?;

        // Set initial timestamp and do initial tick to start animation loop.
        ruffle.tick(0.0);

        Ok(ruffle)
    }

    /// Registers a new Ruffle instance and returns the handle to the instance.
    fn add_instance(instance: RuffleInstance) -> Result<Ruffle, RuffleInstanceError> {
        INSTANCES.try_with(|instances| {
            let mut instances = instances.try_borrow_mut()?;
            let ruffle = Ruffle(instances.insert(RefCell::new(instance)));
            Ok(ruffle)
        })?
    }

    /// Unregisters a Ruffle instance, and returns the removed instance.
    fn remove_instance(&self) -> Result<RuffleInstance, RuffleInstanceError> {
        INSTANCES.try_with(|instances| {
            let mut instances = instances.try_borrow_mut()?;
            if let Some(instance) = instances.remove(self.0) {
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
                if let Some(instance) = instances.get(self.0) {
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
                if let Some(instance) = instances.get(self.0) {
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
                if let Some(instance) = instances.get(self.0) {
                    let instance = instance.try_borrow()?;
                    let _subscriber =
                        tracing::subscriber::set_default(instance.log_subscriber.clone());
                    // This clone lets us drop the instance borrow to avoid potential double-borrows.
                    let core = instance.core.clone();
                    drop(instance);
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
                if let Some(instance) = instances.get(self.0) {
                    let instance = instance.try_borrow()?;
                    let _subscriber =
                        tracing::subscriber::set_default(instance.log_subscriber.clone());
                    // This clone lets us drop the instance to avoid potential double-borrows.
                    let core = instance.core.clone();
                    drop(instance);
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

struct JavascriptMethod {
    this: JsValue,
    function: JsValue,
}

impl ExternalInterfaceMethod for JavascriptMethod {
    fn call(&self, context: &mut UpdateContext<'_, '_>, args: &[ExternalValue]) -> ExternalValue {
        let old_context = CURRENT_CONTEXT.with(|v| {
            v.replace(Some(unsafe {
                std::mem::transmute::<&mut UpdateContext, &mut UpdateContext<'static, 'static>>(
                    context,
                )
            } as *mut UpdateContext))
        });
        let result = if let Some(function) = self.function.dyn_ref::<Function>() {
            let args_array = Array::new();
            for arg in args {
                args_array.push(&external_to_js_value(arg.to_owned()));
            }
            if let Ok(result) = function.apply(&self.this, &args_array) {
                js_to_external_value(&result)
            } else {
                ExternalValue::Undefined
            }
        } else {
            ExternalValue::Undefined
        };
        CURRENT_CONTEXT.with(|v| v.replace(old_context));
        result
    }
}

#[wasm_bindgen(raw_module = "./ruffle-imports")]
extern "C" {
    #[wasm_bindgen(catch, js_name = "getProperty")]
    pub fn get_property(target: &JsValue, key: &JsValue) -> Result<JsValue, JsValue>;
}

impl JavascriptInterface {
    fn new(js_player: JavascriptPlayer) -> Self {
        Self { js_player }
    }

    fn find_method(&self, root: JsValue, name: &str) -> Option<JavascriptMethod> {
        let mut parent = JsValue::UNDEFINED;
        let mut value = root;
        for key in name.split('.') {
            parent = value;
            value = get_property(&parent, &JsValue::from_str(key)).ok()?;
        }
        if value.is_function() {
            Some(JavascriptMethod {
                this: parent,
                function: value,
            })
        } else {
            None
        }
    }
}

impl ExternalInterfaceProvider for JavascriptInterface {
    fn get_method(&self, name: &str) -> Option<Box<dyn ExternalInterfaceMethod>> {
        if let Some(method) = self.find_method(self.js_player.clone().into(), name) {
            return Some(Box::new(method));
        }
        if let Some(window) = web_sys::window() {
            if let Some(method) = self.find_method(window.into(), name) {
                return Some(Box::new(method));
            }
        }

        // Return a dummy method, as `ExternalInterface.call` must return `undefined`, not `null`.
        Some(Box::new(JavascriptMethod {
            this: JsValue::UNDEFINED,
            function: JsValue::UNDEFINED,
        }))
    }

    fn on_callback_available(&self, name: &str) {
        self.js_player.on_callback_available(name);
    }

    fn on_fs_command(&self, command: &str, args: &str) -> bool {
        self.js_player
            .on_fs_command(command, args)
            .unwrap_or_default()
    }
}

fn js_to_external_value(js: &JsValue) -> ExternalValue {
    if let Some(value) = js.as_f64() {
        ExternalValue::Number(value)
    } else if let Some(value) = js.as_string() {
        ExternalValue::String(value)
    } else if let Some(value) = js.as_bool() {
        ExternalValue::Bool(value)
    } else if let Some(array) = js.dyn_ref::<Array>() {
        let values: Vec<_> = array
            .values()
            .into_iter()
            .flatten()
            .map(|v| js_to_external_value(&v))
            .collect();
        ExternalValue::List(values)
    } else if let Some(object) = js.dyn_ref::<Object>() {
        let mut values = BTreeMap::new();
        for entry in Object::entries(object).values() {
            if let Ok(entry) = entry.and_then(|v| v.dyn_into::<Array>()) {
                if let Some(key) = entry.get(0).as_string() {
                    values.insert(key, js_to_external_value(&entry.get(1)));
                }
            }
        }
        ExternalValue::Object(values)
    } else if js.is_null() {
        ExternalValue::Null
    } else {
        ExternalValue::Undefined
    }
}

fn external_to_js_value(external: ExternalValue) -> JsValue {
    match external {
        Value::Undefined => JsValue::UNDEFINED,
        Value::Null => JsValue::NULL,
        Value::Bool(value) => JsValue::from_bool(value),
        Value::Number(value) => JsValue::from_f64(value),
        Value::String(value) => JsValue::from_str(&value),
        Value::Object(object) => {
            let entries = Array::new();
            for (key, value) in object {
                entries.push(&Array::of2(
                    &JsValue::from_str(&key),
                    &external_to_js_value(value),
                ));
            }
            if let Ok(result) = Object::from_entries(&entries) {
                result.into()
            } else {
                JsValue::NULL
            }
        }
        Value::List(values) => {
            let array = Array::new();
            for value in values {
                array.push(&external_to_js_value(value));
            }
            array.into()
        }
    }
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

    let mut renderer_list = vec!["webgpu", "wgpu-webgl", "webgl", "canvas"];
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

                    match ruffle_render_wgpu::backend::WgpuRenderBackend::for_canvas(canvas.clone())
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

                match ruffle_render_wgpu::backend::WgpuRenderBackend::for_canvas(canvas.clone())
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

pub fn set_panic_handler() {
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

/// Convert a web `KeyboardEvent.code` value into a Ruffle `KeyCode`.
/// Return `KeyCode::Unknown` if there is no matching Flash key code.
fn web_to_ruffle_key_code(key_code: &str) -> KeyCode {
    match key_code {
        "Backspace" => KeyCode::Backspace,
        "Tab" => KeyCode::Tab,
        "Enter" => KeyCode::Return,
        "ShiftLeft" | "ShiftRight" => KeyCode::Shift,
        "ControlLeft" | "ControlRight" => KeyCode::Control,
        "AltLeft" | "AltRight" => KeyCode::Alt,
        "CapsLock" => KeyCode::CapsLock,
        "Escape" => KeyCode::Escape,
        "Space" => KeyCode::Space,
        "Digit0" => KeyCode::Key0,
        "Digit1" => KeyCode::Key1,
        "Digit2" => KeyCode::Key2,
        "Digit3" => KeyCode::Key3,
        "Digit4" => KeyCode::Key4,
        "Digit5" => KeyCode::Key5,
        "Digit6" => KeyCode::Key6,
        "Digit7" => KeyCode::Key7,
        "Digit8" => KeyCode::Key8,
        "Digit9" => KeyCode::Key9,
        "KeyA" => KeyCode::A,
        "KeyB" => KeyCode::B,
        "KeyC" => KeyCode::C,
        "KeyD" => KeyCode::D,
        "KeyE" => KeyCode::E,
        "KeyF" => KeyCode::F,
        "KeyG" => KeyCode::G,
        "KeyH" => KeyCode::H,
        "KeyI" => KeyCode::I,
        "KeyJ" => KeyCode::J,
        "KeyK" => KeyCode::K,
        "KeyL" => KeyCode::L,
        "KeyM" => KeyCode::M,
        "KeyN" => KeyCode::N,
        "KeyO" => KeyCode::O,
        "KeyP" => KeyCode::P,
        "KeyQ" => KeyCode::Q,
        "KeyR" => KeyCode::R,
        "KeyS" => KeyCode::S,
        "KeyT" => KeyCode::T,
        "KeyU" => KeyCode::U,
        "KeyV" => KeyCode::V,
        "KeyW" => KeyCode::W,
        "KeyX" => KeyCode::X,
        "KeyY" => KeyCode::Y,
        "KeyZ" => KeyCode::Z,
        "Semicolon" => KeyCode::Semicolon,
        "Equal" => KeyCode::Equals,
        "Comma" => KeyCode::Comma,
        "Minus" => KeyCode::Minus,
        "Period" => KeyCode::Period,
        "Slash" => KeyCode::Slash,
        "Backquote" => KeyCode::Grave,
        "BracketLeft" => KeyCode::LBracket,
        "Backslash" => KeyCode::Backslash,
        "BracketRight" => KeyCode::RBracket,
        "Quote" => KeyCode::Apostrophe,
        "Numpad0" => KeyCode::Numpad0,
        "Numpad1" => KeyCode::Numpad1,
        "Numpad2" => KeyCode::Numpad2,
        "Numpad3" => KeyCode::Numpad3,
        "Numpad4" => KeyCode::Numpad4,
        "Numpad5" => KeyCode::Numpad5,
        "Numpad6" => KeyCode::Numpad6,
        "Numpad7" => KeyCode::Numpad7,
        "Numpad8" => KeyCode::Numpad8,
        "Numpad9" => KeyCode::Numpad9,
        "NumpadMultiply" => KeyCode::Multiply,
        "NumpadAdd" => KeyCode::Plus,
        "NumpadSubtract" => KeyCode::NumpadMinus,
        "NumpadDecimal" => KeyCode::NumpadPeriod,
        "NumpadDivide" => KeyCode::NumpadSlash,
        "NumpadEnter" => KeyCode::Return,
        "PageUp" => KeyCode::PgUp,
        "PageDown" => KeyCode::PgDown,
        "End" => KeyCode::End,
        "Home" => KeyCode::Home,
        "ArrowLeft" => KeyCode::Left,
        "ArrowUp" => KeyCode::Up,
        "ArrowRight" => KeyCode::Right,
        "ArrowDown" => KeyCode::Down,
        "Insert" => KeyCode::Insert,
        "Delete" => KeyCode::Delete,
        "Pause" => KeyCode::Pause,
        "ScrollLock" => KeyCode::ScrollLock,
        "F1" => KeyCode::F1,
        "F2" => KeyCode::F2,
        "F3" => KeyCode::F3,
        "F4" => KeyCode::F4,
        "F5" => KeyCode::F5,
        "F6" => KeyCode::F6,
        "F7" => KeyCode::F7,
        "F8" => KeyCode::F8,
        "F9" => KeyCode::F9,
        "F10" => KeyCode::F10,
        "F11" => KeyCode::F11,
        "F12" => KeyCode::F12,
        _ => KeyCode::Unknown,
    }
}

/// Convert a web `KeyboardEvent.key` value into a character codepoint.
/// Return `None` if they input was not a printable character.
fn web_key_to_codepoint(key: &str) -> Option<char> {
    // TODO: This is a very cheesy way to tell if a `KeyboardEvent.key` is a printable character.
    // Single character strings will be an actual printable char that we can use as text input.
    // All the other special values are multiple characters (e.g. "ArrowLeft").
    // It's probably better to explicitly match on all the variants.
    let mut chars = key.chars();
    let (c1, c2) = (chars.next(), chars.next());
    if c2.is_none() {
        // Single character.
        c1
    } else {
        // Check for special characters.
        match key {
            "Backspace" => Some(8 as char),
            "Delete" => Some(127 as char),
            _ => None,
        }
    }
}
