#![deny(clippy::unwrap_used)]
// By default, Windows creates an additional console window for our program.
//
//
// This is silently ignored on non-windows systems.
// See https://docs.microsoft.com/en-us/cpp/build/reference/subsystem?view=msvc-160 for details.
#![windows_subsystem = "windows"]

mod app;
mod audio;
mod cli;
mod custom_event;
mod executor;
mod gui;
mod navigator;
mod storage;
mod task;
mod ui;
mod util;

use anyhow::{anyhow, Context, Error};
use app::App;
use clap::Parser;
use cli::Opt;
use isahc::{config::RedirectPolicy, prelude::*, HttpClient};
use rfd::FileDialog;
use ruffle_core::{tag_utils::SwfMovie, PlayerBuilder, StaticCallstack};
use ruffle_render_wgpu::backend::WgpuRenderBackend;
use std::cell::RefCell;
use std::io::Read;
use std::panic::PanicInfo;
use std::path::{Path, PathBuf};
use std::time::Instant;
use url::Url;

thread_local! {
    static CALLSTACK: RefCell<Option<StaticCallstack>> = RefCell::default();
    static RENDER_INFO: RefCell<Option<String>> = RefCell::default();
    static SWF_INFO: RefCell<Option<String>> = RefCell::default();
}

#[cfg(feature = "tracy")]
#[global_allocator]
static GLOBAL: tracing_tracy::client::ProfiledAllocator<std::alloc::System> =
    tracing_tracy::client::ProfiledAllocator::new(std::alloc::System, 0);

static RUFFLE_VERSION: &str = include_str!(concat!(env!("OUT_DIR"), "/version-info.txt"));

fn parse_url(path: &Path) -> Result<Url, Error> {
    if path.exists() {
        let absolute_path = path.canonicalize().unwrap_or_else(|_| path.to_owned());
        Url::from_file_path(absolute_path)
            .map_err(|_| anyhow!("Path must be absolute and cannot be a URL"))
    } else {
        Url::parse(path.to_str().unwrap_or_default())
            .ok()
            .filter(|url| url.host().is_some() || url.scheme() == "file")
            .ok_or_else(|| anyhow!("Input path is not a file and could not be parsed as a URL."))
    }
}

fn pick_file() -> Option<PathBuf> {
    FileDialog::new()
        .add_filter("Flash Files", &["swf", "spl"])
        .add_filter("All Files", &["*"])
        .set_title("Load a Flash File")
        .pick_file()
}

fn load_movie(url: &Url, opt: &Opt) -> Result<SwfMovie, Error> {
    let mut movie = if url.scheme() == "file" {
        SwfMovie::from_path(
            url.to_file_path()
                .map_err(|_| anyhow!("Invalid swf path"))?,
            None,
        )
        .map_err(|e| anyhow!(e.to_string()))
        .context("Couldn't load swf")?
    } else {
        let proxy = opt.proxy.as_ref().and_then(|url| url.as_str().parse().ok());
        let builder = HttpClient::builder()
            .proxy(proxy)
            .redirect_policy(RedirectPolicy::Follow);
        let client = builder.build().context("Couldn't create HTTP client")?;
        let response = client
            .get(url.to_string())
            .with_context(|| format!("Couldn't load URL {url}"))?;
        let mut buffer: Vec<u8> = Vec::new();
        response
            .into_body()
            .read_to_end(&mut buffer)
            .context("Couldn't read response from server")?;

        SwfMovie::from_data(&buffer, url.to_string(), None)
            .map_err(|e| anyhow!(e.to_string()))
            .context("Couldn't load swf")?
    };

    movie.append_parameters(opt.parameters());

    Ok(movie)
}

fn run_timedemo(opt: Opt) -> Result<(), Error> {
    let path = opt
        .input_path
        .as_ref()
        .ok_or_else(|| anyhow!("Input file necessary for timedemo"))?;
    let movie_url = parse_url(path)?;
    let movie = load_movie(&movie_url, &opt).context("Couldn't load movie")?;
    let movie_frames = Some(movie.num_frames());

    let viewport_width = 1920;
    let viewport_height = 1080;
    let viewport_scale_factor = 1.0;

    let renderer = WgpuRenderBackend::for_offscreen(
        (viewport_width, viewport_height),
        opt.graphics.into(),
        opt.power.into(),
        opt.trace_path(),
    )
    .map_err(|e| anyhow!(e.to_string()))
    .context("Couldn't create wgpu rendering backend")?;

    let mut builder = PlayerBuilder::new();

    if cfg!(feature = "software_video") {
        builder = builder.with_video(ruffle_video_software::backend::SoftwareVideoBackend::new());
    }

    let player = builder
        .with_renderer(renderer)
        .with_movie(movie)
        .with_viewport_dimensions(viewport_width, viewport_height, viewport_scale_factor)
        .with_autoplay(true)
        .build();

    let mut player_lock = player.lock().expect("Cannot reenter");

    println!("Running {}...", path.to_string_lossy());

    let start = Instant::now();
    let mut num_frames = 0;
    const MAX_FRAMES: u32 = 5000;
    while num_frames < MAX_FRAMES && player_lock.current_frame() < movie_frames {
        player_lock.run_frame();
        player_lock.render();
        num_frames += 1;
    }
    let end = Instant::now();
    let duration = end.duration_since(start);

    println!("Ran {num_frames} frames in {}s.", duration.as_secs_f32());

    Ok(())
}

