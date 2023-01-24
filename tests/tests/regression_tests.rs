//! Tests running SWFs in a headless Ruffle instance.
//!
//! Trace output can be compared with correct output from the official Flash Player.

use approx::assert_relative_eq;
use regex::Regex;
use ruffle_core::backend::{
    log::LogBackend,
    navigator::{NullExecutor, NullNavigatorBackend},
    storage::{MemoryStorageBackend, StorageBackend},
};
use ruffle_core::context::UpdateContext;
use ruffle_core::events::MouseButton as RuffleMouseButton;
use ruffle_core::external::Value as ExternalValue;
use ruffle_core::external::{ExternalInterfaceMethod, ExternalInterfaceProvider};
use ruffle_core::limits::ExecutionLimit;
use ruffle_core::tag_utils::SwfMovie;
use ruffle_core::{Player, PlayerBuilder, PlayerEvent, ViewportDimensions};
use ruffle_input_format::{AutomatedEvent, InputInjector, MouseButton as InputMouseButton};

use libtest_mimic::{Arguments, Trial};
#[cfg(feature = "imgtests")]
use ruffle_render_wgpu::backend::WgpuRenderBackend;
#[cfg(feature = "imgtests")]
use ruffle_render_wgpu::{target::TextureTarget, wgpu};
use serde::Deserialize;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Deserialize)]
#[serde(default)]
struct TestOptions {
    num_frames: u32,
    sleep_to_meet_frame_rate: bool,
    image: bool,
    ignore: bool,
    approximations: Option<Approximations>,
    player_options: Option<PlayerOptions>,
}

