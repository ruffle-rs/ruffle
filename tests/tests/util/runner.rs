use crate::util::test::Test;
use anyhow::{anyhow, Result};
use ruffle_core::backend::log::LogBackend;
use ruffle_core::backend::navigator::{NullExecutor, NullNavigatorBackend};
use ruffle_core::events::MouseButton as RuffleMouseButton;
use ruffle_core::limits::ExecutionLimit;
use ruffle_core::tag_utils::SwfMovie;
use ruffle_core::{Player, PlayerBuilder, PlayerEvent};
use ruffle_input_format::{AutomatedEvent, InputInjector, MouseButton as InputMouseButton};
#[cfg(feature = "imgtests")]
use ruffle_render_wgpu::backend::WgpuRenderBackend;
#[cfg(feature = "imgtests")]
use ruffle_render_wgpu::{target::TextureTarget, wgpu};
use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub const RUN_IMG_TESTS: bool = cfg!(feature = "imgtests");

struct TestLogBackend {
    trace_output: Rc<RefCell<Vec<String>>>,
}

impl TestLogBackend {
    pub fn new(trace_output: Rc<RefCell<Vec<String>>>) -> Self {
        Self { trace_output }
    }
}

impl LogBackend for TestLogBackend {
    fn avm_trace(&self, message: &str) {
        self.trace_output.borrow_mut().push(message.to_string());
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
    let trace_output = Rc::new(RefCell::new(Vec::new()));

    #[allow(unused_mut)]
    let mut builder = PlayerBuilder::new();

    #[cfg(feature = "imgtests")]
    if test.options.image {
        const BACKEND: wgpu::Backends = wgpu::Backends::PRIMARY;

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: BACKEND,
            dx12_shader_compiler: wgpu::Dx12Compiler::default(),
        });

        let descriptors =
            futures::executor::block_on(WgpuRenderBackend::<TextureTarget>::build_descriptors(
                BACKEND,
                instance,
                None,
                Default::default(),
                None,
            ))
            .map_err(|e| anyhow!(e.to_string()))?;

        let width = movie.width().to_pixels() as u32;
        let height = movie.height().to_pixels() as u32;

        let target = TextureTarget::new(&descriptors.device, (width, height))
            .map_err(|e| anyhow!(e.to_string()))?;

        builder = builder
            .with_renderer(
                WgpuRenderBackend::new(Arc::new(descriptors), target, 4)
                    .map_err(|e| anyhow!(e.to_string()))?,
            )
            .with_viewport_dimensions(width, height, 1.0);
    };

    let player = builder
        .with_log(TestLogBackend::new(trace_output.clone()))
        .with_navigator(NullNavigatorBackend::with_base_path(base_path, &executor)?)
        .with_max_execution_duration(Duration::from_secs(300))
        .with_viewport_dimensions(
            movie.width().to_pixels() as u32,
            movie.height().to_pixels() as u32,
            1.0,
        )
        .with_movie(movie)
        .build();

    if let Some(options) = &test.options.player_options {
        options.setup(&player);
    }

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
    if test.options.image {
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

        let info = renderer.descriptors().adapter.get_info();
        let suffix = format!("{}-{:?}", std::env::consts::OS, info.backend);

        let expected_image_path = base_path.join(format!("expected-{}.png", &suffix));
        let expected_image = image::open(&expected_image_path);

        let matches = match expected_image {
            Ok(img) => {
                img.as_rgba8().expect("Expected 8-bit RGBA image").as_raw() == actual_image.as_raw()
            }
            Err(e) => {
                eprintln!(
                    "Failed to open expected image {:?}: {e:?}",
                    &expected_image_path
                );
                false
            }
        };

        if !matches {
            let actual_image_path = base_path.join(format!("actual-{suffix}.png"));
            actual_image.save_with_format(&actual_image_path, image::ImageFormat::Png)?;
            panic!("Test output does not match expected image - saved actual image to {actual_image_path:?}");
        }
    }

    before_end(player)?;

    executor.run();

    let trace = trace_output.borrow().join("\n");
    Ok(trace)
}
