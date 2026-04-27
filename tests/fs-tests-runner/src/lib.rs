//! Makes it easy to discover, filter, and execute filesystem-based tests.
//!
//! Filesystem-based tests are tests that are defined in directories with
//! specific descriptor files (e.g. test.toml).
//!
//! libtest_mimic is used so that the runner is compatible with `cargo test`.

use anyhow::Context;
pub use libtest_mimic::Conclusion;
use libtest_mimic::{Arguments, Trial};
use regex::Regex;
use std::borrow::Cow;
use std::cmp::Ordering;
use std::path::Path;
use std::path::PathBuf;
use vfs::PhysicalFS;
use vfs::VfsPath;

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

fn is_candidate(test_name: &str, filter: Option<&str>, exact: bool, skip: &[String]) -> bool {
    if let Some(filter) = filter {
        let expected_test_name = filter_to_test_name(filter);
        match exact {
            true if test_name != expected_test_name => return false,
            false if !test_name.contains(&expected_test_name) => return false,
            _ => {}
        };
    }

    for skip_filter in skip {
        let skipped_test_name = filter_to_test_name(skip_filter);
        match exact {
            true if test_name == skipped_test_name => return false,
            false if test_name.contains(&skipped_test_name) => return false,
            _ => {}
        }
    }

    true
}

pub struct TestLoaderParams<'a> {
    pub test_dir: VfsPath,
    pub test_dir_real: Cow<'a, Path>,
    pub test_name: &'a str,
}

pub type TestLoader<T> = Box<dyn Fn(TestLoaderParams, &mut dyn FnMut(T))>;
pub type TestSorter<T> = Box<dyn FnMut(&T, &T) -> Ordering>;

pub struct FsTestsRunner<T> {
    root_dir: PathBuf,
    descriptor_name: Cow<'static, str>,
    additional_tests: Vec<T>,
    test_loader: Option<TestLoader<T>>,
    canonicalize_paths: bool,
    sorter: Option<TestSorter<T>>,
    exact: bool,
    filter: Option<String>,
    skip: Vec<String>,
}

impl<T> Default for FsTestsRunner<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> FsTestsRunner<T> {
    pub fn new() -> Self {
        Self {
            root_dir: PathBuf::from("tests"),
            descriptor_name: Cow::Borrowed("test.toml"),
            additional_tests: Vec::new(),
            test_loader: None,
            canonicalize_paths: false,
            sorter: None,
            exact: false,
            filter: None,
            skip: Vec::new(),
        }
    }

    pub fn with_exact(&mut self, exact: bool) -> &mut Self {
        self.exact = exact;
        self
    }

    pub fn with_filter(&mut self, filter: Option<String>) -> &mut Self {
        self.filter = filter;
        self
    }

    pub fn with_skip(&mut self, skip: Vec<String>) -> &mut Self {
        self.skip = skip;
        self
    }

    pub fn with_root_dir(&mut self, root_dir: PathBuf) -> &mut Self {
        self.root_dir = root_dir;
        self
    }

    pub fn with_descriptor_name(&mut self, descriptor_name: Cow<'static, str>) -> &mut Self {
        self.descriptor_name = descriptor_name;
        self
    }

    pub fn with_additional_test(&mut self, test: T) -> &mut Self {
        self.additional_tests.push(test);
        self
    }

    pub fn with_test_loader(&mut self, test_loader: TestLoader<T>) -> &mut Self {
        self.test_loader = Some(test_loader);
        self
    }

    pub fn with_canonicalize_paths(&mut self, canonicalize_paths: bool) -> &mut Self {
        self.canonicalize_paths = canonicalize_paths;
        self
    }

    pub fn with_sorter(&mut self, sorter: TestSorter<T>) -> &mut Self {
        self.sorter = Some(sorter);
        self
    }

    pub fn find_tests(mut self) -> Vec<T> {
        self.ensure_root_dir_exists();

        // When this is true, we are looking for one specific test.
        // This is an important optimization for nextest,
        // as it executes tests one by one.
        let filter_exact = self.exact && self.filter.is_some();

        let root = &self.root_dir;
        let mut tests = Vec::new();

        if filter_exact {
            // Ignore "errors" here.
            let _ = self.look_up_test(&mut tests);
        } else {
            let walk = walkdir::WalkDir::new(root)
                .into_iter()
                .map(Result::unwrap)
                .filter(|entry| {
                    entry.file_type().is_file()
                        && entry.file_name() == self.descriptor_name.as_ref()
                });

            for file in walk {
                let Some(parent) = file.path().parent() else {
                    continue;
                };
                let name = parent
                    .strip_prefix(root)
                    .context("Couldn't strip root prefix from test dir")
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/");
                if is_candidate(&name, self.filter.as_deref(), self.exact, &self.skip) {
                    self.load_test(file.path(), &name, &mut tests)
                }
            }
        };

        tests.append(&mut self.additional_tests);

        if let Some(sorter) = &mut self.sorter {
            tests.sort_unstable_by(sorter);
        }

        tests
    }

    fn ensure_root_dir_exists(&self) {
        if !self.root_dir.is_dir() {
            let root_dir = self.root_dir.to_string_lossy();
            panic!("The root directory does not exist or is not a directory: {root_dir}");
        }
    }

    fn look_up_test(&self, out: &mut Vec<T>) -> anyhow::Result<()> {
        let root = &self.root_dir;

        let name = filter_to_test_name(self.filter.as_ref().unwrap());
        let absolute_root = std::fs::canonicalize(root).unwrap();
        let path = absolute_root
            .join(&name)
            .join(self.descriptor_name.as_ref())
            .canonicalize()?;

        // Make sure that:
        //   1. There's no path traversal (e.g. `cargo test ../../test`)
        //   2. The path is still exact (e.g. `cargo test avm1/../avm1/test`)
        if path.strip_prefix(absolute_root)? != Path::new(&name).join(self.descriptor_name.as_ref())
        {
            return Ok(());
        }

        if path.is_file() {
            self.load_test(&path, &name, out)
        }
        Ok(())
    }

    fn load_test(&self, file: &Path, name: &str, out: &mut Vec<T>) {
        let Some(test_loader) = self.test_loader.as_ref() else {
            return;
        };
        let mut test_dir_real = Cow::Borrowed(file.parent().unwrap());
        if self.canonicalize_paths {
            test_dir_real = Cow::Owned(
                test_dir_real
                    .canonicalize()
                    .expect("Test dir canonicalization failed"),
            );
        }
        let test_dir = VfsPath::new(PhysicalFS::new(&test_dir_real));
        let params = TestLoaderParams {
            test_dir,
            test_dir_real,
            test_name: name,
        };
        test_loader(params, &mut |trial| out.push(trial));
    }
}

impl FsTestsRunner<Trial> {
    pub fn with_args_from_libtest_mimic(&mut self) -> &mut Self {
        let arguments = Arguments::from_args();
        self.exact = arguments.exact;
        self.filter = arguments.filter;
        self.skip = arguments.skip;
        self
    }

    pub fn run(self) -> Conclusion {
        let mut args = Arguments::from_args();
        args.exact = self.exact;
        args.filter = self.filter.clone();
        args.skip = self.skip.clone();
        let tests = self.find_tests();
        libtest_mimic::run(&args, tests)
    }
}
