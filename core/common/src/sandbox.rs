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
    /// When the URL is remote, [`SandboxType::Remote`] is used.
    /// When the URL is local, [`SandboxType::LocalWithFile`] or
    /// [`SandboxType::LocalWithNetwork`] is used depending on
    /// the preference from the header.
    pub fn infer(url: &str, header: &HeaderExt) -> Self {
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
