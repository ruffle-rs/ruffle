use std::path::PathBuf;

use clap::Parser;
use ruffle_render_wgpu::clap::{GraphicsBackend, PowerPreference};

#[derive(Parser, Debug, Copy, Clone)]
pub(crate) struct SizeOpt {
    /// The amount to scale the page size with
    #[clap(long = "scale", default_value = "1.0")]
    pub(crate) scale: f64,

    /// Optionally override the output width
    #[clap(long = "width")]
    pub(crate) width: Option<u32>,

    /// Optionally override the output height
    #[clap(long = "height")]
    pub(crate) height: Option<u32>,
}

#[derive(Parser, Debug)]
#[clap(name = "Ruffle Exporter", author, version)]
pub(crate) struct Opt {
    /// The file or directory of files to export frames from
    #[clap(name = "swf")]
    pub(crate) swf: PathBuf,

    /// The file or directory (if multiple frames/files) to store the capture in.
    /// The default value will either be:
    /// - If given one swf and one frame, the name of the swf + ".png"
    /// - If given one swf and multiple frames, the name of the swf as a directory
    /// - If given multiple swfs, this field is required.
    #[clap(name = "output")]
    pub(crate) output_path: Option<PathBuf>,

    /// Number of frames to capture per file
    #[clap(short = 'f', long = "frames", default_value = "1")]
    pub(crate) frames: u32,

    /// Number of frames to skip
    #[clap(long = "skipframes", default_value = "0")]
    pub(crate) skipframes: u32,

    /// Don't show a progress bar
    #[clap(short, long, action)]
    pub(crate) silent: bool,

    #[clap(flatten)]
    pub(crate) size: SizeOpt,

    /// Type of graphics backend to use. Not all options may be supported by your current system.
    /// Default will attempt to pick the most supported graphics backend.
    #[clap(long, short, default_value = "default")]
    pub(crate) graphics: GraphicsBackend,

    /// Power preference for the graphics device used. High power usage tends to prefer dedicated GPUs,
    /// whereas a low power usage tends prefer integrated GPUs.
    #[clap(long, short, default_value = "high")]
    pub(crate) power: PowerPreference,

    /// Skip unsupported movie types (currently AVM 2)
    #[clap(long, action)]
    pub(crate) skip_unsupported: bool,

    /// The amount of frames that are rendered into memory before writing to disk. 0 implies no cache.
    #[clap(long, default_value = "0")]
    pub(crate) frame_cache: u32,
}
