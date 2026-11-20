use std::sync::Arc;

use crate::{TestFilterOptions, find_root_dir, load_test_dir};
use clap::Args;
use ruffle_fs_tests_runner::FsTestsRunner;
use ruffle_test_framework::environment::CompileMode;

#[derive(Args)]
pub struct CompileOptions {
    #[command(flatten)]
    filter: TestFilterOptions,
}

pub fn main_compile(compile_options: CompileOptions) {
    let mut runner = FsTestsRunner::new();
    compile_options.filter.apply(&mut runner);
    runner
        .with_root_dir(find_root_dir())
        .with_test_loader(Box::new(move |params, register_trial| {
            let mut tests = load_test_dir(&params.test_dir, params.test_name);
            if let Some(first) = tests.next() {
                // Only load one subtest per dir; all subtests will always have the same shared options.
                for other in tests {
                    assert!(Arc::ptr_eq(&first.options.shared, &other.options.shared));
                }

                if !first.options.shared.compilers.is_empty() {
                    register_trial(first);
                }
            }
        }))
        .with_sorter(Box::new(|a, b| a.name().cmp(b.name())));
    let tests = runner.find_tests();
    for test in tests {
        println!("Compiling {}", test.name());
        test.compile(CompileMode::CompileSilently).unwrap();
    }
    println!("Done!");
}
