use crate::{find_root_dir, load_test_dir};
use clap::Args;
use ruffle_fs_tests_runner::FsTestsRunner;
use ruffle_test_framework::environment::CompileMode;

#[derive(Args)]
pub struct CompileOptions {}

pub fn main_compile(_compile_options: CompileOptions) {
    let mut runner = FsTestsRunner::new();
    runner
        .with_root_dir(find_root_dir())
        .with_test_loader(Box::new(move |params, register_trial| {
            for test in load_test_dir(&params.test_dir, params.test_name) {
                if !test.options.compilers.is_empty() {
                    register_trial(test);
                }
            }
        }))
        .with_sorter(Box::new(|a, b| a.name().cmp(b.name())));
    let tests = runner.find_tests();
    for test in tests {
        if let Some(subtest) = &test.options.subtest_name {
            println!("Compiling {} ([{}])", test.name(), subtest);
        } else {
            println!("Compiling {}", test.name());
        }
        test.compile(CompileMode::CompileSilently).unwrap();
    }
    println!("Done!");
}
