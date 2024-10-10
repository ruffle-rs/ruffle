use crate::preferences::storage::StorageBackend;
use crate::RUFFLE_VERSION;
use anyhow::{anyhow, Error};
use clap::{Parser, ValueEnum};
use ruffle_core::backend::navigator::{OpenURLMode, SocketMode};
use ruffle_core::config::Letterbox;
use ruffle_core::events::{GamepadButton, KeyCode};
use ruffle_core::{LoadBehavior, PlayerRuntime, StageAlign, StageScaleMode};
use ruffle_render::quality::StageQuality;
use ruffle_render_wgpu::clap::{GraphicsBackend, PowerPreference};
use std::path::Path;
use std::str::FromStr;
use std::time::Duration;
use url::Url;

fn get_default_save_directory() -> std::path::PathBuf {
    dirs::data_local_dir()
        .expect("Couldn't find a valid data_local dir")
        .join("ruffle")
        .join("SharedObjects")
}

fn get_default_config_directory() -> std::path::PathBuf {
    dirs::config_local_dir()
        .expect("Couldn't find a valid config_local dir")
        .join("ruffle")
}

fn get_default_cache_directory() -> std::path::PathBuf {
    dirs::cache_dir()
        .expect("Couldn't find a valid cache dir")
        .join("ruffle")
}

#[derive(Parser, Debug, Clone)]
#[clap(
    name = "Ruffle",
    author,
    version = RUFFLE_VERSION,
)]
pub struct Opt {
    /// Path or URL of a Flash movie (SWF) to play.
    #[clap(name = "FILE", value_parser(parse_movie_file_or_url))]
    pub movie_url: Option<Url>,

    /// A "flashvars" parameter to provide to the movie.
    /// This can be repeated multiple times, for example -Pkey=value -Pfoo=bar.
    #[clap(short = 'P', action = clap::ArgAction::Append)]
    parameters: Vec<String>,

    /// Type of graphics backend to use. Not all options may be supported by your current system.
    ///
    /// Default will attempt to pick the most supported graphics backend.
    /// This option temporarily overrides any stored preference.
    #[clap(long, short)]
    pub graphics: Option<GraphicsBackend>,

    /// Power preference for the graphics device used. High power usage tends to prefer dedicated GPUs,
    /// whereas a low power usage tends prefer integrated GPUs.
    ///
    /// Default preference is high (likely dedicated GPU).
    /// This option temporarily overrides any stored preference.
    #[clap(long, short)]
    pub power: Option<PowerPreference>,

    /// GameMode preference.
    ///
    /// This allows enabling or disabling GameMode manually.
    /// When enabled, GameMode will be requested only when a movie is loaded.
    ///
    /// The default preference enables GameMode when power preference is set to high.
    /// This option temporarily overrides any stored preference.
    ///
    /// See <https://github.com/FeralInteractive/gamemode>.
    #[clap(long)]
    #[cfg_attr(not(target_os = "linux"), clap(hide = true))]
    pub gamemode: Option<GameModePreference>,

    /// Type of storage backend to use. This determines where local storage data is saved (e.g. shared objects).
    ///
    /// This option temporarily overrides any stored preference.
    #[clap(long)]
    pub storage: Option<StorageBackend>,

    /// Width of window in pixels.
    #[clap(long, display_order = 1)]
    pub width: Option<f64>,

    /// Height of window in pixels.
    #[clap(long, display_order = 2)]
    pub height: Option<f64>,

    /// Maximum number of seconds a script can run before scripting is disabled.
    #[clap(long, short, value_parser(parse_duration_seconds))]
    pub max_execution_duration: Option<Duration>,

    /// Base directory or URL used to resolve all relative path statements in the SWF file.
    /// The default is the current directory.
    #[clap(long)]
    pub base: Option<Url>,

    /// Default quality of the movie.
    #[clap(long, short)]
    pub quality: Option<StageQuality>,

