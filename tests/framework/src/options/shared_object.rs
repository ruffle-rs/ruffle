use crate::backends::TestStorageBackend;
use crate::options::SharedObjectConfig;
use anyhow::Result;
use std::collections::HashMap;
use std::io::{Read, Write};

fn diff_binary(actual: &[u8], expected: &[u8], label: &str) -> Result<()> {
    if actual == expected {
        return Ok(());
    }

    let mut message = format!(
        "{label} mismatch. Actual length: {}, Expected length: {}.",
        actual.len(),
        expected.len()
    );

    if let Some((idx, (a, b))) = actual
        .iter()
        .zip(expected.iter())
        .enumerate()
        .find(|(_, (a, b))| a != b)
    {
        message.push_str(&format!(
            "\nFirst mismatch at index {idx}: Actual 0x{a:02x}, Expected 0x{b:02x}",
        ));
    } else {
        message.push_str(&format!(
            "\nLengths differ. Mismatch starts at index {}",
            actual.len().min(expected.len())
        ));
    }

    anyhow::bail!(message)
}

fn check_ruffle_output(
    actual_bytes: &[u8],
    root_path: &vfs::VfsPath,
    expected: &str,
    label: &str,
) -> Result<()> {
    let ruffle_path = root_path.join(format!("{expected}.ruffle.sol"))?;

    if !ruffle_path.exists()? {
        let mut file = ruffle_path.create_file()?;
        file.write_all(actual_bytes)?;

        anyhow::bail!(
            "Created '{}.ruffle.sol'. Please verify it and rerun the test.",
            expected
        );
    }

    let mut ruffle_bytes = Vec::new();
    ruffle_path.open_file()?.read_to_end(&mut ruffle_bytes)?;

    if let Err(mismatch) = diff_binary(actual_bytes, &ruffle_bytes, label) {
        anyhow::bail!(
            "{mismatch}\nThe tracked Ruffle output has changed. Please inspect and update '{}.ruffle.sol' if this change is expected.",
            expected
        );
    }

    Ok(())
}

pub fn check_shared_objects(
    player: &std::sync::Mutex<ruffle_core::Player>,
    shared_objects: &HashMap<String, SharedObjectConfig>,
    root_path: &vfs::VfsPath,
) -> Result<()> {
    for (name, so_config) in shared_objects {
        let Some(expected) = so_config.expected.as_deref() else {
            continue;
        };

        let player_lock = player.lock().unwrap();

        let test_storage =
            <dyn std::any::Any>::downcast_ref::<TestStorageBackend>(player_lock.storage())
                .expect("Storage backend must be a TestStorageBackend");

        let stored_lsos = test_storage.get_stored_data();

        let actual_bytes = match stored_lsos.get(name) {
            Some(bytes) => bytes,
            None => {
                let available_keys: Vec<_> = stored_lsos.keys().cloned().collect();
                anyhow::bail!(
                    "SharedObject '{name}' was expected by the test config but wasn't created by the SWF.\nAvailable SharedObjects in storage: {available_keys:?}"
                );
            }
        };

        if so_config.nondeterministic {
            let flash_path = root_path.join(format!("{expected}.sol"))?;
            if flash_path.exists()? {
                anyhow::bail!(
                    "SharedObject '{name}' is marked as non-deterministic, so the Flash fixture '{}.sol' must not exist. Keep only '{}.ruffle.sol'.",
                    expected,
                    expected
                );
            }

            check_ruffle_output(
                actual_bytes,
                root_path,
                expected,
                &format!("Non-deterministic SharedObject '{name}'"),
            )?;
            continue;
        }

        let expected_path = root_path.join(format!("{expected}.sol"))?;
        if !expected_path.exists()? {
            anyhow::bail!(
                "Expected SharedObject file missing: '{}'. Please ensure it exists.",
                expected
            );
        }

        let mut expected_bytes = Vec::new();
        expected_path
            .open_file()?
            .read_to_end(&mut expected_bytes)?;

        if so_config.known_failure == Some(true) {
            if actual_bytes.as_slice() == expected_bytes.as_slice() {
                anyhow::bail!(
                    "SharedObject test {name} was marked as a known failure, but the output matched the expected Flash output. Remove `known_failure = true`!",
                );
            }

            check_ruffle_output(
                actual_bytes,
                root_path,
                expected,
                &format!("Known failure SharedObject '{name}'"),
            )?;
        } else {
            diff_binary(
                actual_bytes,
                &expected_bytes,
                &format!("SharedObject '{name}'"),
            )?;
        }
    }
    Ok(())
}
