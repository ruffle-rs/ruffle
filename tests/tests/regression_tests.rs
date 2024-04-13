//! Tests running SWFs in a headless Ruffle instance.
//!
//! Trace output can be compared with correct output from the official Flash Player.

use crate::environment::NativeEnvironment;
use crate::external_interface::tests::{external_interface_avm1, external_interface_avm2};
use crate::shared_object::{shared_object_avm1, shared_object_avm2, shared_object_self_ref_avm1};
use anyhow::Context;
use anyhow::Result;
use libtest_mimic::{Arguments, Trial};
use ruffle_test_framework::options::TestOptions;
use ruffle_test_framework::runner::TestStatus;
use ruffle_test_framework::test::Test;
use ruffle_test_framework::vfs::{PhysicalFS, VfsPath};
use std::panic::{catch_unwind, resume_unwind, AssertUnwindSafe};
use std::path::Path;
use std::thread::sleep;

mod environment;
mod external_interface;
mod shared_object;

const TEST_TOML_NAME: &str = "test.toml";

fn is_candidate(args: &Arguments, test_name: &str) -> bool {
    if let Some(filter) = &args.filter {
        match args.exact {
            true if test_name != filter => return false,
            false if !test_name.contains(filter) => return false,
            _ => {}
        };
    }

    for skip_filter in &args.skip {
        match args.exact {
            true if test_name == skip_filter => return false,
            false if test_name.contains(skip_filter) => return false,
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
        let name = args.filter.as_ref().unwrap();
        let absolute_root = std::fs::canonicalize(root).unwrap();
        let path = absolute_root
            .join(name)
            .join(TEST_TOML_NAME)
            .canonicalize()
            .expect("test should exist");
        path.strip_prefix(absolute_root).expect("path traversal");
        if path.is_file() {
            vec![run_test(&args, &path, name)]
        } else {
            vec![]
        }
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

    tests.sort_unstable_by(|a, b| a.name().cmp(b.name()));

    libtest_mimic::run(&args, tests).exit()
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
    if ignore {
        trial = trial.with_ignored_flag(true);
    }
    trial
}
