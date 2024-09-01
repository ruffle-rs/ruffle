//! Security Sandbox implementation, see
//! https://help.adobe.com/en_US/as3/dev/WS5b3ccc516d4fbf351e63e3d118a9b90204-7e3f.html

/// Type of sandbox that defines what a movie can access
/// and how movies interact with each other.
///
/// Note: sandbox type is defined *per SWF*.
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
}
