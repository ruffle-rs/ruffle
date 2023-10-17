// Despite being the older method of defining modules, this is required for test modules
// https://doc.rust-lang.org/book/ch11-03-test-organization.html

pub mod environment;
pub mod fs_commands;
pub mod image_trigger;
pub mod navigator;
pub mod options;
pub mod runner;
pub mod test;
pub mod test_ui;

/// Wrapper around string slice that makes debug output `{:?}` to print string same way as `{}`.
/// Used in different `assert*!` macros in combination with `pretty_assertions` crate to make
/// test failures to show nice diffs.
/// Courtesy of https://github.com/colin-kiegel/rust-pretty-assertions/issues/24
#[derive(PartialEq, Eq)]
#[doc(hidden)]
pub struct PrettyString<'a>(pub &'a str);

/// Make diff to display string as multi-line string
impl<'a> std::fmt::Debug for PrettyString<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(self.0)
    }
}
