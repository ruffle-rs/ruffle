use std::path::{Path, PathBuf};
use url::Url;

/// Extension methods for [`Url`].
pub trait UrlExt: Sized {
    /// Converts an absolute file path into a `file:` URL.
    ///
    /// Returns `None` if the path is not absolute, or on targets without file paths.
    fn from_local_file_path(path: &Path) -> Option<Self>;

    /// Converts a `file:` URL into a file path.
    ///
    /// Returns `None` for URLs with any other scheme.
    /// Use it instead of [`Url::to_file_path`], which does not check the scheme.
    fn to_local_file_path(&self) -> Option<PathBuf>;

    /// Converts a URL into a file path regardless of its scheme.
    ///
    /// This also succeeds for some URLs with other schemes.
    /// Prefer [`UrlExt::to_local_file_path`].
    fn to_file_path_ignoring_scheme(&self) -> Option<PathBuf>;
}

impl UrlExt for Url {
    fn from_local_file_path(path: &Path) -> Option<Self> {
        // `Url::from_file_path` is only available on these targets (e.g. not on web).
        cfg_select! {
            any(
                unix,
                windows,
                target_os = "redox",
                target_os = "wasi",
                target_os = "hermit"
            ) => Url::from_file_path(path).ok(),
            _ => {
                let _ = path;
                None
            }
        }
    }

    fn to_local_file_path(&self) -> Option<PathBuf> {
        if self.scheme() != "file" {
            return None;
        }
        self.to_file_path_ignoring_scheme()
    }

    fn to_file_path_ignoring_scheme(&self) -> Option<PathBuf> {
        // `Url::to_file_path` is only available on these targets (e.g. not on web).
        cfg_select! {
            any(
                unix,
                windows,
                target_os = "redox",
                target_os = "wasi",
                target_os = "hermit"
            ) => {
                #[expect(clippy::disallowed_methods)]
                Url::to_file_path(self).ok()
            }
            _ => None,
        }
    }
}
