//! Tests running SWFs in a headless Ruffle instance.
//!
//! Trace output can be compared with correct output from the official Flash Player.

use crate::environment::NativeEnvironment;
use crate::external_interface::tests::{external_interface_avm1, external_interface_avm2};
use crate::shared_object::{shared_object_avm1, shared_object_avm2, shared_object_self_ref_avm1};
use anyhow::Context;
use anyhow::Result;
use libtest_mimic::{Arguments, Trial};
use regex::Regex;
use ruffle_test_framework::options::TestOptions;
use ruffle_test_framework::runner::TestStatus;
use ruffle_test_framework::test::{Test, TestKind};
use ruffle_test_framework::vfs::{PhysicalFS, VfsPath};
use std::panic::{catch_unwind, resume_unwind, AssertUnwindSafe};
use std::path::Path;
use std::thread::sleep;

mod environment;
mod external_interface;
mod shared_object;

const TEST_TOML_NAME: &str = "test.toml";

/// Convert the filter (e.g. from the CLI) to a test name.
///
/// These two values may differ due to how
/// libtest_mimic handles test kind annotations:
/// a test may be named `test` or `[kind] test` when a kind is present.
/// This function removes the "kind" prefix from
/// the name to match tests similarly to libtest_mimic.
///
/// See [`Arguments::is_filtered_out()`].
/// See [`libtest_mimic::TestInfo::test_name_with_kind()`].
fn filter_to_test_name(filter: &str) -> String {
    Regex::new("^\\[[^]]+] ")
        .unwrap()
        .replace(filter, "")
        .to_string()
}

fn is_candidate(args: &Arguments, test_name: &str) -> bool {
    if let Some(filter) = &args.filter {
        let expected_test_name = filter_to_test_name(filter);
        match args.exact {
            true if test_name != expected_test_name => return false,
            false if !test_name.contains(&expected_test_name) => return false,
            _ => {}
        };
    }

    for skip_filter in &args.skip {
        let skipped_test_name = filter_to_test_name(skip_filter);
        match args.exact {
            true if test_name == skipped_test_name => return false,
            false if test_name.contains(&skipped_test_name) => return false,
            _ => {}
        }
    }

    true
}

fn main() {
    let args = Arguments::from_args();

    let _ = env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info,wgpu_core=warn,wgpu_hal=warn"),
    )
    .format_timestamp(None)
    .is_test(true)
    .try_init();

    let subscriber = tracing_subscriber::fmt::Subscriber::builder()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .finish();
    // Ignore error if it's already been set
    let _ = tracing::subscriber::set_global_default(subscriber);

    // When this is true, we are looking for one specific test.
    // This is an important optimization for nextest,
    // as it executes tests one by one.
    let filter_exact = args.exact && args.filter.is_some();

    let root = Path::new("tests/swfs");
    let mut tests: Vec<Trial> = if filter_exact {
        look_up_test(root, &args).map_or_else(Vec::new, |trial| vec![trial])
    } else {
        walkdir::WalkDir::new(root)
            .into_iter()
            .map(Result::unwrap)
            .filter(|entry| entry.file_type().is_file() && entry.file_name() == TEST_TOML_NAME)
            .filter_map(|file| {
                let name = file
                    .path()
                    .parent()?
                    .strip_prefix(root)
                    .context("Couldn't strip root prefix from test dir")
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/");
                if is_candidate(&args, &name) {
                    Some(run_test(&args, file.path(), &name))
                } else {
                    None
                }
            })
            .collect()
    };

    // Manual tests here, since #[test] doesn't work once we use our own test harness
    tests.push(Trial::test("shared_object_avm1", || {
        shared_object_avm1(&NativeEnvironment)
    }));
    tests.push(Trial::test("shared_object_self_ref_avm1", || {
        shared_object_self_ref_avm1(&NativeEnvironment)
    }));
    tests.push(Trial::test("shared_object_avm2", || {
        shared_object_avm2(&NativeEnvironment)
    }));
    tests.push(Trial::test("external_interface_avm1", || {
        external_interface_avm1(&NativeEnvironment)
    }));
    tests.push(Trial::test("external_interface_avm2", || {
        external_interface_avm2(&NativeEnvironment)
    }));

    tests.sort_unstable_by_key(|t| (TestKind::ord(t.kind()), t.name().to_owned()));

    libtest_mimic::run(&args, tests).exit()
}

fn look_up_test(root: &Path, args: &Arguments) -> Option<Trial> {
    let name = filter_to_test_name(args.filter.as_ref().unwrap());
    let absolute_root = std::fs::canonicalize(root).unwrap();
    let path = absolute_root
        .join(&name)
        .join(TEST_TOML_NAME)
        .canonicalize()
        .ok()?;

    // Make sure that:
    //   1. There's no path traversal (e.g. `cargo test ../../test`)
    //   2. The path is still exact (e.g. `cargo test avm1/../avm1/test`)
    if path.strip_prefix(absolute_root).ok()? != Path::new(&name).join(TEST_TOML_NAME) {
        return None;
    }
    if path.is_file() {
        Some(run_test(args, &path, &name))
    } else {
        None
    }
}

fn run_test(args: &Arguments, file: &Path, name: &str) -> Trial {
    let root = VfsPath::new(PhysicalFS::new(file.parent().unwrap()));
    let test = Test::from_options(
        TestOptions::read(&root.join("test.toml").unwrap())
            .context("Couldn't load test options")
            .unwrap(),
        root,
        name.to_string(),
    )
    .with_context(|| format!("Couldn't create test {name}"))
    .unwrap();

    let ignore = !test.should_run(!args.list, &NativeEnvironment);
    let kind = test.kind();

    let mut trial = Trial::test(test.name.to_string(), move || {
        let test = AssertUnwindSafe(test);
        let unwind_result = catch_unwind(|| {
            let mut runner = test.create_test_runner(&NativeEnvironment)?;

            loop {
                runner.tick();
                match runner.test()? {
                    TestStatus::Continue => {}
                    TestStatus::Sleep(duration) => sleep(duration),
                    TestStatus::Finished => break,
                }
            }

            Result::<_>::Ok(())
        });
        if test.options.known_failure {
            match unwind_result {
                Ok(Ok(())) => Err(
                    format!("{} was known to be failing, but now passes successfully. Please update it and remove `known_failure = true`!", test.name).into()
                ),
                Ok(Err(_)) | Err(_) => Ok(()),
            }
        } else {
            match unwind_result {
                Ok(r) => Ok(r?),
                Err(e) => resume_unwind(e),
            }
        }
    });
    if let Some(kind) = kind {
        trial = trial.with_kind(kind.name());
    }
    if ignore {
        trial = trial.with_ignored_flag(true);
    }
    trial
}
