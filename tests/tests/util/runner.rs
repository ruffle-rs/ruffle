use crate::util::test::Test;
use anyhow::{anyhow, Result};
use ruffle_core::backend::log::LogBackend;
use ruffle_core::backend::navigator::{NullExecutor, NullNavigatorBackend};
use ruffle_core::events::MouseButton as RuffleMouseButton;
use ruffle_core::limits::ExecutionLimit;
use ruffle_core::tag_utils::SwfMovie;
use ruffle_core::{Player, PlayerBuilder, PlayerEvent};
use ruffle_input_format::{AutomatedEvent, InputInjector, MouseButton as InputMouseButton};
use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::Duration;

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
    Ok(trace)
}