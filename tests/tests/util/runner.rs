use crate::util::options::TestOptions;
use crate::util::test::Test;
use anyhow::{anyhow, Result};
use notify::event::ModifyKind;
use notify::Config;
use notify::EventKind;
use notify::RecursiveMode;
use notify::Watcher;
use ruffle_core::backend::log::LogBackend;
use ruffle_core::backend::navigator::{NullExecutor, NullNavigatorBackend};
use ruffle_core::events::MouseButton as RuffleMouseButton;
use ruffle_core::limits::ExecutionLimit;
use ruffle_core::tag_utils::SwfMovie;
use ruffle_core::{Player, PlayerBuilder, PlayerEvent};
use ruffle_input_format::{AutomatedEvent, InputInjector, MouseButton as InputMouseButton};
use std::cell::RefCell;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::time::Instant;
use tempfile::TempDir;

struct TestLogBackend {
    trace_output: Rc<RefCell<String>>,
}

impl TestLogBackend {
    pub fn new(trace_output: Rc<RefCell<String>>) -> Self {
        Self { trace_output }
    }
}

impl LogBackend for TestLogBackend {
    fn avm_trace(&self, message: &str) {
        self.trace_output.borrow_mut().push_str(message);
        self.trace_output.borrow_mut().push('\n');
    }
}

/// Loads an SWF and runs it through the Ruffle core for a number of frames.
/// Tests that the trace output matches the given expected output.
pub fn run_swf(
    test: &Test,
    mut injector: InputInjector,
    before_start: impl FnOnce(Arc<Mutex<Player>>) -> Result<()>,
    before_end: impl FnOnce(Arc<Mutex<Player>>) -> Result<()>,
) -> Result<String> {
    let base_path = Path::new(&test.output_path).parent().unwrap();
    let mut executor = NullExecutor::new();
    let movie = SwfMovie::from_path(&test.swf_path, None).map_err(|e| anyhow!(e.to_string()))?;
    let frame_time = 1000.0 / movie.frame_rate().to_f64();
    let frame_time_duration = Duration::from_millis(frame_time as u64);
    let trace_output = Rc::new(RefCell::new(String::new()));

    let builder = PlayerBuilder::new()
        .with_log(TestLogBackend::new(trace_output.clone()))
        .with_navigator(NullNavigatorBackend::with_base_path(base_path, &executor)?)
        .with_max_execution_duration(Duration::from_secs(300))
        .with_viewport_dimensions(
            movie.width().to_pixels() as u32,
            movie.height().to_pixels() as u32,
            1.0,
        );

    // Test player options may override anything set above
    let player = test
        .options
        .player_options
        .setup(builder, &movie)?
        .with_movie(movie)
        .build();

    before_start(player.clone())?;

    for _ in 0..test.options.num_frames {
        // If requested, ensure that the 'expected' amount of
        // time actually elapses between frames. This is useful for
        // tests that call 'flash.utils.getTimer()' and use
        // 'setInterval'/'flash.utils.Timer'
        //
        // Note that when Ruffle actually runs frames, we can
        // execute frames faster than this in order to 'catch up'
        // if we've fallen behind. However, in order to make regression
        // tests deterministic, we always call 'update_timers' with
        // an elapsed time of 'frame_time'. By sleeping for 'frame_time_duration',
        // we ensure that the result of 'flash.utils.getTimer()' is consistent
        // with timer execution (timers will see an elapsed time of *at least*
        // the requested timer interval).
        if test.options.sleep_to_meet_frame_rate {
            std::thread::sleep(frame_time_duration);
        }

        while !player
            .lock()
            .unwrap()
            .preload(&mut ExecutionLimit::exhausted())
        {}

        player.lock().unwrap().run_frame();
        player.lock().unwrap().update_timers(frame_time);
        executor.run();

        injector.next(|evt, _btns_down| {
            player.lock().unwrap().handle_event(match evt {
                AutomatedEvent::MouseDown { pos, btn } => PlayerEvent::MouseDown {
                    x: pos.0,
                    y: pos.1,
                    button: match btn {
                        InputMouseButton::Left => RuffleMouseButton::Left,
                        InputMouseButton::Middle => RuffleMouseButton::Middle,
                        InputMouseButton::Right => RuffleMouseButton::Right,
                    },
                },
                AutomatedEvent::MouseMove { pos } => PlayerEvent::MouseMove { x: pos.0, y: pos.1 },
                AutomatedEvent::MouseUp { pos, btn } => PlayerEvent::MouseUp {
                    x: pos.0,
                    y: pos.1,
                    button: match btn {
                        InputMouseButton::Left => RuffleMouseButton::Left,
                        InputMouseButton::Middle => RuffleMouseButton::Middle,
                        InputMouseButton::Right => RuffleMouseButton::Right,
                    },
                },
                AutomatedEvent::Wait => unreachable!(),
            });
        });
        // Rendering has side-effects (such as processing 'DisplayObject.scrollRect' updates)
        player.lock().unwrap().render();
    }

    // Render the image to disk
    // FIXME: Determine how we want to compare against on on-disk image
    #[cfg(feature = "imgtests")]
    if let Some(image_comparison) = &test.options.image_comparison {
        if crate::util::environment::WGPU.is_some() {
            use anyhow::Context;
            use ruffle_render_wgpu::backend::WgpuRenderBackend;
            use ruffle_render_wgpu::target::TextureTarget;

            let mut player_lock = player.lock().unwrap();
            player_lock.render();
            let renderer = player_lock
                .renderer_mut()
                .downcast_mut::<WgpuRenderBackend<TextureTarget>>()
                .unwrap();

            // Use straight alpha, since we want to save this as a PNG
            let actual_image = renderer
                .capture_frame(false)
                .expect("Failed to capture image");

            let expected_image_path = base_path.join("expected.png");
            if expected_image_path.is_file() {
                let expected_image = image::open(&expected_image_path)
                    .context("Failed to open expected image")?
                    .into_rgba8();

                image_comparison.test(
                    actual_image,
                    expected_image,
                    base_path,
                    renderer.descriptors().adapter.get_info(),
                )?;
            } else {
                actual_image.save(expected_image_path)?;
            }
        }
    }

    before_end(player)?;

    executor.run();

    let trace = trace_output.borrow().clone();

    #[cfg(feature = "fpcompare")]
    run_flash_player(&test.swf_path, &trace, &test.options);

    Ok(trace)
}