impl Default for TestOptions {
    fn default() -> Self {
        Self {
            num_frames: 1,
            sleep_to_meet_frame_rate: false,
            image: false,
            ignore: false,
            approximations: None,
            player_options: None,
        }
    }
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct Approximations {
    number_patterns: Vec<String>,
    epsilon: Option<f64>,
    max_relative: Option<f64>,
}

impl Approximations {
    pub fn compare(&self, actual: f64, expected: f64) {
        match (self.epsilon, self.max_relative) {
            (Some(epsilon), Some(max_relative)) => assert_relative_eq!(
                actual,
                expected,
                epsilon = epsilon,
                max_relative = max_relative
            ),
            (Some(epsilon), None) => assert_relative_eq!(actual, expected, epsilon = epsilon),
            (None, Some(max_relative)) => {
                assert_relative_eq!(actual, expected, max_relative = max_relative)
            }
            (None, None) => assert_relative_eq!(actual, expected),
        }
    }
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct PlayerOptions {
    max_execution_duration: Option<Duration>,
    viewport_dimensions: Option<ViewportDimensions>,
}

impl PlayerOptions {
    pub fn setup(&self, player: Arc<Mutex<Player>>) {
        if let Some(max_execution_duration) = self.max_execution_duration {
            player
                .lock()
                .unwrap()
                .set_max_execution_duration(max_execution_duration);
        }
        if let Some(viewport_dimensions) = self.viewport_dimensions {
            player
                .lock()
                .unwrap()
                .set_viewport_dimensions(viewport_dimensions);
        }
    }
}

const RUN_IMG_TESTS: bool = cfg!(feature = "imgtests");

fn set_logger() {
    let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp(None)
        .is_test(true)
        .try_init();
}

type Error = Box<dyn std::error::Error>;

fn external_interface_avm1() -> Result<(), Error> {
    set_logger();
    test_swf_with_hooks(
        "tests/swfs/avm1/external_interface/test.swf",
        1,
        "tests/swfs/avm1/external_interface/input.json",
        "tests/swfs/avm1/external_interface/output.txt",
        |player| {
            player
                .lock()
                .unwrap()
                .add_external_interface(Box::new(ExternalInterfaceTestProvider::new()));
            Ok(())
        },
        |player| {
            let mut player_locked = player.lock().unwrap();

            let parroted =
                player_locked.call_internal_interface("parrot", vec!["Hello World!".into()]);
            player_locked.log_backend().avm_trace(&format!(
                "After calling `parrot` with a string: {parroted:?}",
            ));

            let mut nested = BTreeMap::new();
            nested.insert(
                "list".to_string(),
                vec![
                    "string".into(),
                    100.into(),
                    false.into(),
                    ExternalValue::Object(BTreeMap::new()),
                ]
                .into(),
            );

            let mut root = BTreeMap::new();
            root.insert("number".to_string(), (-500.1).into());
            root.insert("string".to_string(), "A string!".into());
            root.insert("true".to_string(), true.into());
            root.insert("false".to_string(), false.into());
            root.insert("null".to_string(), ExternalValue::Null);
            root.insert("nested".to_string(), nested.into());
            let result = player_locked
                .call_internal_interface("callWith", vec!["trace".into(), root.into()]);
            player_locked.log_backend().avm_trace(&format!(
                "After calling `callWith` with a complex payload: {result:?}",
            ));
            Ok(())
        },
        false,
        false,
    )
}

fn external_interface_avm2() -> Result<(), Error> {
    set_logger();
    test_swf_with_hooks(
        "tests/swfs/avm2/external_interface/test.swf",
        1,
        "tests/swfs/avm2/external_interface/input.json",
        "tests/swfs/avm2/external_interface/output.txt",
        |player| {
            player
                .lock()
                .unwrap()
                .add_external_interface(Box::new(ExternalInterfaceTestProvider::new()));
            Ok(())
        },
        |player| {
            let mut player_locked = player.lock().unwrap();

            let parroted =
                player_locked.call_internal_interface("parrot", vec!["Hello World!".into()]);
            player_locked.log_backend().avm_trace(&format!(
                "After calling `parrot` with a string: {parroted:?}",
            ));

            player_locked.call_internal_interface("freestanding", vec!["Hello World!".into()]);

            let root: ExternalValue = vec![
                "string".into(),
                100.into(),
                ExternalValue::Null,
                false.into(),
            ]
            .into();

            let result =
                player_locked.call_internal_interface("callWith", vec!["trace".into(), root]);
            player_locked.log_backend().avm_trace(&format!(
                "After calling `callWith` with a complex payload: {result:?}",
            ));
            Ok(())
        },
        false,
        false,
    )
}

fn shared_object_avm1() -> Result<(), Error> {
    set_logger();
    // Test SharedObject persistence. Run an SWF that saves data
    // to a shared object twice and verify that the data is saved.
    let mut memory_storage_backend: Box<dyn StorageBackend> =
        Box::<MemoryStorageBackend>::default();

    // Initial run; no shared object data.
    test_swf_with_hooks(
        "tests/swfs/avm1/shared_object/test.swf",
        1,
        "tests/swfs/avm1/shared_object/input1.json",
        "tests/swfs/avm1/shared_object/output1.txt",
        |_player| Ok(()),
        |player| {
            // Save the storage backend for next run.
            let mut player = player.lock().unwrap();
            std::mem::swap(player.storage_mut(), &mut memory_storage_backend);
            Ok(())
        },
        false,
        false,
    )?;

    // Verify that the flash cookie matches the expected one
    let expected = std::fs::read("tests/swfs/avm1/shared_object/RuffleTest.sol")?;
    assert_eq!(
        expected,
        memory_storage_backend
            .get("localhost//RuffleTest")
            .unwrap_or_default()
    );

    // Re-run the SWF, verifying that the shared object persists.
    test_swf_with_hooks(
        "tests/swfs/avm1/shared_object/test.swf",
        1,
        "tests/swfs/avm1/shared_object/input2.json",
        "tests/swfs/avm1/shared_object/output2.txt",
        |player| {
            // Swap in the previous storage backend.
            let mut player = player.lock().unwrap();
            std::mem::swap(player.storage_mut(), &mut memory_storage_backend);
            Ok(())
        },
        |_player| Ok(()),
        false,
        false,
    )?;

    Ok(())
}

fn shared_object_avm2() -> Result<(), Error> {
    set_logger();
    // Test SharedObject persistence. Run an SWF that saves data
    // to a shared object twice and verify that the data is saved.
    let mut memory_storage_backend: Box<dyn StorageBackend> =
        Box::<MemoryStorageBackend>::default();

    // Initial run; no shared object data.
    test_swf_with_hooks(
        "tests/swfs/avm2/shared_object/test.swf",
        1,
        "tests/swfs/avm2/shared_object/input1.json",
        "tests/swfs/avm2/shared_object/output1.txt",
        |_player| Ok(()),
        |player| {
            // Save the storage backend for next run.
            let mut player = player.lock().unwrap();
            std::mem::swap(player.storage_mut(), &mut memory_storage_backend);
            Ok(())
        },
        false,
        false,
    )?;

    // Verify that the flash cookie matches the expected one
    let expected = std::fs::read("tests/swfs/avm2/shared_object/RuffleTest.sol")?;
    assert_eq!(
        expected,
        memory_storage_backend
            .get("localhost//RuffleTest")
            .unwrap_or_default()
    );

    // Re-run the SWF, verifying that the shared object persists.
    test_swf_with_hooks(
        "tests/swfs/avm2/shared_object/test.swf",
        1,
        "tests/swfs/avm2/shared_object/input2.json",
        "tests/swfs/avm2/shared_object/output2.txt",
        |player| {
            // Swap in the previous storage backend.
            let mut player = player.lock().unwrap();
            std::mem::swap(player.storage_mut(), &mut memory_storage_backend);
            Ok(())
        },
        |_player| Ok(()),
        false,
        false,
    )?;

    Ok(())
}

/// Wrapper around string slice that makes debug output `{:?}` to print string same way as `{}`.
/// Used in different `assert*!` macros in combination with `pretty_assertions` crate to make
/// test failures to show nice diffs.
/// Courtesy of https://github.com/colin-kiegel/rust-pretty-assertions/issues/24
#[derive(PartialEq, Eq)]
#[doc(hidden)]
pub struct PrettyString<'a>(pub &'a str);

/// Make diff to display string as multi-line string
impl<'a> std::fmt::Debug for PrettyString<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(self.0)
    }
}

