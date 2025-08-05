//! Makes it easy to discover, filter, and execute filesystem-based tests.
//!
//! Filesystem-based tests are tests that are defined in directories with
//! specific descriptor files (e.g. test.toml).
//!
//! libtest_mimic is used so that the runner is compatible with `cargo test`.

use anyhow::Context;
use libtest_mimic::{Arguments, Trial};
use regex::Regex;
use std::borrow::Cow;
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

pub struct TestLoaderParams<'a> {
    pub args: &'a Arguments,
    pub test_dir: VfsPath,
    pub test_dir_real: Cow<'a, Path>,
    pub test_name: &'a str,
}

pub type TestLoader = Box<dyn Fn(TestLoaderParams) -> Option<Trial>>;

pub struct FsTestsRunner {
    root_dir: PathBuf,
    descriptor_name: Cow<'static, str>,
    additional_tests: Vec<Trial>,
    test_loader: Option<TestLoader>,
    canonicalize_paths: bool,
}

impl Default for FsTestsRunner {
    fn default() -> Self {
        Self::new()
    }
}

impl FsTestsRunner {
    pub fn new() -> Self {
        Self {
            root_dir: PathBuf::from("tests"),
            descriptor_name: Cow::Borrowed("test.toml"),
            additional_tests: Vec::new(),
            test_loader: None,
            canonicalize_paths: false,
        }
    }

    pub fn with_root_dir(&mut self, root_dir: PathBuf) -> &mut Self {
        self.root_dir = root_dir;
        self
    }

    pub fn with_descriptor_name(&mut self, descriptor_name: Cow<'static, str>) -> &mut Self {
        self.descriptor_name = descriptor_name;
        self
    }

    pub fn with_additional_test(&mut self, test: Trial) -> &mut Self {
        self.additional_tests.push(test);
        self
    }

    pub fn with_test_loader(&mut self, test_loader: TestLoader) -> &mut Self {
        self.test_loader = Some(test_loader);
        self
    }

    pub fn with_canonicalize_paths(&mut self, canonicalize_paths: bool) -> &mut Self {
        self.canonicalize_paths = canonicalize_paths;
        self
    }

    pub fn run(mut self) -> ! {
        self.ensure_root_dir_exists();

        let args = Arguments::from_args();

        // When this is true, we are looking for one specific test.
        // This is an important optimization for nextest,
        // as it executes tests one by one.
        let filter_exact = args.exact && args.filter.is_some();

        let root = &self.root_dir;
        let mut tests: Vec<Trial> = if filter_exact {
            self.look_up_test(&args)
                .map_or_else(Vec::new, |trial| vec![trial])
        } else {
            walkdir::WalkDir::new(root)
                .into_iter()
                .map(Result::unwrap)
                .filter(|entry| {
                    entry.file_type().is_file()
                        && entry.file_name() == self.descriptor_name.as_ref()
                })
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
                        self.load_test(&args, file.path(), &name)
                    } else {
                        None
                    }
                })
                .collect()
        };

        tests.append(&mut self.additional_tests);

        tests.sort_unstable_by(|a, b| a.name().cmp(b.name()));

        libtest_mimic::run(&args, tests).exit()
    }

    fn ensure_root_dir_exists(&self) {
        if !self.root_dir.is_dir() {
            let root_dir = self.root_dir.to_string_lossy();
            panic!("The root directory does not exist or is not a directory: {root_dir}");
        }
    }

    fn look_up_test(&self, args: &Arguments) -> Option<Trial> {
        let root = &self.root_dir;

        let name = filter_to_test_name(args.filter.as_ref().unwrap());
        let absolute_root = std::fs::canonicalize(root).unwrap();
        let path = absolute_root
            .join(&name)
            .join(self.descriptor_name.as_ref())
            .canonicalize()
            .ok()?;

        // Make sure that:
        //   1. There's no path traversal (e.g. `cargo test ../../test`)
        //   2. The path is still exact (e.g. `cargo test avm1/../avm1/test`)
        if path.strip_prefix(absolute_root).ok()?
            != Path::new(&name).join(self.descriptor_name.as_ref())
        {
            return None;
        }

        if path.is_file() {
            self.load_test(args, &path, &name)
        } else {
            None
        }
    }

    fn load_test(&self, args: &Arguments, file: &Path, name: &str) -> Option<Trial> {
        let test_loader = self.test_loader.as_ref()?;
        let mut test_dir_real = Cow::Borrowed(file.parent().unwrap());
        if self.canonicalize_paths {
            test_dir_real = Cow::Owned(
                test_dir_real
                    .canonicalize()
                    .expect("Test dir canonicalization failed"),
            );
        }
        let test_dir = VfsPath::new(PhysicalFS::new(&test_dir_real));
        test_loader(TestLoaderParams {
            args,
            test_dir,
            test_dir_real,
            test_name: name,
        })
    }
}