    /// The alignment of the stage.
    #[clap(long, short, value_parser(parse_align))]
    pub align: Option<StageAlign>,

    /// Prevent movies from changing the stage alignment.
    #[clap(long, action)]
    pub force_align: bool,

    /// The scale mode of the stage.
    #[clap(long, short)]
    pub scale: Option<StageScaleMode>,

    /// Audio volume as a number between 0 (muted) and 1 (full volume). Default is 1.
    #[clap(long, short)]
    pub volume: Option<f32>,

    /// Prevent movies from changing the stage scale mode.
    #[clap(long, action)]
    pub force_scale: bool,

    /// Location to store save data for games.
    ///
    /// This option has no effect if `storage` is not `disk`.
    #[clap(long, default_value_os_t=get_default_save_directory())]
    pub save_directory: std::path::PathBuf,

    /// Location of a directory to store Ruffle configuration.
    #[clap(long, default_value_os_t=get_default_config_directory())]
    pub config: std::path::PathBuf,

    /// Directory that contains non-essential files created by Ruffle.
    ///
    /// This directory can be deleted without affecting functionality.
    #[clap(long, default_value_os_t=get_default_cache_directory())]
    pub cache_directory: std::path::PathBuf,

    /// Proxy to use when loading movies via URL.
    #[clap(long)]
    pub proxy: Option<Url>,

    /// Add an endpoint (`[host]:[port]`) to the socket whitelist.
    #[clap(long = "socket-allow", number_of_values = 1, action = clap::ArgAction::Append)]
    pub socket_allow: Vec<String>,

    /// Define how to deal with TCP Socket connections.
    #[clap(long = "tcp-connections")]
    pub tcp_connections: Option<SocketMode>,

    /// Replace all embedded HTTP URLs with HTTPS.
    #[clap(long, action)]
    pub upgrade_to_https: bool,

    /// Start application in fullscreen.
    #[clap(long, action)]
    pub fullscreen: bool,

    #[clap(long)]
    pub load_behavior: Option<LoadBehavior>,

    /// Specify how Ruffle should handle areas outside the movie stage.
    #[clap(long)]
    pub letterbox: Option<Letterbox>,

    /// Spoofs the root SWF URL provided to ActionScript.
    #[clap(long, value_parser)]
    pub spoof_url: Option<Url>,

    /// Spoofs the HTTP referer header.
    #[clap(long, value_parser)]
    pub referer: Option<Url>,

    /// Spoofs the HTTP cookie header.
    /// This is a string of the form "name1=value1; name2=value2".
    #[clap(long)]
    pub cookie: Option<String>,

    /// The version of the player to emulate
    #[clap(long)]
    pub player_version: Option<u8>,

    /// The runtime to emulate (Flash Player or Adobe AIR)
    #[clap(long)]
    pub player_runtime: Option<PlayerRuntime>,

    /// Set and lock the player's frame rate, overriding the movie's frame rate.
    #[clap(long)]
    pub frame_rate: Option<f64>,

    /// The handling mode of links opening a new website.
    #[clap(long)]
    pub open_url_mode: Option<OpenUrlMode>,

    /// How to handle non-interactive filesystem access.
    #[clap(long, default_value = "ask")]
    pub filesystem_access_mode: FilesystemAccessMode,

    /// Provide a dummy (completely empty) External Interface to the movie.
    /// This may break some movies that expect an External Interface to be functional,
    /// but may fix others that always require an External Interface.
    #[clap(long)]
    pub dummy_external_interface: bool,

    /// Hides the menu bar (the bar at the top of the window).
    #[clap(long)]
    pub no_gui: bool,

