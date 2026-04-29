//! Tests running SWFs in a headless Ruffle instance.
//!
//! Trace output can be compared with correct output from the official Flash Player.

use crate::environment::NativeEnvironment;
use crate::external_interface::tests::{external_interface_avm1, external_interface_avm2};
use crate::shared_object::{shared_object_avm1, shared_object_avm2, shared_object_self_ref_avm1};
use anyhow::Context;
use clap::Parser;
use libtest_mimic::Trial;
use ruffle_fs_tests_runner::FsTestsRunner;
use ruffle_test_framework::environment::CompileMode;
use ruffle_test_framework::options::TestOptions;
use ruffle_test_framework::runner::TestStatus;
use ruffle_test_framework::test::Test;
use ruffle_test_framework::vfs::VfsPath;
use std::borrow::Cow;
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

    /// Ignore tests that are known to be failing
    #[clap(long, default_value = "compile-silently")]
    compile_mode: CompileMode,
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
    let ruffle_test_opts = RuffleTestOpts::parse_from(
        // The first argument is the name of the executable, in our case pass
        // the variable name in order to show the proper help message.
        std::iter::once("RUFFLE_TEST_OPTS=")
            .chain(ruffle_test_opts.split(" ").filter(|s| !s.is_empty())),
    );

    let env = Arc::new(NativeEnvironment::new(ruffle_test_opts.compile_mode));

    let mut runner = FsTestsRunner::new();

    let env_clone = env.clone();
    runner
        .with_descriptor_name(Cow::Borrowed(TEST_TOML_NAME))
        .with_root_dir(PathBuf::from("tests/swfs"))
        .with_test_loader(Box::new(move |params, register_trial| {
            for test in load_test_dir(&params.test_dir, params.test_name) {
                let trial = trial_for_test(&env_clone, &ruffle_test_opts, test, params.args.list);
                register_trial(trial);
            }
        }));

    // Manual tests here, since #[test] doesn't work once we use our own test harness
    let env_clone = env.clone();
    runner.with_additional_test(Trial::test("shared_object_avm1", move || {
        shared_object_avm1(&*env_clone)
    }));

    let env_clone = env.clone();
    runner.with_additional_test(Trial::test("shared_object_self_ref_avm1", move || {
        shared_object_self_ref_avm1(&*env_clone)
    }));

    let env_clone = env.clone();
    runner.with_additional_test(Trial::test("shared_object_avm2", move || {
        shared_object_avm2(&*env_clone)
    }));

    let env_clone = env.clone();
    runner.with_additional_test(Trial::test("external_interface_avm1", move || {
        external_interface_avm1(&*env_clone)
    }));

    let env_clone = env.clone();
    runner.with_additional_test(Trial::test("external_interface_avm2", move || {
        external_interface_avm2(&*env_clone)
    }));

    let conclusion = runner.run();

    // Workaround for shutdown races on slow / software GPU drivers; see
    // `NativeEnvironment::flush_gpu_with_timeout`.
    env.flush_gpu_with_timeout(std::time::Duration::from_secs(15));

    conclusion.exit()
}

fn load_test_dir<'a>(test_dir: &'a VfsPath, name: &'a str) -> impl Iterator<Item = Test> + 'a {
    let options = TestOptions::read_with_subtests(&test_dir.join("test.toml").unwrap())
        .with_context(|| format!("Couldn't load test options for test {name}"))
        .unwrap();
    options.into_iter().map(move |opts| {
        Test::from_options(opts, test_dir.to_owned(), name.to_owned())
            .with_context(|| format!("Couldn't create test {name}"))
            .unwrap()
    })
}

fn trial_for_test(
    env: &Arc<NativeEnvironment>,
    opts: &RuffleTestOpts,
    test: Test,
    list_only: bool,
) -> Trial {
    let ignore = !test.should_run(opts.ignore_known_failures, !list_only, env.as_ref());

    // Put extra info into the test 'kind' instead of appending it to the test name,
    // to not break `cargo test some/test -- --exact` and `cargo test -- --list`.
    let mut test_kind = String::new();
    if test.options.has_known_failure() {
        test_kind.push('!');
    }
    if let Some(name) = &test.options.subtest_name {
        test_kind.push_str(name);
    }

    let env = env.clone();

    let trial = Trial::test(test.name.clone(), move || {
        let mut runner = test.create_test_runner(env.as_ref())?;

        loop {
            match runner.tick()? {
                TestStatus::Continue => (),
                TestStatus::Sleep(duration) => sleep(duration),
                TestStatus::Finished => break Ok(()),
            }
        }
    });

    trial.with_ignored_flag(ignore).with_kind(test_kind)
}
