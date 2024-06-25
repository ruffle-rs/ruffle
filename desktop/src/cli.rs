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

    /// Location to store a wgpu trace output
    #[clap(long)]
    #[cfg(feature = "render_trace")]
    trace_path: Option<std::path::PathBuf>,

    /// Location to store save data for games.
    ///
    /// This option has no effect if `storage` is not `disk`.
    #[clap(long, default_value_os_t=get_default_save_directory())]
    pub save_directory: std::path::PathBuf,

    /// Location of a directory to store Ruffle configuration.
    #[clap(long, default_value_os_t=get_default_config_directory())]
    pub config: std::path::PathBuf,

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
    #[clap(long, default_value = "allow")]
    pub open_url_mode: OpenURLMode,

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

    #[clap(long, action)]
    pub avm_output_json: bool,
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
    let key_code = KeyCode::from_str(&mapping[pos + 1..], true).map_err(|err| {
        anyhow!(
            "Could not parse <key name>: {err}\n  The possible values are: {}",
            to_aliases(KeyCode::value_variants())
        )
    })?;
    Ok((button, key_code))
}

impl Opt {
    #[cfg(feature = "render_trace")]
    pub fn trace_path(&self) -> Option<&Path> {
        if let Some(path) = &self.trace_path {
            let _ = std::fs::create_dir_all(path);
            Some(path)
        } else {
            None
        }
    }

    #[cfg(not(feature = "render_trace"))]
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
