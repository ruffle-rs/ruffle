//! Ruffle test harness.
//!
//! Common utilities for running a Ruffle test or benchmark using a headless player.
//! Using `TestPlayerBuilder`, you can configure and run a test, receiving the trace log as output.

use ruffle_core::backend::{
    log::LogBackend,
    navigator::{NullExecutor, NullNavigatorBackend},
    storage::StorageBackend,
};
use ruffle_core::{
    events::MouseButton as RuffleMouseButton, external::ExternalInterfaceProvider,
    tag_utils::SwfMovie, Player, PlayerBuilder, PlayerEvent,
};
use ruffle_input_format::{AutomatedEvent, InputInjector, MouseButton as InputMouseButton};
use ruffle_render_wgpu::{target::TextureTarget, wgpu, WgpuRenderBackend};
use std::{
    cell::RefCell,
    path::{Path, PathBuf},
    rc::Rc,
    sync::{Arc, Mutex},
    time::Duration,
};

type BeforeEndFn<'a> = Box<dyn 'a + FnOnce(&Arc<Mutex<Player>>) -> Result<(), Error>>;
type Error = Box<dyn std::error::Error>;

/// Controls a `Player` instance for a test or benchmark.
pub struct TestPlayer {
    player: Arc<Mutex<Player>>,
    executor: NullExecutor,
    input_injector: InputInjector,
    output_image: bool,
    trace_log: Rc<RefCell<String>>,
}

impl TestPlayer {
    pub fn player(&self) -> &Arc<Mutex<Player>> {
        &self.player
    }