macro_rules! assert_eq {
    ($left:expr, $right:expr) => {
        pretty_assertions::assert_eq!(PrettyString($left.as_ref()), PrettyString($right.as_ref()));
    };
    ($left:expr, $right:expr, $message:expr) => {
        pretty_assertions::assert_eq!(
            PrettyString($left.as_ref()),
            PrettyString($right.as_ref()),
            $message
        );
    };
}

/// Loads an SWF and runs it through the Ruffle core for a number of frames.
/// Tests that the trace output matches the given expected output.
#[allow(clippy::too_many_arguments)]
fn test_swf_with_hooks(
    swf_path: &str,
    num_frames: u32,
    simulated_input_path: &str,
    expected_output_path: &str,
    before_start: impl FnOnce(Arc<Mutex<Player>>) -> Result<(), Error>,
    before_end: impl FnOnce(Arc<Mutex<Player>>) -> Result<(), Error>,
    check_img: bool,
    frame_time_sleep: bool,
) -> Result<(), Error> {
    let injector =
        InputInjector::from_file(simulated_input_path).unwrap_or_else(|_| InputInjector::empty());
    let mut expected_output = std::fs::read_to_string(expected_output_path)?.replace("\r\n", "\n");

    // Strip a trailing newline if it has one.
    if expected_output.ends_with('\n') {
        expected_output = expected_output[0..expected_output.len() - "\n".len()].to_string();
    }

    let trace_log = run_swf(
        swf_path,
        num_frames,
        before_start,
        injector,
        before_end,
        check_img,
        frame_time_sleep,
    )?;
    assert_eq!(
        trace_log, expected_output,
        "ruffle output != flash player output"
    );

    Ok(())
}

