//! Security Sandbox implementation, see
//! https://help.adobe.com/en_US/as3/dev/WS5b3ccc516d4fbf351e63e3d118a9b90204-7e3f.html

use swf::HeaderExt;
use url::Url;

/// Type of sandbox that defines what a movie can access
/// and how movies interact with each other.
///
/// Note: sandbox type is defined *per SWF*.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SandboxType {
    /// The movie originates from a remote URL.
    ///
    /// In this case domain-based sandbox rules are used,
    /// no filesystem access.
    Remote,

    /// The movie is a local movie with filesystem access.
    ///
    /// This implies no network access.
    LocalWithFile,

    /// The movie is a local movie with network access.
    ///
    /// This implies no filesystem access.
    LocalWithNetwork,

    /// The movie is a trusted local movie with access to both filesystem and network.
    LocalTrusted,

    /// The movie is an AIR application with access to both filesystem and network.
    Application,
}

impl SandboxType {
    /// Infer sandbox type based on SWF URL and its header.
    ///
    /// When emulating AIR, `air_application_url` is the URL of the
    /// application's SWF. SWFs in its directory (or a subdirectory) are part
    /// of the application, so [`SandboxType::Application`] is used for them.
    ///
    /// Otherwise, when the URL is remote, [`SandboxType::Remote`] is used.
    /// When the URL is local, [`SandboxType::LocalWithFile`] or
    /// [`SandboxType::LocalWithNetwork`] is used depending on
    /// the preference from the header.
    pub fn infer(url: &str, header: &HeaderExt, air_application_url: Option<&str>) -> Self {
        if air_application_url.is_some_and(|app_url| is_in_directory_of(url, app_url)) {
            return Self::Application;
        }

        match Url::parse(url) {
            Ok(url) => {
                if url.scheme() == "file" {
                    if header.use_network_sandbox() {
                        Self::LocalWithNetwork
                    } else {
                        Self::LocalWithFile
                    }
                } else {
                    Self::Remote
                }
            }
            Err(e) => {
                let sandbox_type = Self::LocalWithFile;
                tracing::warn!("Failed to parse URL {url}: {e}, using {sandbox_type:?}");
                sandbox_type
            }
        }
    }
}

/// Whether `url` points into the directory containing `base_url`.
fn is_in_directory_of(url: &str, base_url: &str) -> bool {
    let (Ok(url), Ok(base_url)) = (Url::parse(url), Url::parse(base_url)) else {
        return false;
    };
    let Ok(directory) = base_url.join(".") else {
        return false;
    };

    url.scheme() == directory.scheme()
        && url.host() == directory.host()
        && url.port_or_known_default() == directory.port_or_known_default()
        && url.path().starts_with(directory.path())
}

#[cfg(test)]
mod tests {
    use super::is_in_directory_of;

    const APP: &str = "file:///games/app/main.swf";

    #[test]
    fn in_directory_of() {
        assert!(is_in_directory_of("file:///games/app/main.swf", APP));
        assert!(is_in_directory_of("file:///games/app/child.swf", APP));
        assert!(is_in_directory_of("file:///games/app/levels/1.swf", APP));
        assert!(is_in_directory_of(
            "file:///games/app/levels/../child.swf",
            APP
        ));

        assert!(!is_in_directory_of("file:///games/child.swf", APP));
        assert!(!is_in_directory_of("file:///games/app2/child.swf", APP));
        assert!(!is_in_directory_of("file:///games/app/../child.swf", APP));
        assert!(!is_in_directory_of(
            "http://localhost/games/app/child.swf",
            APP
        ));
        assert!(!is_in_directory_of("not a url", APP));

        let remote = "http://example.com/app/main.swf";
        assert!(is_in_directory_of(
            "http://example.com/app/child.swf",
            remote
        ));
        assert!(is_in_directory_of(
            "http://example.com:80/app/child.swf",
            remote
        ));
        assert!(!is_in_directory_of(
            "http://example.com:8080/app/child.swf",
            remote
        ));
        assert!(!is_in_directory_of(
            "https://example.com/app/child.swf",
            remote
        ));
        assert!(!is_in_directory_of(
            "http://other.com/app/child.swf",
            remote
        ));
    }
}
