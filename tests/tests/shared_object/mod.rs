use ruffle_core::backend::storage::{MemoryStorageBackend, StorageBackend};
use ruffle_test_framework::environment::Environment;
use ruffle_test_framework::options::TestOptions;
use ruffle_test_framework::runner::TestStatus;
use ruffle_test_framework::test::Test;
use ruffle_test_framework::vfs::{PhysicalFS, VfsPath};
use std::thread::sleep;

pub fn shared_object_avm1(environment: &impl Environment) -> Result<(), libtest_mimic::Failed> {
    // Test SharedObject persistence. Run an SWF that saves data
    // to a shared object twice and verify that the data is saved.
    let mut memory_storage_backend: Box<dyn StorageBackend> =
        Box::<MemoryStorageBackend>::default();

    // Initial run; no shared object data.
    let test1 = &Test::from_options(
        TestOptions {
            num_frames: Some(1),
            output_path: "output1.txt".into(),
            ..Default::default()
        },
        VfsPath::new(PhysicalFS::new("tests/swfs/avm1/shared_object/")),
        "shared_object_avm1".to_string(),
    )?;
    let mut runner = test1.create_test_runner(environment)?;

    loop {
        runner.tick();
        match runner.test()? {
            TestStatus::Continue => {}
            TestStatus::Sleep(duration) => sleep(duration),
            TestStatus::Finished => break,
        }
    }

    // Save the storage backend for next run.
    {
        let mut player = runner.player().lock().unwrap();
        std::mem::swap(player.storage_mut(), &mut memory_storage_backend);
    }

    // Verify that the flash cookie matches the expected one
    let expected = std::fs::read("tests/swfs/avm1/shared_object/RuffleTest.sol")?;
    std::assert_eq!(
        expected,
        memory_storage_backend
            .get("localhost//RuffleTest")
            .unwrap_or_default()
    );

    // Re-run the SWF, verifying that the shared object persists.
    let test2 = &Test::from_options(
        TestOptions {
            num_frames: Some(1),
            output_path: "output2.txt".into(),
            ..Default::default()
        },
        VfsPath::new(PhysicalFS::new("tests/swfs/avm1/shared_object/")),
        "shared_object_avm1".to_string(),
    )?;

    let mut runner = test2.create_test_runner(environment)?;
    {
        // Swap in the previous storage backend.
        let mut player = runner.player().lock().unwrap();
        std::mem::swap(player.storage_mut(), &mut memory_storage_backend);
    }

    loop {
        runner.tick();
        match runner.test()? {
            TestStatus::Continue => {}
            TestStatus::Sleep(duration) => sleep(duration),
            TestStatus::Finished => break,
        }
    }

    Ok(())
}

pub fn shared_object_self_ref_avm1(
    environment: &impl Environment,
) -> Result<(), libtest_mimic::Failed> {
    // Test SharedObject persistence. Run an SWF that saves data
    // to a shared object twice and verify that the data is saved.
    let mut memory_storage_backend: Box<dyn StorageBackend> =
        Box::<MemoryStorageBackend>::default();

    // Initial run; no shared object data.
    let test1 = &Test::from_options(
        TestOptions {
            num_frames: Some(1),
            output_path: "output1.txt".into(),
            ..Default::default()
        },
        VfsPath::new(PhysicalFS::new("tests/swfs/avm1/shared_object_self_ref/")),
        "shared_object_self_ref_avm1".to_string(),
    )?;
    let mut runner = test1.create_test_runner(environment)?;

    loop {
        runner.tick();
        match runner.test()? {
            TestStatus::Continue => {}
            TestStatus::Sleep(duration) => sleep(duration),
            TestStatus::Finished => break,
        }
    }

    {
        // Save the storage backend for next run.
        let mut player = runner.player().lock().unwrap();
        std::mem::swap(player.storage_mut(), &mut memory_storage_backend);
    }

    // Verify that the flash cookie matches the expected one
    let expected = std::fs::read("tests/swfs/avm1/shared_object_self_ref/RuffleTestRef.sol")?;
    std::assert_eq!(
        expected,
        memory_storage_backend
            .get("localhost//RuffleTestRef")
            .unwrap_or_default()
    );

    // Re-run the SWF, verifying that the shared object persists.
    let test2 = &Test::from_options(
        TestOptions {
            num_frames: Some(1),
            output_path: "output2.txt".into(),
            ..Default::default()
        },
        VfsPath::new(PhysicalFS::new("tests/swfs/avm1/shared_object_self_ref/")),
        "shared_object_self_ref_avm1".to_string(),
    )?;
    let mut runner = test2.create_test_runner(environment)?;
    {
        // Swap in the previous storage backend.
        let mut player = runner.player().lock().unwrap();
        std::mem::swap(player.storage_mut(), &mut memory_storage_backend);
    }

    loop {
        runner.tick();
        match runner.test()? {
            TestStatus::Continue => {}
            TestStatus::Sleep(duration) => sleep(duration),
            TestStatus::Finished => break,
        }
    }

    Ok(())
}

pub fn shared_object_avm2(environment: &impl Environment) -> Result<(), libtest_mimic::Failed> {
    // Test SharedObject persistence. Run an SWF that saves data
    // to a shared object twice and verify that the data is saved.
    let mut memory_storage_backend: Box<dyn StorageBackend> =
        Box::<MemoryStorageBackend>::default();

    // Initial run; no shared object data.
    let test1 = &Test::from_options(
        TestOptions {
            num_frames: Some(1),
            output_path: "output1.txt".into(),
            ..Default::default()
        },
        VfsPath::new(PhysicalFS::new("tests/swfs/avm2/shared_object/")),
        "shared_object_avm2".to_string(),
    )?;
    let mut runner = test1.create_test_runner(environment)?;

    loop {
        runner.tick();
        match runner.test()? {
            TestStatus::Continue => {}
            TestStatus::Sleep(duration) => sleep(duration),
            TestStatus::Finished => break,
        }
    }

    {
        // Save the storage backend for next run.
        let mut player = runner.player().lock().unwrap();
        std::mem::swap(player.storage_mut(), &mut memory_storage_backend);
    }

    // Verify that the flash cookie matches the expected one
    let expected = std::fs::read("tests/swfs/avm2/shared_object/RuffleTest.sol")?;
    std::assert_eq!(
        expected,
        memory_storage_backend
            .get("localhost//RuffleTest")
            .unwrap_or_default()
    );

    // Re-run the SWF, verifying that the shared object persists.
    let test2 = &Test::from_options(
        TestOptions {
            num_frames: Some(1),
            output_path: "output2.txt".into(),
            ..Default::default()
        },
        VfsPath::new(PhysicalFS::new("tests/swfs/avm2/shared_object/")),
        "shared_object_avm2".to_string(),
    )?;
    let mut runner = test2.create_test_runner(environment)?;
    {
        // Swap in the previous storage backend.
        let mut player = runner.player().lock().unwrap();
        std::mem::swap(player.storage_mut(), &mut memory_storage_backend);
    }

    loop {
        runner.tick();
        match runner.test()? {
            TestStatus::Continue => {}
            TestStatus::Sleep(duration) => sleep(duration),
            TestStatus::Finished => break,
        }
    }

    Ok(())
}