/// Loads an SWF and runs it through the Ruffle core for a number of frames.
/// Tests that the trace output matches the given expected output.
/// If a line has a floating point value, it will be compared approxinmately using the given epsilon.
fn test_swf_approx(
    swf_path: &str,
    num_frames: u32,
    simulated_input_path: &str,
    expected_output_path: &str,
    num_patterns: &[Regex],
    check_img: bool,
    approx_assert_fn: impl Fn(f64, f64),
) -> Result<(), Error> {
    let injector =
        InputInjector::from_file(simulated_input_path).unwrap_or_else(|_| InputInjector::empty());
    let trace_log = run_swf(
        swf_path,
        num_frames,
        |_| Ok(()),
        injector,
        |_| Ok(()),
        check_img,
        false,
    )?;
    let mut expected_data = std::fs::read_to_string(expected_output_path)?;

    // Strip a trailing newline if it has one.
    if expected_data.ends_with('\n') {
        expected_data = expected_data[0..expected_data.len() - "\n".len()].to_string();
    }

    std::assert_eq!(
        trace_log.lines().count(),
        expected_data.lines().count(),
        "# of lines of output didn't match"
    );

    for (actual, expected) in trace_log.lines().zip(expected_data.lines()) {
        // If these are numbers, compare using approx_eq.
        if let (Ok(actual), Ok(expected)) = (actual.parse::<f64>(), expected.parse::<f64>()) {
            // NaNs should be able to pass in an approx test.
            if actual.is_nan() && expected.is_nan() {
                continue;
            }

            // TODO: Lower this epsilon as the accuracy of the properties improves.
            // if let Some(relative_epsilon) = relative_epsilon {
            //     assert_relative_eq!(
            //         actual,
            //         expected,
            //         epsilon = absolute_epsilon,
            //         max_relative = relative_epsilon
            //     );
            // } else {
            //     assert_abs_diff_eq!(actual, expected, epsilon = absolute_epsilon);
            // }
            approx_assert_fn(actual, expected);
        } else {
            let mut found = false;
            // Check each of the user-provided regexes for a match
            for pattern in num_patterns {
                if let (Some(actual_captures), Some(expected_captures)) =
                    (pattern.captures(actual), pattern.captures(expected))
                {
                    found = true;
                    std::assert_eq!(
                        actual_captures.len(),
                        expected_captures.len(),
                        "Differing numbers of regex captures"
                    );

                    // Each capture group (other than group 0, which is always the entire regex
                    // match) represents a floating-point value
                    for (actual_val, expected_val) in actual_captures
                        .iter()
                        .skip(1)
                        .zip(expected_captures.iter().skip(1))
                    {
                        let actual_num = actual_val
                            .expect("Missing capture gruop value for 'actual'")
                            .as_str()
                            .parse::<f64>()
                            .expect("Failed to parse 'actual' capture group as float");
                        let expected_num = expected_val
                            .expect("Missing capture gruop value for 'expected'")
                            .as_str()
                            .parse::<f64>()
                            .expect("Failed to parse 'expected' capture group as float");
                        approx_assert_fn(actual_num, expected_num);
                    }
                    let modified_actual = pattern.replace(actual, "");
                    let modified_expected = pattern.replace(expected, "");
                    assert_eq!(modified_actual, modified_expected);
                    break;
                }
            }
            if !found {
                assert_eq!(actual, expected);
            }
        }
    }
    Ok(())
}

