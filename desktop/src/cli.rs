use crate::RUFFLE_VERSION;
use anyhow::Error;
use clap::Parser;
use ruffle_core::backend::navigator::{OpenURLMode, SocketMode};
use ruffle_core::config::Letterbox;
use ruffle_core::{LoadBehavior, PlayerRuntime, StageAlign, StageScaleMode};
use ruffle_render::quality::StageQuality;
use ruffle_render_wgpu::clap::{GraphicsBackend, PowerPreference};
use std::path::Path;
use url::Url;

#[derive(Parser, Debug)]
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
    /// Default will attempt to pick the most supported graphics backend.
    #[clap(long, short, default_value = "default")]
    pub graphics: GraphicsBackend,

    /// Power preference for the graphics device used. High power usage tends to prefer dedicated GPUs,
    /// whereas a low power usage tends prefer integrated GPUs.
    #[clap(long, short, default_value = "high")]
    pub power: PowerPreference,

    /// Width of window in pixels.
    #[clap(long, display_order = 1)]
    pub width: Option<f64>,

    /// Height of window in pixels.
    #[clap(long, display_order = 2)]
    pub height: Option<f64>,

    /// Maximum number of seconds a script can run before scripting is disabled.
    #[clap(long, short, default_value = "Infinity")]
    pub max_execution_duration: f64,

    /// Base directory or URL used to resolve all relative path statements in the SWF file.
    /// The default is the current directory.
    #[clap(long)]
    pub base: Option<Url>,

    /// Default quality of the movie.
    #[clap(long, short, default_value = "high")]
    pub quality: StageQuality,

    /// The alignment of the stage.
    #[clap(long, short)]
    pub align: Option<StageAlign>,

    /// Prevent movies from changing the stage alignment.
    #[clap(long, action)]
    pub force_align: bool,

    /// The scale mode of the stage.
    #[clap(long, short, default_value = "show-all")]
    pub scale: StageScaleMode,

    /// Audio volume as a number between 0 (muted) and 1 (full volume)
    #[clap(long, short, default_value = "1.0")]
    pub volume: f32,

    /// Prevent movies from changing the stage scale mode.
    #[clap(long, action)]
    pub force_scale: bool,

    /// Location to store a wgpu trace output
    #[clap(long)]
    #[cfg(feature = "render_trace")]
    trace_path: Option<std::path::PathBuf>,

    /// Proxy to use when loading movies via URL.
    #[clap(long)]
    pub proxy: Option<Url>,

    /// Add an endpoint (`[host]:[port]`) to the socket whitelist.
    #[clap(long = "socket-allow", number_of_values = 1, action = clap::ArgAction::Append)]
    pub socket_allow: Vec<String>,

    /// Define how to deal with TCP Socket connections.
    #[clap(long = "tcp-connections", default_value = "ask")]
    pub tcp_connections: SocketMode,

    /// Replace all embedded HTTP URLs with HTTPS.
    #[clap(long, action)]
    pub upgrade_to_https: bool,

    /// Start application in fullscreen.
    #[clap(long, action)]
    pub fullscreen: bool,

    #[clap(long, action)]
    pub timedemo: bool,

    #[clap(long, default_value = "streaming")]
    pub load_behavior: LoadBehavior,

    /// Specify how Ruffle should handle areas outside the movie stage.
    #[clap(long, default_value = "on")]
    pub letterbox: Letterbox,

    /// Spoofs the root SWF URL provided to ActionScript.
    #[clap(long, value_parser)]
    pub spoof_url: Option<Url>,

    /// The version of the player to emulate
    #[clap(long)]
    pub player_version: Option<u8>,

    /// The runtime to emulate (Flash Player or Adobe AIR)
    #[clap(long, default_value = "flash-player")]
    pub player_runtime: PlayerRuntime,

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
}

fn parse_movie_file_or_url(path: &str) -> Result<Url, Error> {
    crate::util::parse_url(Path::new(path))
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
