//! Tests running SWFs in a headless Ruffle instance.
//!
//! Trace output can be compared with correct output from the official Flash Player.

use ruffle_core::backend::{
    log::LogBackend,
    storage::{MemoryStorageBackend, StorageBackend},
};
use ruffle_core::external::Value as ExternalValue;

use crate::external_interface::ExternalInterfaceTestProvider;
use crate::util::runner::test_swf_with_hooks;
use anyhow::Context;
use anyhow::Result;
use libtest_mimic::{Arguments, Trial};
#[cfg(feature = "imgtests")]
use ruffle_render_wgpu::backend::WgpuRenderBackend;
#[cfg(feature = "imgtests")]
use ruffle_render_wgpu::{target::TextureTarget, wgpu};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::Path;
use std::rc::Rc;
use util::test::Test;

mod external_interface;
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
            let mut trial = Trial::test(test.name.to_string(), || test.run());
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
