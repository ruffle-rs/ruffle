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

pub fn check_shared_objects(
    player: &std::sync::Mutex<ruffle_core::Player>,
    shared_objects: &HashMap<String, SharedObjectConfig>,
    root_path: &vfs::VfsPath,
) -> Result<()> {
    for (name, so_config) in shared_objects {
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

        let expected_path = root_path.join(format!("{}.sol", so_config.expected))?;
        if !expected_path.exists()? {
            anyhow::bail!(
                "Expected SharedObject file missing: '{}'. Please ensure it exists.",
                so_config.expected
            );
        }

        let mut expected_bytes = Vec::new();
        expected_path
            .open_file()?
            .read_to_end(&mut expected_bytes)?;

        if so_config.known_failure {
            if actual_bytes.as_slice() == expected_bytes.as_slice() {
                anyhow::bail!(
                    "SharedObject test {name} was marked as a known failure, but the output matched the expected Flash output. Remove `known_failure = true`!",
                );
            }

            let ruffle_path = root_path.join(format!("{}.ruffle.sol", so_config.expected))?;

            if !ruffle_path.exists()? {
                let mut file = ruffle_path.create_file()?;
                file.write_all(actual_bytes)?;

                anyhow::bail!(
                    "Created '{}.ruffle.sol'. Please verify it and rerun the test.",
                    so_config.expected
                );
            }

            let mut ruffle_bytes = Vec::new();
            ruffle_path.open_file()?.read_to_end(&mut ruffle_bytes)?;

            if let Err(mismatch) = diff_binary(
                actual_bytes,
                &ruffle_bytes,
                &format!("Known failure SharedObject '{name}'"),
            ) {
                anyhow::bail!(
                    "{mismatch}\nThe tracked Ruffle output has changed. Please inspect and update '{}.ruffle.sol' if this change is expected.",
                    so_config.expected
                );
            }
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
