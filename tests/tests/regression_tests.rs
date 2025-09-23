//! Tests running SWFs in a headless Ruffle instance.
//!
//! Trace output can be compared with correct output from the official Flash Player.

use crate::environment::NativeEnvironment;
use crate::external_interface::tests::{external_interface_avm1, external_interface_avm2};
use crate::shared_object::{shared_object_avm1, shared_object_avm2, shared_object_self_ref_avm1};
use anyhow::Context;
use anyhow::Result;
use clap::Parser;
use libtest_mimic::Trial;
use ruffle_fs_tests_runner::{FsTestsRunner, TestLoaderParams};
use ruffle_test_framework::options::TestOptions;
use ruffle_test_framework::runner::TestStatus;
use ruffle_test_framework::test::Test;
use std::borrow::Cow;
use std::panic::{catch_unwind, resume_unwind, AssertUnwindSafe};
use std::path::PathBuf;
use std::sync::Arc;
use std::thread::sleep;

mod environment;
mod external_interface;
mod shared_object;

const TEST_TOML_NAME: &str = "test.toml";

/// CLI options for running Ruffle tests, separate from cargo test's interface.
#[derive(Parser, Debug, Clone)]
struct RuffleTestOpts {
    /// Ignore tests that are known to be failing
    #[clap(long, action)]
    ignore_known_failures: bool,
}

fn main() {
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

    let ruffle_test_opts = match std::env::var("RUFFLE_TEST_OPTS") {
        Ok(val) => val,
        Err(std::env::VarError::NotPresent) => "".to_owned(),
        e @ Err(_) => panic!("{e:?}"),
    };
    let ruffle_test_opts = Arc::new(RuffleTestOpts::parse_from(
        // The first argument is the name of the executable, in our case pass
        // the variable name in order to show the proper help message.
        std::iter::once("RUFFLE_TEST_OPTS=")
            .chain(ruffle_test_opts.split(" ").filter(|s| !s.is_empty())),
    ));

    let mut runner = FsTestsRunner::new();

    runner
        .with_descriptor_name(Cow::Borrowed(TEST_TOML_NAME))
        .with_root_dir(PathBuf::from("tests/swfs"))
        .with_test_loader(Box::new(move |params| {
            Some(load_test(ruffle_test_opts.clone(), params))
        }));

    // Manual tests here, since #[test] doesn't work once we use our own test harness
    runner.with_additional_test(Trial::test("shared_object_avm1", || {
        shared_object_avm1(&NativeEnvironment)
    }));
    runner.with_additional_test(Trial::test("shared_object_self_ref_avm1", || {
        shared_object_self_ref_avm1(&NativeEnvironment)
    }));
    runner.with_additional_test(Trial::test("shared_object_avm2", || {
        shared_object_avm2(&NativeEnvironment)
    }));
    runner.with_additional_test(Trial::test("external_interface_avm1", || {
        external_interface_avm1(&NativeEnvironment)
    }));
    runner.with_additional_test(Trial::test("external_interface_avm2", || {
        external_interface_avm2(&NativeEnvironment)
    }));

    runner.run()
}

fn load_test(opts: Arc<RuffleTestOpts>, params: TestLoaderParams) -> Trial {
    let args = params.args;
    let root = params.test_dir;
    let name = params.test_name;

    let test = Test::from_options(
        TestOptions::read(&root.join("test.toml").unwrap())
            .context("Couldn't load test options")
            .unwrap(),
        root,
        name.to_string(),
    )
    .with_context(|| format!("Couldn't create test {name}"))
    .unwrap();

    let ignore = !test.should_run(opts.ignore_known_failures, !args.list, &NativeEnvironment);

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