    /// Remaps a specific button on a gamepad to a keyboard key.
    /// This can be used to add new gamepad support to existing games, for example mapping
    /// the D-pad to the arrow keys with -B d-pad-up=up -B d-pad-down=down etc.
    ///
    /// A case-insensitive list of supported gamepad-buttons is:
    /// - north, east, south, west
    /// - d-pad-up, d-pad-down, d-pad-left, d-pad-right
    /// - left-trigger, left-trigger2
    /// - right-trigger, right-trigger2
    /// - select, start
    ///
    /// A case-insensitive (non-exhaustive) list of common key-names is:
    /// - a, b, c, ..., z
    /// - up, down, left, right
    /// - return
    /// - space
    /// - comma, semicolon
    /// - key0, key1, ..., key9
    ///
    /// The complete list of supported key-names can be found by using -B start=nonexistent.
    #[clap(
        long,
        short = 'B',
        value_parser(parse_gamepad_button),
        verbatim_doc_comment,
        value_name = "GAMEPAD BUTTON>=<KEY NAME"
    )]
    pub gamepad_button: Vec<(GamepadButton, KeyCode)>,

    /// Disable AVM2 optimizer.
    /// Note that some early opcode conversions
    /// (like inlining constant pool entries) can't be disabled.
    #[clap(long)]
    pub no_avm2_optimizer: bool,
}

fn parse_movie_file_or_url(path: &str) -> Result<Url, Error> {
    crate::util::parse_url(Path::new(path))
}

fn parse_duration_seconds(value: &str) -> Result<Duration, Error> {
    Ok(Duration::from_secs_f64(value.parse()?))
}

fn parse_align(value: &str) -> Result<StageAlign, Error> {
    value
        .parse()
        .map_err(|_| anyhow::anyhow!("Invalid stage alignment"))
}

fn parse_gamepad_button(mapping: &str) -> Result<(GamepadButton, KeyCode), Error> {
    let pos = mapping.find('=').ok_or_else(|| {
        anyhow!("invalid <gamepad button>=<key name>: no `=` found in `{mapping}`")
    })?;

    fn to_aliases<T: ValueEnum>(variants: &[T]) -> String {
        let aliases: Vec<String> = variants
            .iter()
            .map(|variant| {
                variant
                    .to_possible_value()
                    .expect("Must have a PossibleValue")
                    .get_name_and_aliases()
                    .next()
                    .expect("Must have one alias")
                    .to_owned()
            })
            .collect();
        aliases.join(", ")
    }

    let button = GamepadButton::from_str(&mapping[..pos], true).map_err(|err| {
        anyhow!(
            "Could not parse <gamepad button>: {err}\n  The possible values are: {}",
            to_aliases(GamepadButton::value_variants())
        )
    })?;
    let key_code = NamedKeyCode::from_str(&mapping[pos + 1..], true).map_err(|err| {
        anyhow!(
            "Could not parse <key name>: {err}\n  The possible values are: {}",
            to_aliases(NamedKeyCode::value_variants())
        )
    })?;
    Ok((button, KeyCode::from_code(key_code as u32)))
}

impl Opt {
    pub fn trace_path(&self) -> Option<&Path> {
        None
    }

    pub fn parameters(&self) -> impl '_ + Iterator<Item = (String, String)> {
        self.parameters.iter().map(|parameter| {
            let mut split = parameter.splitn(2, '=');
            if let (Some(key), Some(value)) = (split.next(), split.next()) {
                (key.to_owned(), value.to_owned())
            } else {
                (parameter.clone(), "".to_string())
            }
        })
    }
}

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, clap::ValueEnum)]
pub enum GameModePreference {
    #[default]
    Default,
    On,
    Off,
}

impl GameModePreference {
    pub fn as_str(&self) -> Option<&'static str> {
        match self {
            GameModePreference::Default => None,
            GameModePreference::On => Some("on"),
            GameModePreference::Off => Some("off"),
        }
    }
}

impl FromStr for GameModePreference {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "on" => Ok(GameModePreference::On),
            "off" => Ok(GameModePreference::Off),
            _ => Err(()),
        }
    }
}

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, clap::ValueEnum)]
pub enum OpenUrlMode {
    #[default]
    Confirm,
    Allow,
    Deny,
}

