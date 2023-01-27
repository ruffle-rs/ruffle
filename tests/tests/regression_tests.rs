//! Tests running SWFs in a headless Ruffle instance.
//!
//! Trace output can be compared with correct output from the official Flash Player.

use crate::external_interface::tests::{external_interface_avm1, external_interface_avm2};
use crate::shared_object::{shared_object_avm1, shared_object_avm2};
use anyhow::Context;
use anyhow::Result;
use libtest_mimic::{Arguments, Trial};
use std::path::Path;
use util::test::Test;

mod external_interface;
mod shared_object;
mod util;

const RUN_IMG_TESTS: bool = cfg!(feature = "imgtests");

fn set_logger() {
    let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp(None)
        .is_test(true)
        .try_init();
}

fn main() {
    let args = Arguments::from_args();

    let root = Path::new("tests/swfs");
    let mut tests: Vec<Trial> = walkdir::WalkDir::new(root)
        .into_iter()
        .map(Result::unwrap)
        .filter(|entry| entry.file_type().is_file() && entry.file_name() == "test.toml")
        .map(|file| {
            let test = Test::from_options(file.path(), root)
                .context("Couldn't create test")
                .unwrap();
            let ignore = test.options.ignore || (test.options.image && !RUN_IMG_TESTS);
            let mut trial = Trial::test(test.name.to_string(), || test.run());
            if ignore {
                trial = trial.with_ignored_flag(true);
            }
            trial
        })
        .collect();

    // Manual tests here, since #[test] doesn't work once we use our own test harness
    tests.push(Trial::test("shared_object_avm1", || {
        shared_object_avm1().map_err(|e| e.to_string().into())
    }));
    tests.push(Trial::test("shared_object_avm2", || {
        shared_object_avm2().map_err(|e| e.to_string().into())
    }));
    tests.push(Trial::test("external_interface_avm1", || {
        external_interface_avm1().map_err(|e| e.to_string().into())
    }));
    tests.push(Trial::test("external_interface_avm2", || {
        external_interface_avm2().map_err(|e| e.to_string().into())
    }));

    tests.sort_unstable_by(|a, b| a.name().cmp(b.name()));

    libtest_mimic::run(&args, tests).exit()
}
