use crate::{set_panic_hook, JavascriptPlayer, RuffleHandle, SocketProxy, RUFFLE_GLOBAL_PANIC};
use js_sys::Promise;
use ruffle_core::backend::navigator::OpenURLMode;
use ruffle_core::config::{Letterbox, NetworkingAccessMode};
use ruffle_core::{Color, PlayerRuntime, StageAlign, StageScaleMode};
use ruffle_render::quality::StageQuality;
use std::str::FromStr;
use std::time::Duration;
use wasm_bindgen::prelude::*;
use web_sys::HtmlElement;

#[wasm_bindgen(inspectable)]
#[derive(Debug, Clone)]
pub struct RuffleInstanceBuilder {
    pub(crate) allow_script_access: bool,
    pub(crate) background_color: Option<Color>,
    pub(crate) letterbox: Letterbox,
    pub(crate) upgrade_to_https: bool,
    pub(crate) compatibility_rules: bool,
    pub(crate) base_url: Option<String>,
    pub(crate) show_menu: bool,
    pub(crate) allow_fullscreen: bool,
    pub(crate) stage_align: StageAlign,
    pub(crate) force_align: bool,
    pub(crate) quality: Option<StageQuality>,
    pub(crate) scale: Option<StageScaleMode>,
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
    // TODO: Add font related options
    // TODO: Add volume
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
            compatibility_rules: true,
            base_url: None,
            show_menu: true,
            allow_fullscreen: false,
            stage_align: StageAlign::empty(),
            force_align: false,
            quality: None,
            scale: None,
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
        self.compatibility_rules = value;
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
            "low" => Some(StageQuality::Low),
            "medium" => Some(StageQuality::Medium),
            "high" => Some(StageQuality::High),
            "best" => Some(StageQuality::Best),
            "8x8" => Some(StageQuality::High8x8),
            "8x8linear" => Some(StageQuality::High8x8Linear),
            "16x16" => Some(StageQuality::High16x16),
            "16x16linear" => Some(StageQuality::High16x16Linear),
            _ => return,
        };
    }

    #[wasm_bindgen(js_name = "setScale")]
    pub fn set_scale(&mut self, value: &str) {
        self.scale = match value {
            "exactfit" => Some(StageScaleMode::ExactFit),
            "noborder" => Some(StageScaleMode::NoBorder),
            "noscale" => Some(StageScaleMode::NoScale),
            "showall" => Some(StageScaleMode::ShowAll),
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
            set_panic_hook();

            let ruffle = RuffleHandle::new_internal(parent, js_player, copy)
                .await
                .map_err(|err| JsValue::from(format!("Error creating player: {}", err)))?;
            Ok(JsValue::from(ruffle))
        })
    }
}