impl OpenUrlMode {
    pub fn as_str(&self) -> Option<&'static str> {
        match self {
            OpenUrlMode::Confirm => None,
            OpenUrlMode::Allow => Some("allow"),
            OpenUrlMode::Deny => Some("deny"),
        }
    }
}

impl FromStr for OpenUrlMode {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "allow" => Ok(OpenUrlMode::Allow),
            "deny" => Ok(OpenUrlMode::Deny),
            _ => Err(()),
        }
    }
}

impl From<OpenUrlMode> for OpenURLMode {
    fn from(value: OpenUrlMode) -> Self {
        match value {
            OpenUrlMode::Confirm => OpenURLMode::Confirm,
            OpenUrlMode::Allow => OpenURLMode::Allow,
            OpenUrlMode::Deny => OpenURLMode::Deny,
        }
    }
}

// TODO The following enum exists in order to preserve
//   the behavior of mapping gamepad buttons,
//   We should probably do something smarter here.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, clap::ValueEnum)]
enum NamedKeyCode {
    Unknown = 0,
    MouseLeft = 1,
    MouseRight = 2,
    MouseMiddle = 4,
    Backspace = 8,
    Tab = 9,
    Return = 13,
    Command = 15,
    Shift = 16,
    Control = 17,
    Alt = 18,
    Pause = 19,
    CapsLock = 20,
    Numpad = 21,
    Escape = 27,
    Space = 32,
    PgUp = 33,
    PgDown = 34,
    End = 35,
    Home = 36,
    Left = 37,
    Up = 38,
    Right = 39,
    Down = 40,
    Insert = 45,
    Delete = 46,
    Key0 = 48,
    Key1 = 49,
    Key2 = 50,
    Key3 = 51,
    Key4 = 52,
    Key5 = 53,
    Key6 = 54,
    Key7 = 55,
    Key8 = 56,
    Key9 = 57,
    A = 65,
    B = 66,
    C = 67,
    D = 68,
    E = 69,
    F = 70,
    G = 71,
    H = 72,
    I = 73,
    J = 74,
    K = 75,
    L = 76,
    M = 77,
    N = 78,
    O = 79,
    P = 80,
    Q = 81,
    R = 82,
    S = 83,
    T = 84,
    U = 85,
    V = 86,
    W = 87,
    X = 88,
    Y = 89,
    Z = 90,
    Numpad0 = 96,
    Numpad1 = 97,
    Numpad2 = 98,
    Numpad3 = 99,
    Numpad4 = 100,
    Numpad5 = 101,
    Numpad6 = 102,
    Numpad7 = 103,
    Numpad8 = 104,
    Numpad9 = 105,
    Multiply = 106,
    Plus = 107,
    NumpadEnter = 108,
    NumpadMinus = 109,
    NumpadPeriod = 110,
    NumpadSlash = 111,
    F1 = 112,
    F2 = 113,
    F3 = 114,
    F4 = 115,
    F5 = 116,
    F6 = 117,
    F7 = 118,
    F8 = 119,
    F9 = 120,
    F10 = 121,
    F11 = 122,
    F12 = 123,
    F13 = 124,
    F14 = 125,
    F15 = 126,
    F16 = 127,
    F17 = 128,
    F18 = 129,
    F19 = 130,
    F20 = 131,
    F21 = 132,
    F22 = 133,
    F23 = 134,
    F24 = 135,
    NumLock = 144,
    ScrollLock = 145,
    Semicolon = 186,
    Equals = 187,
    Comma = 188,
    Minus = 189,
    Period = 190,
    Slash = 191,
    Grave = 192,
    LBracket = 219,
    Backslash = 220,
    RBracket = 221,
    Apostrophe = 222,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, clap::ValueEnum)]
pub enum FilesystemAccessMode {
    /// Always allow non-interactive access to the filesystem.
    Allow,

    /// Refuse all non-interactive access to the filesystem.
    Deny,

    /// Ask the user before accessing the filesystem non-interactively.
    Ask,
}
