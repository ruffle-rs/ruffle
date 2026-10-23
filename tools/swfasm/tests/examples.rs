//! Checks that the examples compile into the SWFs stored next to them.

use libtest_mimic::{Arguments, Failed, Trial};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

fn main() {
    let args = Arguments::from_args();
    let examples_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples");
    let trials = WalkDir::new(&examples_dir)
        .sort_by_file_name()
        .into_iter()
        .map(|entry| entry.expect("failed to read the examples directory"))
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "swfasm"))
        .map(|entry| {
            let path = entry.into_path();
            let name = path
                .strip_prefix(&examples_dir)
                .unwrap()
                .with_extension("")
                .to_string_lossy()
                .replace('\\', "/");
            Trial::test(name, move || run_test(&path))
        })
        .collect();
    libtest_mimic::run(&args, trials).exit()
}

fn run_test(source_path: &Path) -> Result<(), Failed> {
    let expected_path = source_path.with_extension("swf");
    let regenerate = || {
        format!(
            "regenerate it with `cargo run -p swfasm -- {}`",
            relative_to_workspace(source_path).display()
        )
    };

    let source = std::fs::read_to_string(source_path)?;
    let actual = swfasm::compile_str(&source).map_err(|e| format!("{e:#}"))?;
    let expected = std::fs::read(&expected_path).map_err(|e| {
        format!(
            "failed to read {}: {e}, {}",
            relative_to_workspace(&expected_path).display(),
            regenerate()
        )
    })?;

    check_same(&actual, &expected, &expected_path)
}

fn check_same(actual: &[u8], expected: &[u8], expected_path: &Path) -> Result<(), Failed> {
    if actual != expected {
        let offset = actual
            .iter()
            .zip(expected)
            .position(|(a, b)| a != b)
            .unwrap_or(actual.len().min(expected.len()));
        return Err(format!(
            "the compiled SWF ({} bytes) differs from {} ({} bytes), starting at offset {offset}",
            actual.len(),
            relative_to_workspace(expected_path).display(),
            expected.len()
        )
        .into());
    }
    Ok(())
}

fn relative_to_workspace(path: &Path) -> PathBuf {
    let workspace_dir = Path::new(env!("CARGO_MANIFEST_DIR")).ancestors().nth(2);
    workspace_dir
        .and_then(|dir| path.strip_prefix(dir).ok())
        .unwrap_or(path)
        .to_path_buf()
}