    /// Runs the player for the spcified number of frames.
    pub fn run(&mut self, num_frames: u32) -> Result<(), Error> {
        let frame_time = 1000.0 / self.player.lock().unwrap().frame_rate();
        for _ in 0..num_frames {
            let mut player = self.player.lock().unwrap();
            player.run_frame();
            player.update_timers(frame_time);
            drop(player);
            self.executor.run();

            self.input_injector.next(|evt, _btns_down| {
                self.player.lock().unwrap().handle_event(match evt {
                    AutomatedEvent::MouseDown { pos, btn } => PlayerEvent::MouseDown {
                        x: pos.0,
                        y: pos.1,
                        button: match btn {
                            InputMouseButton::Left => RuffleMouseButton::Left,
                            InputMouseButton::Middle => RuffleMouseButton::Middle,
                            InputMouseButton::Right => RuffleMouseButton::Right,
                        },
                    },
                    AutomatedEvent::MouseMove { pos } => {
                        PlayerEvent::MouseMove { x: pos.0, y: pos.1 }
                    }
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
        }
        Ok(())
    }

    #[inline]
    pub fn run_avm1_bench(&mut self) {
        self.player.lock().unwrap().run_avm1_bench();
    }

    #[inline]
    pub fn run_avm2_bench(&mut self) {
        self.player.lock().unwrap().run_avm2_bench();
    }

    fn finish(self) -> TestResult {
        // Screenshot the final frame, if requested.
        let (image, platform_id) = if self.output_image {
            let mut player = self.player.lock().unwrap();
            player.render();
            let renderer = player
                .renderer_mut()
                .downcast_mut::<WgpuRenderBackend<TextureTarget>>()
                .unwrap();
            let target = renderer.target();
            let image = target
                .capture(renderer.device())
                .expect("Failed to capture image");
            (
                Some(image),
                format!("{}-{}", std::env::consts::OS, renderer.adapter_name()),
            )
        } else {
            (None, format!("{}-unknown", std::env::consts::OS))
        };

        TestResult {
            trace_log: self.trace_log.borrow().clone(),
            platform_id,
            image,
            player: self.player.clone(),
        }
    }
}

/// A factory for building a `TestPlayer` harness.
///
/// Methods can be chained in order to configure this.
#[must_use]
pub struct TestBuilder<'a> {
    builder: PlayerBuilder,
    swf_path: PathBuf,
    simulated_input_path: PathBuf,
    output_image: bool,
    is_avm_bench: bool,
    before_end_fn: Option<BeforeEndFn<'a>>,
}

/// A builder for a test.
impl<'a> TestBuilder<'a> {
    const DEFAULT_MAX_DURATION: Duration = Duration::from_secs(300);

    /// Creates a new builder for a `TestPlayer` with the default options.
    pub fn new() -> Self {
        Self {
            builder: PlayerBuilder::new().with_max_execution_duration(Self::DEFAULT_MAX_DURATION),
            swf_path: PathBuf::new(),
            simulated_input_path: PathBuf::new(),
            before_end_fn: None,
            is_avm_bench: false,
            output_image: false,
        }
    }

    /// Sets the path to a local SWF file that this test will run.
    ///
    /// If an empty path is given, the player will run the default empty movie.
    pub fn with_swf_path(mut self, swf_path: impl Into<PathBuf>) -> Self {
        self.swf_path = swf_path.into();
        self
    }

    /// Sets the path to the simulated input for this test.
    ///
    /// If an empty path is given, the player will run with no input.
    pub fn with_simulated_input_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.simulated_input_path = path.into();
        self
    }

    /// Sets a function that the test will run after completion.
    ///
    /// This can be used to append values to the trace log or do other checks on the final state
    /// of the player.
    pub fn before_end_fn<F>(mut self, f: F) -> Self
    where
        F: 'a + FnOnce(&Arc<Mutex<Player>>) -> Result<(), Error>,
    {
        self.before_end_fn = Some(Box::new(f));
        self
    }

    /// Set the `StorageBackend` used by the test.
    ///
    /// A`MemoryStorageBackend` is used by default.
    pub fn with_storage(mut self, storage: impl 'static + StorageBackend) -> Self {
        self.builder = self.builder.with_storage(storage);
        self
    }

    /// Adds an external interface to the test.
    pub fn with_external_interface(
        mut self,
        external_interface: impl 'static + ExternalInterfaceProvider,
    ) -> Self {
        self.builder = self.builder.with_external_interface(external_interface);
        self
    }

    /// Sets the maximum ActionScript execution duration for the player.
    ///
    /// The default duration is 300 seconds by default.
    pub fn with_max_execution_duration(mut self, duration: Duration) -> Self {
        self.builder = self.builder.with_max_execution_duration(duration);
        self
    }

    /// Sets whether the test run will capture a screenshot of the final frame into `TestResult::image`.
    ///
    /// `false` by default.
    pub fn with_output_image(mut self, output_image: bool) -> Self {
        self.output_image = output_image;
        self
    }

    /// Sets whether this test is an AVM benchmark.
    pub fn is_avm_bench(mut self, is_avm_bench: bool) -> Self {
        self.is_avm_bench = is_avm_bench;
        self
    }

    /// Sets the initial viewport dimensions of the player.
    ///
    /// 550x400 is the default viewport size.
    pub fn with_viewport_dimensions(mut self, width: u32, height: u32) -> Self {
        self.builder = self.builder.with_viewport_dimensions(width, height, 1.0);
        self
    }

    /// Builds a `TestPlayer` with the given options.
    ///
    /// The test can be run using `TestPlayer::run`, or you can build the player and run the test
    /// in one step using the `TestPlayerBuilder::run` convenience method.
    pub fn build(self) -> Result<TestPlayer, Error> {
        let base_path = Path::new(&self.swf_path).parent().unwrap();
        let executor = NullExecutor::new();
        let movie = if self.swf_path.as_os_str().is_empty() {
            SwfMovie::empty(8)
        } else {
            SwfMovie::from_path(&self.swf_path, None)?
        };
        let movie_dimensions = (
            movie.width().to_pixels() as u32,
            movie.height().to_pixels() as u32,
        );
        let mut builder = self.builder.with_movie(movie);
        let trace_output = Rc::new(RefCell::new(String::new()));

        let input_injector = InputInjector::from_file(self.simulated_input_path)
            .unwrap_or_else(|_| InputInjector::empty());

        // Use the wgpu renderer if we are outputting an image.
        if self.output_image {
            let backend_bit = wgpu::Backends::PRIMARY;
            let instance = wgpu::Instance::new(backend_bit);

            let descriptors = futures::executor::block_on(
                WgpuRenderBackend::<TextureTarget>::build_descriptors(
                    backend_bit,
                    instance,
                    None,
                    Default::default(),
                    None,
                ),
            )?;
            let target = TextureTarget::new(&descriptors.device, movie_dimensions);
            builder = builder
                .with_renderer(WgpuRenderBackend::new(descriptors, target)?)
                .with_software_video();
        }

        let player = builder
            .with_navigator(NullNavigatorBackend::with_base_path(base_path, &executor))
            .with_log(TestLogBackend::new(trace_output.clone()))
            .build();
        {
            let mut player = player.lock().unwrap();
            // Prime the action queue is this is an AVM1 benchmark.
            if self.is_avm_bench {
                player.init_avm_bench()?;
            }
        }

        Ok(TestPlayer {
            player,
            executor,
            input_injector,
            trace_log: trace_output,
            output_image: self.output_image,
        })
    }

    /// Builds the `TestPlayer` and runs the test in one step.
    pub fn run(mut self, num_frames: u32) -> Result<TestResult, Error> {
        let before_end_fn = self.before_end_fn.take();
        let mut player = self.build()?;
        player.run(num_frames)?;
        if let Some(before_end_fn) = before_end_fn {
            before_end_fn(&player.player)?;
        }
        Ok(player.finish())
    }
}

impl<'a> Default for TestBuilder<'a> {
    fn default() -> Self {
        Self::new()
    }
}

/// A log backend that stores the trace log in `String` for later inspection.
///
/// The trace log is compared to expected output from the Flash Player.
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
        let mut trace_log = self.trace_output.borrow_mut();
        trace_log.push_str(message);
        trace_log.push('\n');
    }
}

/// The results of a `TestPlayer` run.
pub struct TestResult {
    /// The trace log of the test run.
    ///
    /// This will include a trailing newline.
    pub trace_log: String,

    /// The OS and graphics adapter the test is running on, in the format of `{OS}-{adapter}`.
    ///
    /// The graphics adapter will be `unknown` if this test does not output an image.
    pub platform_id: String,

    /// A screenshot of the final frame, if requested.
    ///
    /// Only set if `TestPlayerBuilder::output_image` was called with `true`.
    pub image: Option<image::RgbaImage>,

    /// The `ruffle_core::Player` that ran the test.
    pub player: Arc<Mutex<Player>>,
}