fn init() {
    // When linked with the windows subsystem windows won't automatically attach
    // to the console of the parent process, so we do it explicitly. This fails
    // silently if the parent has no console.
    #[cfg(windows)]
    unsafe {
        use winapi::um::wincon::{AttachConsole, ATTACH_PARENT_PROCESS};
        AttachConsole(ATTACH_PARENT_PROCESS);
    }

    let prev_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        prev_hook(info);
        panic_hook(info);
    }));

    let subscriber = tracing_subscriber::fmt::Subscriber::builder()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .finish();
    #[cfg(feature = "tracy")]
    let subscriber = {
        use tracing_subscriber::layer::SubscriberExt;
        let tracy_subscriber = tracing_tracy::TracyLayer::new();
        subscriber.with(tracy_subscriber)
    };
    tracing::subscriber::set_global_default(subscriber).expect("Couldn't set up global subscriber");
}

fn panic_hook(info: &PanicInfo) {
    CALLSTACK.with(|callstack| {
        if let Some(callstack) = &*callstack.borrow() {
            callstack.avm2(|callstack| println!("AVM2 stack trace: {callstack}"))
        }
    });

    // [NA] Let me just point out that PanicInfo::message() exists but isn't stable and that sucks.
    let panic_text = info.to_string();
    let message = if let Some(text) = panic_text.strip_prefix("panicked at '") {
        let location = info.location().map(|l| l.to_string()).unwrap_or_default();
        if let Some(text) = text.strip_suffix(&format!("', {location}")) {
            text.trim()
        } else {
            text.trim()
        }
    } else {
        panic_text.trim()
    };
    if rfd::MessageDialog::new()
        .set_level(rfd::MessageLevel::Error)
        .set_title("Ruffle")
        .set_description(&format!(
            "Ruffle has encountered a fatal error, this is a bug.\n\n\
            {message}\n\n\
            Please report this to us so that we can fix it. Thank you!\n\
            Pressing Yes will open a browser window."
        ))
        .set_buttons(rfd::MessageButtons::YesNo)
        .show()
    {
        let mut params = vec![
            ("panic_text", info.to_string()),
            ("platform", "Desktop app".to_string()),
            ("operating_system", os_info::get().to_string()),
            ("ruffle_version", RUFFLE_VERSION.to_string()),
        ];
        let mut extra_info = vec![];
        SWF_INFO.with(|i| {
            if let Some(swf_name) = i.take() {
                extra_info.push(format!("Filename: {swf_name}\n"));
                params.push(("title", format!("Crash on {swf_name}")));
            }
        });
        CALLSTACK.with(|callstack| {
            if let Some(callstack) = &*callstack.borrow() {
                callstack.avm2(|callstack| {
                    extra_info.push(format!("### AVM2 Callstack\n```{callstack}\n```\n"));
                });
            }
        });
        RENDER_INFO.with(|i| {
            if let Some(render_info) = i.take() {
                extra_info.push(format!("### Render Info\n{render_info}\n"));
            }
        });
        if !extra_info.is_empty() {
            params.push(("extra_info", extra_info.join("\n")));
        }
        if let Ok(url) = Url::parse_with_params("https://github.com/ruffle-rs/ruffle/issues/new?assignees=&labels=bug&template=crash_report.yml", &params) {
            let _ = webbrowser::open(url.as_str());
        }
    }
}

fn shutdown() {
    // Without explicitly detaching the console cmd won't redraw it's prompt.
    #[cfg(windows)]
    unsafe {
        winapi::um::wincon::FreeConsole();
    }
}

fn main() -> Result<(), Error> {
    init();
    let opt = Opt::parse();
    let result = if opt.timedemo {
        run_timedemo(opt)
    } else {
        App::new(opt).map(|app| app.run())
    };
    #[cfg(windows)]
    if let Err(error) = &result {
        eprintln!("{:?}", error)
    }
    shutdown();
    result
}
