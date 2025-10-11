use crate::player_ext::PlayerExporterExt;
use anyhow::Result;
use clap::Parser;
use ruffle_core::Player;
use ruffle_render_wgpu::clap::{GraphicsBackend, PowerPreference};
use std::num::NonZeroU32;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

#[derive(Parser, Debug, Copy, Clone)]
pub struct SizeOpt {
    /// The amount to scale the page size with
    #[clap(long = "scale", default_value = "1.0")]
    pub scale: f64,

    /// Optionally override the output width
    #[clap(long = "width")]
    pub width: Option<u32>,

    /// Optionally override the output height
    #[clap(long = "height")]
    pub height: Option<u32>,
}

#[derive(Debug, Clone, Copy)]
pub enum FrameSelection {
    All,
    Count(NonZeroU32),
}

impl FrameSelection {
    pub fn is_single_frame(self) -> bool {
        match self {
            FrameSelection::All => false,
            FrameSelection::Count(n) => n.get() == 1,
        }
    }

    pub fn total_frames(self, player: &Arc<Mutex<Player>>, skipframes: u32) -> u32 {
        match self {
            // TODO Getting frame count from the header won't always work.
            FrameSelection::All => player.header_frames() as u32,
            FrameSelection::Count(n) => n.get() + skipframes,
        }
    }
}

impl FromStr for FrameSelection {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s_lower = s.to_ascii_lowercase();
        if s_lower == "all" {
            Ok(FrameSelection::All)
        } else if let Ok(n) = s.parse::<u32>() {
            let non_zero = NonZeroU32::new(n)
                .ok_or_else(|| "Frame count must be greater than 0".to_string())?;
            Ok(FrameSelection::Count(non_zero))
        } else {
            Err(format!("Invalid value for --frames: {s}"))
        }
    }
}

#[derive(Parser, Debug)]
#[clap(name = "Ruffle Exporter", author, version)]
pub struct Opt {
    /// The file or directory of files to export frames from
    #[clap(name = "swf")]
    pub swf: PathBuf,

    /// The file or directory (if multiple frames/files) to store the capture in.
    /// The default value will either be:
    /// - If given one swf and one frame, the name of the swf + ".png"
    /// - If given one swf and multiple frames, the name of the swf as a directory
    /// - If given multiple swfs, this field is required.
    #[clap(name = "output")]
    pub output_path: Option<PathBuf>,

    /// Number of frames to capture per file. Use 'all' to capture all frames.
    #[clap(short = 'f', long = "frames", default_value = "1")]
    pub frames: FrameSelection,

    /// Number of frames to skip
    #[clap(long = "skipframes", default_value = "0")]
    pub skipframes: u32,

    /// Don't show a progress bar
    #[clap(short, long, action)]
    pub silent: bool,

    #[clap(flatten)]
    pub size: SizeOpt,

    /// Force the main timeline to play, bypassing "Click to Play" buttons and similar restrictions.
    /// This can help automate playback in some SWFs, but may break or alter content that expects user interaction.
    /// Use with caution: enabling this may cause some movies to behave incorrectly.
    #[clap(long)]
    pub force_play: bool,

    /// Type of graphics backend to use. Not all options may be supported by your current system.
    /// Default will attempt to pick the most supported graphics backend.
    #[clap(long, short, default_value = "default")]
    pub graphics: GraphicsBackend,

    /// Power preference for the graphics device used. High power usage tends to prefer dedicated GPUs,
    /// whereas a low power usage tends prefer integrated GPUs.
    #[clap(long, short, default_value = "high")]
    pub power: PowerPreference,

    /// TODO Unused, remove after some time
    #[clap(long, action, hide = true)]
    pub skip_unsupported: bool,
}
