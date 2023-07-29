//! Tests running SWFs in a headless Ruffle instance.
//!
//! Trace output can be compared with correct output from the official Flash Player.

use crate::external_interface::tests::{external_interface_avm1, external_interface_avm2};
use crate::shared_object::{shared_object_avm1, shared_object_avm2, shared_object_self_ref_avm1};
use anyhow::Context;
use anyhow::Result;
use libtest_mimic::{Arguments, Trial};
use std::panic::{catch_unwind, resume_unwind};
use std::path::Path;
use util::test::Test;

mod external_interface;
mod shared_object;
mod util;

fn set_logger() {
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
}

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

    let root = Path::new("tests/swfs");
    let mut tests: Vec<Trial> = walkdir::WalkDir::new(root)
        .into_iter()
        .map(Result::unwrap)
        .filter(|entry| entry.file_type().is_file() && entry.file_name() == "test.toml")
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
                let test = Test::from_options_file(file.path(), name)
                    .context("Couldn't create test")
                    .unwrap();
                let ignore = !test.should_run(!args.list);
                let mut trial = Trial::test(test.name.to_string(), move || {
                    let unwind_result = catch_unwind(|| test.run(|_| Ok(()), |_| Ok(())));
                    if test.options.known_failure {
                        match unwind_result {
                            Ok(Ok(())) => Err(
                                format!("{} was known to be failing, but now passes successfully. Please update it and remove `known_failure = true`!", test.name).into()
                            ),
                            Ok(Err(_)) | Err(_) => Ok(()),
                        }
                    } else {
                        match unwind_result {
                            Ok(r) => r,
                            Err(e) => resume_unwind(e),
                        }
                    }
                });
                if ignore {
                    trial = trial.with_ignored_flag(true);
                }
                Some(trial)
            } else {
                None
            }
        })
        .collect();

    // Manual tests here, since #[test] doesn't work once we use our own test harness
    tests.push(Trial::test("shared_object_avm1", shared_object_avm1));
    tests.push(Trial::test(
        "shared_object_self_ref_avm1",
        shared_object_self_ref_avm1,
    ));
    tests.push(Trial::test("shared_object_avm2", shared_object_avm2));
    tests.push(Trial::test(
        "external_interface_avm1",
        external_interface_avm1,
    ));
    tests.push(Trial::test(
        "external_interface_avm2",
        external_interface_avm2,
    ));

    tests.sort_unstable_by(|a, b| a.name().cmp(b.name()));

    libtest_mimic::run(&args, tests).exit()
}
