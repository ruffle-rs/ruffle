use crate::set_logger;
use crate::util::options::TestOptions;
use crate::util::test::Test;
use ruffle_core::backend::storage::{MemoryStorageBackend, StorageBackend};
use std::path::Path;

pub fn shared_object_avm1() -> Result<(), libtest_mimic::Failed> {
    set_logger();
    // Test SharedObject persistence. Run an SWF that saves data
    // to a shared object twice and verify that the data is saved.
    let mut memory_storage_backend: Box<dyn StorageBackend> =
        Box::<MemoryStorageBackend>::default();

    // Initial run; no shared object data.
    Test::from_options(
        TestOptions {
            num_frames: Some(1),
            output_path: "output1.txt".into(),
            ..Default::default()
        },
        Path::new("tests/swfs/avm1/shared_object/"),
        "shared_object_avm1".to_string(),
    )?
    .run(
        |_| Ok(()),
        |player| {
            // Save the storage backend for next run.
            let mut player = player.lock().unwrap();
            std::mem::swap(player.storage_mut(), &mut memory_storage_backend);
            Ok(())
        },
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
    Test::from_options(
        TestOptions {
            num_frames: Some(1),
            output_path: "output2.txt".into(),
            ..Default::default()
        },
        Path::new("tests/swfs/avm1/shared_object/"),
        "shared_object_avm1".to_string(),
    )?
    .run(
        |player| {
            // Swap in the previous storage backend.
            let mut player = player.lock().unwrap();
            std::mem::swap(player.storage_mut(), &mut memory_storage_backend);
            Ok(())
        },
        |_| Ok(()),
    )?;

    Ok(())
}

pub fn shared_object_avm2() -> Result<(), libtest_mimic::Failed> {
    set_logger();
    // Test SharedObject persistence. Run an SWF that saves data
    // to a shared object twice and verify that the data is saved.
    let mut memory_storage_backend: Box<dyn StorageBackend> =
        Box::<MemoryStorageBackend>::default();

    // Initial run; no shared object data.
    Test::from_options(
        TestOptions {
            num_frames: Some(1),
            output_path: "output1.txt".into(),
            ..Default::default()
        },
        Path::new("tests/swfs/avm2/shared_object/"),
        "shared_object_avm2".to_string(),
    )?
    .run(
        |_player| Ok(()),
        |player| {
            // Save the storage backend for next run.
            let mut player = player.lock().unwrap();
            std::mem::swap(player.storage_mut(), &mut memory_storage_backend);
            Ok(())
        },
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
    Test::from_options(
        TestOptions {
            num_frames: Some(1),
            output_path: "output2.txt".into(),
            ..Default::default()
        },
        Path::new("tests/swfs/avm2/shared_object/"),
        "shared_object_avm2".to_string(),
    )?
    .run(
        |player| {
            // Swap in the previous storage backend.
            let mut player = player.lock().unwrap();
            std::mem::swap(player.storage_mut(), &mut memory_storage_backend);
            Ok(())
        },
        |_player| Ok(()),
    )?;

    Ok(())
}
