use crate::external_interface::ExternalInterfaceTestProvider;
use ruffle_core::external::Value as ExternalValue;
use ruffle_test_framework::environment::Environment;
use ruffle_test_framework::options::TestOptions;
use ruffle_test_framework::runner::TestStatus;
use ruffle_test_framework::test::Test;
use ruffle_test_framework::vfs::{PhysicalFS, VfsPath};
use std::collections::BTreeMap;
use std::thread::sleep;

pub fn external_interface_avm1(
    environment: &impl Environment,
) -> Result<(), libtest_mimic::Failed> {
    let test = &Test::from_options(
        TestOptions {
            num_frames: Some(2),
            ..Default::default()
        },
        VfsPath::new(PhysicalFS::new("tests/swfs/avm1/external_interface/")),
        "external_interface_avm1".to_string(),
    )?;
    let mut runner = test.create_test_runner(environment)?;

    runner
        .player()
        .lock()
        .unwrap()
        .set_external_interface_provider(Some(Box::new(ExternalInterfaceTestProvider::new())));

    let mut first = true;

    loop {
        runner.tick();
        match runner.test()? {
            TestStatus::Continue => {}
            TestStatus::Sleep(duration) => sleep(duration),
            TestStatus::Finished => break,
        }

        if first {
            first = false;
            let mut player_locked = runner.player().lock().unwrap();

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
        }
    }

    Ok(())
}

pub fn external_interface_avm2(
    environment: &impl Environment,
) -> Result<(), libtest_mimic::Failed> {
    let test = &Test::from_options(
        TestOptions {
            num_frames: Some(2),
            ..Default::default()
        },
        VfsPath::new(PhysicalFS::new("tests/swfs/avm2/external_interface/")),
        "external_interface_avm2".to_string(),
    )?;
    let mut runner = test.create_test_runner(environment)?;
    runner
        .player()
        .lock()
        .unwrap()
        .set_external_interface_provider(Some(Box::new(ExternalInterfaceTestProvider::new())));

    let mut first = true;

    loop {
        runner.tick();
        match runner.test()? {
            TestStatus::Continue => {}
            TestStatus::Sleep(duration) => sleep(duration),
            TestStatus::Finished => break,
        }

        if first {
            first = false;
            let mut player_locked = runner.player().lock().unwrap();

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
        }
    }

    Ok(())
}