/// Loads an SWF and runs it through the Ruffle core for a number of frames.
/// Tests that the trace output matches the given expected output.
fn run_swf(
    swf_path: &str,
    num_frames: u32,
    before_start: impl FnOnce(Arc<Mutex<Player>>) -> Result<(), Error>,
    mut injector: InputInjector,
    before_end: impl FnOnce(Arc<Mutex<Player>>) -> Result<(), Error>,
    #[allow(unused)] mut check_img: bool,
    frame_time_sleep: bool,
) -> Result<String, Error> {
    #[allow(unused_assignments)]
    {
        check_img &= RUN_IMG_TESTS;
    }

    let base_path = Path::new(swf_path).parent().unwrap();
    let mut executor = NullExecutor::new();
    let movie = SwfMovie::from_path(swf_path, None)?;
    let frame_time = 1000.0 / movie.frame_rate().to_f64();
    let frame_time_duration = Duration::from_millis(frame_time as u64);
    let trace_output = Rc::new(RefCell::new(Vec::new()));

    #[allow(unused_mut)]
    let mut builder = PlayerBuilder::new();

    #[cfg(feature = "imgtests")]
    if check_img {
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
            ))?;

        let width = movie.width().to_pixels() as u32;
        let height = movie.height().to_pixels() as u32;

        let target = TextureTarget::new(&descriptors.device, (width, height))?;

        builder = builder
            .with_renderer(WgpuRenderBackend::new(Arc::new(descriptors), target, 4)?)
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

    before_start(player.clone())?;

    for _ in 0..num_frames {
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
        if frame_time_sleep {
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
    if check_img {
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

#[derive(Default)]
pub struct ExternalInterfaceTestProvider {}

impl ExternalInterfaceTestProvider {
    pub fn new() -> Self {
        Default::default()
    }
}

fn do_trace(context: &mut UpdateContext<'_, '_>, args: &[ExternalValue]) -> ExternalValue {
    context.avm_trace(&format!("[ExternalInterface] trace: {args:?}"));
    "Traced!".into()
}

fn do_ping(context: &mut UpdateContext<'_, '_>, _args: &[ExternalValue]) -> ExternalValue {
    context.avm_trace("[ExternalInterface] ping");
    "Pong!".into()
}

fn do_reentry(context: &mut UpdateContext<'_, '_>, _args: &[ExternalValue]) -> ExternalValue {
    context.avm_trace("[ExternalInterface] starting reentry");
    if let Some(callback) = context.external_interface.get_callback("callWith") {
        callback.call(
            context,
            "callWith",
            vec!["trace".into(), "successful reentry!".into()],
        )
    } else {
        ExternalValue::Null
    }
}

impl ExternalInterfaceProvider for ExternalInterfaceTestProvider {
    fn get_method(&self, name: &str) -> Option<Box<dyn ExternalInterfaceMethod>> {
        match name {
            "trace" => Some(Box::new(do_trace)),
            "ping" => Some(Box::new(do_ping)),
            "reentry" => Some(Box::new(do_reentry)),
            _ => None,
        }
    }

    fn on_callback_available(&self, _name: &str) {}

    fn on_fs_command(&self, _command: &str, _args: &str) -> bool {
        false
    }
}

fn run_test(options: TestOptions, root: &Path) -> Result<(), libtest_mimic::Failed> {
    set_logger();

    if let Some(approximations) = &options.approximations {
        let num_patterns: Vec<Regex> = approximations
            .number_patterns
            .iter()
            .map(|p| Regex::new(&p).unwrap())
            .collect();
        test_swf_approx(
            root.join("test.swf").to_str().unwrap(),
            options.num_frames,
            root.join("input.json").to_str().unwrap(),
            root.join("output.txt").to_str().unwrap(),
            &num_patterns,
            options.image,
            |actual, expected| approximations.compare(actual, expected),
        )
        .map_err(|e| e.to_string().into())
    } else {
        test_swf_with_hooks(
            root.join("test.swf").to_str().unwrap(),
            options.num_frames,
            root.join("input.json").to_str().unwrap(),
            root.join("output.txt").to_str().unwrap(),
            |player| {
                if let Some(player_options) = &options.player_options {
                    player_options.setup(player);
                }
                Ok(())
            },
            |_| Ok(()),
            options.image,
            options.sleep_to_meet_frame_rate,
        )
        .map_err(|e| e.to_string().into())
    }
}

fn main() {
    let args = Arguments::from_args();

    let root = Path::new("tests/swfs");
    let mut tests: Vec<Trial> = walkdir::WalkDir::new(root)
        .into_iter()
        .map(Result::unwrap)
        .filter(|entry| entry.file_type().is_file() && entry.file_name() == "test.toml")
        .map(|file| {
            let options: TestOptions =
                toml::from_str(&fs::read_to_string(&file.path()).unwrap()).unwrap();
            let test_dir = file.path().parent().unwrap().to_owned();
            let name = test_dir
                .strip_prefix(root)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/");
            let ignore = options.ignore || (options.image && !RUN_IMG_TESTS);
            let mut test = Trial::test(name, move || run_test(options, &test_dir));
            if ignore {
                test = test.with_ignored_flag(true);
            }
            test
        })
        .collect();

    // Manual tests here, since #[test] doesn't work once we use our own test harness
    tests.push(Trial::test("shared_object_avm1", || {
        shared_object_avm1().map_err(|e| e.to_string().into())
    }));
    tests.push(Trial::test("shared_object_avm2", || {
        shared_object_avm2().map_err(|e| e.to_string().into())
    }));
    tests.push(Trial::test("external_interface_avm1", || {
        external_interface_avm1().map_err(|e| e.to_string().into())
    }));
    tests.push(Trial::test("external_interface_avm2", || {
        external_interface_avm2().map_err(|e| e.to_string().into())
    }));

    tests.sort_unstable_by(|a, b| a.name().cmp(b.name()));

    libtest_mimic::run(&args, tests).exit()
}
