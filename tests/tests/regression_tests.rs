//! Tests running SWFs in a headless Ruffle instance.
//!
//! Trace output can be compared with correct output from the official Flash Player.

use ruffle_core::backend::{
    log::LogBackend,
    storage::{MemoryStorageBackend, StorageBackend},
};
use ruffle_core::context::UpdateContext;
use ruffle_core::external::Value as ExternalValue;
use ruffle_core::external::{ExternalInterfaceMethod, ExternalInterfaceProvider};
use ruffle_core::Player;

use crate::util::runner::test_swf_approx;
use anyhow::Context;
use anyhow::Result;
use libtest_mimic::{Arguments, Trial};
use ruffle_input_format::InputInjector;
#[cfg(feature = "imgtests")]
use ruffle_render_wgpu::backend::WgpuRenderBackend;
#[cfg(feature = "imgtests")]
use ruffle_render_wgpu::{target::TextureTarget, wgpu};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::Path;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use util::runner::run_swf;
use util::test::Test;

mod util;

const RUN_IMG_TESTS: bool = cfg!(feature = "imgtests");

fn set_logger() {
    let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp(None)
        .is_test(true)
        .try_init();
}

fn external_interface_avm1() -> Result<()> {
    set_logger();
    test_swf_with_hooks(
        Path::new("tests/swfs/avm1/external_interface/test.swf"),
        1,
        Path::new("tests/swfs/avm1/external_interface/input.json"),
        Path::new("tests/swfs/avm1/external_interface/output.txt"),
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

fn external_interface_avm2() -> Result<()> {
    set_logger();
    test_swf_with_hooks(
        Path::new("tests/swfs/avm2/external_interface/test.swf"),
        1,
        Path::new("tests/swfs/avm2/external_interface/input.json"),
        Path::new("tests/swfs/avm2/external_interface/output.txt"),
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

fn shared_object_avm1() -> Result<()> {
    set_logger();
    // Test SharedObject persistence. Run an SWF that saves data
    // to a shared object twice and verify that the data is saved.
    let mut memory_storage_backend: Box<dyn StorageBackend> =
        Box::<MemoryStorageBackend>::default();

    // Initial run; no shared object data.
    test_swf_with_hooks(
        Path::new("tests/swfs/avm1/shared_object/test.swf"),
        1,
        Path::new("tests/swfs/avm1/shared_object/input1.json"),
        Path::new("tests/swfs/avm1/shared_object/output1.txt"),
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
    std::assert_eq!(
        expected,
        memory_storage_backend
            .get("localhost//RuffleTest")
            .unwrap_or_default()
    );

    // Re-run the SWF, verifying that the shared object persists.
    test_swf_with_hooks(
        Path::new("tests/swfs/avm1/shared_object/test.swf"),
        1,
        Path::new("tests/swfs/avm1/shared_object/input2.json"),
        Path::new("tests/swfs/avm1/shared_object/output2.txt"),
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

fn shared_object_avm2() -> Result<()> {
    set_logger();
    // Test SharedObject persistence. Run an SWF that saves data
    // to a shared object twice and verify that the data is saved.
    let mut memory_storage_backend: Box<dyn StorageBackend> =
        Box::<MemoryStorageBackend>::default();

    // Initial run; no shared object data.
    test_swf_with_hooks(
        Path::new("tests/swfs/avm2/shared_object/test.swf"),
        1,
        Path::new("tests/swfs/avm2/shared_object/input1.json"),
        Path::new("tests/swfs/avm2/shared_object/output1.txt"),
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
    std::assert_eq!(
        expected,
        memory_storage_backend
            .get("localhost//RuffleTest")
            .unwrap_or_default()
    );

    // Re-run the SWF, verifying that the shared object persists.
    test_swf_with_hooks(
        Path::new("tests/swfs/avm2/shared_object/test.swf"),
        1,
        Path::new("tests/swfs/avm2/shared_object/input2.json"),
        Path::new("tests/swfs/avm2/shared_object/output2.txt"),
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

/// Loads an SWF and runs it through the Ruffle core for a number of frames.
/// Tests that the trace output matches the given expected output.
#[allow(clippy::too_many_arguments)]
fn test_swf_with_hooks(
    swf_path: &Path,
    num_frames: u32,
    simulated_input_path: &Path,
    expected_output_path: &Path,
    before_start: impl FnOnce(Arc<Mutex<Player>>) -> Result<()>,
    before_end: impl FnOnce(Arc<Mutex<Player>>) -> Result<()>,
    check_img: bool,
    frame_time_sleep: bool,
) -> Result<()> {
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

fn run_test(test: Test) -> Result<(), libtest_mimic::Failed> {
    set_logger();

    if let Some(approximations) = &test.options.approximations {
        test_swf_approx(
            &test.swf_path,
            test.options.num_frames,
            &test.input_path,
            &test.output_path,
            &approximations.number_patterns(),
            test.options.image,
            |actual, expected| approximations.compare(actual, expected),
        )
        .map_err(|e| e.to_string().into())
    } else {
        test_swf_with_hooks(
            &test.swf_path,
            test.options.num_frames,
            &test.input_path,
            &test.output_path,
            |player| {
                if let Some(player_options) = &test.options.player_options {
                    player_options.setup(player);
                }
                Ok(())
            },
            |_| Ok(()),
            test.options.image,
            test.options.sleep_to_meet_frame_rate,
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
            let test = Test::from_options(file.path(), root)
                .context("Couldn't create test")
                .unwrap();
            let ignore = test.options.ignore || (test.options.image && !RUN_IMG_TESTS);
            let mut trial = Trial::test(test.name.to_string(), move || run_test(test));
            if ignore {
                trial = trial.with_ignored_flag(true);
            }
            trial
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
