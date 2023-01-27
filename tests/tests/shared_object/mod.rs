use crate::set_logger;
use crate::util::runner::test_swf_with_hooks;
use anyhow::Result;
use ruffle_core::backend::storage::{MemoryStorageBackend, StorageBackend};
use std::path::Path;

pub fn shared_object_avm1() -> Result<()> {
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

pub fn shared_object_avm2() -> Result<()> {
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