#[allow(dead_code)]
fn run_flash_player(swf_path: &Path, expected_output: &str, options: &TestOptions) {
    if !options.fpcompare {
        return;
    }

    let dir = TempDir::new().unwrap();
    let agent_script_path = PathBuf::new()
        .join(env!("CARGO_MANIFEST_DIR"))
        .join("fprunner")
        .join("agent.js");

    let mm_cfg_path = PathBuf::new()
        .join(env!("CARGO_MANIFEST_DIR"))
        .join("fprunner")
        .join("mm.cfg")
        .to_str()
        .unwrap()
        .to_string();

    let frida_globals = format!("{{\"MM_CFG_PATH\": \"{mm_cfg_path}\"}}");

    let abs_swf_path = std::fs::canonicalize(swf_path).unwrap();

    let fp_path = if let Ok(path) = std::env::var("FLASH_PLAYER_DEBUGGER") {
        path
    } else {
        PathBuf::new()
            .join(env!("CARGO_MANIFEST_DIR"))
            .join("fprunner")
            .join("download")
            .join("flashplayerdebugger")
            .to_str()
            .unwrap()
            .to_string()
    };

    let frida_out_path = dir.path().join("frida_stdout.log");
    let frida_out = File::create(&frida_out_path).unwrap();

    let flash_log_path = dir.path().join("flash_log.txt");
    // Create it in advance so that the `notify` crate can watch it
    File::create(&flash_log_path).unwrap();

    struct FlashPlayerKiller {
        pid_location: PathBuf,
    }

    impl Drop for FlashPlayerKiller {
        fn drop(&mut self) {
            if let Ok(pid) = std::fs::read_to_string(&self.pid_location) {
                let pid = pid.trim().parse::<u32>().unwrap();
                let status = std::process::Command::new("kill")
                    .arg(pid.to_string())
                    .status()
                    .unwrap_or_else(|e| {
                        panic!("Failed to spawn `kill` for flash player with pid {pid}: {e}",)
                    });
                if !status.success() {
                    eprintln!("Failed to kill flash player with pid {pid}");
                }
            }
        }
    }

    let _killer = FlashPlayerKiller {
        pid_location: dir.path().join("flashplayer_pid"),
    };

    let _child = Command::new("frida")
        .args(&[
            "-l",
            agent_script_path.to_str().unwrap(),
            "--parameters",
            &frida_globals,
            "-f",
            &fp_path,
            abs_swf_path.to_str().unwrap(),
        ])
        .current_dir(&dir)
        .stdin(Stdio::piped())
        .stdout(frida_out.try_clone().unwrap())
        .stderr(frida_out.try_clone().unwrap())
        .spawn()
        .unwrap();

    let mut fp_output = match tail_lines(
        &flash_log_path,
        Duration::new(5, 0),
        Duration::new(5, 0),
        expected_output.lines().count(),
    ) {
        Ok(fp_output) => fp_output,
        Err(e) => {
            eprintln!(
                "Failed to get output from Flash Player. Frida logs: {}",
                std::fs::read_to_string(frida_out_path).unwrap()
            );
            panic!("Failed to get output: {e:?}");
        }
    };

    // Normalize this in the same way that we normalize our own trace output.
    fp_output = fp_output.replace('\r', "\n");

    assert_eq!(
        fp_output,
        expected_output,
        "Real Flash Player output does not match expected output\n\nFrida output:\n```\n{}```\n",
        std::fs::read_to_string(&frida_out_path).unwrap(),
    );
}

fn tail_lines(
    path: &Path,
    write_wait_timeout: Duration,
    overall_timeout: Duration,
    expected_lines: usize,
) -> Result<String, anyhow::Error> {
    let (sender, receiver) = std::sync::mpsc::channel();
    let mut watcher = notify::recommended_watcher(move |res| {
        let _ = sender.send(res);
    })?;

    watcher.configure(Config::default().with_poll_interval(Duration::from_secs(1)))?;

    watcher.watch(path, RecursiveMode::NonRecursive)?;

    let mut file = std::fs::File::open(path)?;
    let mut data = String::new();
    let mut found_lines = 0;

    let start = Instant::now();

    while Instant::now() - start < overall_timeout {
        match receiver.recv_timeout(write_wait_timeout)? {
            Ok(event) if matches!(event.kind, EventKind::Modify(ModifyKind::Data(_))) => {
                let mut new_data = String::new();
                file.read_to_string(&mut new_data)?;
                found_lines += new_data
                    .chars()
                    .filter(|c| *c == '\r' || *c == '\n')
                    .count();
                data += &new_data;
                if found_lines >= expected_lines {
                    return Ok(data);
                }
            }
            _ => {}
        }
    }

    Err(anyhow!("Timed out waiting for all lines to be read"))
}
