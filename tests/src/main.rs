mod compile;

use crate::compile::{CompileOptions, main_compile};
use anyhow::Context;
use clap::{Args, Parser, Subcommand};
use ruffle_fs_tests_runner::FsTestsRunner;
use ruffle_test_framework::options::TestOptions;
use ruffle_test_framework::test::Test;
use ruffle_test_framework::vfs::VfsPath;
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Opt {
    #[command(subcommand)]
    command: Commands,
}

// Copied out of libtest_mimic to be similar enough that you can hop between actual tests & this util CLI.
#[derive(Args)]
struct TestFilterOptions {
    /// If set, filters are matched exactly rather than by substring.
    #[arg(
        long = "exact",
        help = "Exactly match filters rather than by substring."
    )]
    pub exact: bool,

    /// A list of filters. Tests whose names contain parts of any of these
    /// filters are skipped.
    #[arg(
        long = "skip",
        value_name = "FILTER",
        help = "Skip tests whose names contain FILTER (this flag can be used multiple times)."
    )]
    pub skip: Vec<String>,

    /// Filter string. Only tests which contain this string are run.
    #[arg(
        value_name = "FILTER",
        help = "The FILTER string is tested against the name of all tests, and only those tests \
                 whose names contain the filter are run."
    )]
    pub filter: Option<String>,
}

impl TestFilterOptions {
    pub fn apply<T>(&self, runner: &mut FsTestsRunner<T>) {
        runner.with_filter(self.filter.clone());
        runner.with_skip(self.skip.clone());
        runner.with_exact(self.exact);
    }
}

#[derive(Subcommand)]
enum Commands {
    /// Compile test assets from their source files.
    Compile(CompileOptions),
}

fn main() {
    let opt = Opt::parse();

    match opt.command {
        Commands::Compile(compile_options) => {
            main_compile(compile_options);
        }
    }
}

pub fn load_test_dir<'a>(test_dir: &'a VfsPath, name: &'a str) -> impl Iterator<Item = Test> + 'a {
    let options = TestOptions::read_with_subtests(&test_dir.join("test.toml").unwrap())
        .with_context(|| format!("Couldn't load test options for test {name}"))
        .unwrap();
    options.into_iter().map(move |opts| {
        Test::from_options(opts, test_dir.to_owned(), name.to_owned())
            .with_context(|| format!("Couldn't create test {name}"))
            .unwrap()
    })
}

pub fn find_root_dir() -> PathBuf {
    let path: PathBuf = std::env::var_os("LOCAL_RUFFLE_TESTS_SWFS_DIR")
        .expect("LOCAL_RUFFLE_TESTS_SWFS_DIR not set")
        .into();
    if !path.is_dir() {
        panic!("LOCAL_RUFFLE_TESTS_SWFS_DIR is not a directory!");
    }
    path
}
