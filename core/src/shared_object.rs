//! Shared object logic common to AVM1 and AVM2.

/// The keys of a local shared object.
#[derive(Debug, PartialEq)]
pub struct LocalSharedObjectKey {
    instance: String,
    storage: String,
}

impl LocalSharedObjectKey {
    /// `getLocal` returns the same object for the same instance key.
    pub fn instance(&self) -> &str {
        &self.instance
    }

    /// The key the shared object is stored under.
    pub fn storage(&self) -> &str {
        &self.storage
    }
}

/// Returns the keys of the local shared object accessed by
/// `SharedObject.getLocal(name, local_path, secure)` from a movie at `movie_url`,
/// or `None` when the shared object can't be created.
pub fn local_shared_object_key(
    movie_url: &str,
    name: &str,
    local_path: Option<&str>,
    secure: bool,
) -> Option<LocalSharedObjectKey> {
    // TODO: It appears that Flash does some kind of escaping here:
    // the name "foo\uD800" correspond to a file named "fooE#FB#FB#D.sol".

    const INVALID_CHARS: &str = "~%&\\;:\"',<>?# ";
    if name.contains(|c| INVALID_CHARS.contains(c)) {
        tracing::error!("SharedObject::get_local: Invalid character in name");
        return None;
    }

    let mut movie_url = if let Ok(url) = url::Url::parse(movie_url) {
        url
    } else {
        tracing::error!("SharedObject::get_local: Unable to parse movie URL");
        return None;
    };
    movie_url.set_query(None);
    movie_url.set_fragment(None);

    // Secure parameter disallows using the shared object from non-HTTPS.
    if secure && movie_url.scheme() != "https" {
        tracing::warn!(
            "SharedObject.get_local: Tried to load a secure shared object from non-HTTPS origin"
        );
        return None;
    }

    // Shared objects are sandboxed per-domain.
    // By default, they are keyed based on the SWF URL, but the `localHost` parameter can modify this path.
    let mut movie_path = movie_url.path();
    // Remove leading/trailing slashes.
    movie_path = movie_path.strip_prefix('/').unwrap_or(movie_path);
    movie_path = movie_path.strip_suffix('/').unwrap_or(movie_path);

    let movie_host = if movie_url.scheme() == "file" {
        // Remove drive letter on Windows. This is observable, as `localPath` is
        // validated against the path without the drive letter.
        // Flash Player on Linux removes the first path component instead
        // (`/home/user/game.swf` becomes `user/game.swf`), which looks like
        // a bug that came from the drive letter logic. We don't replicate it.
        if let [_, b':', b'/', ..] = movie_path.as_bytes() {
            movie_path = &movie_path[3..];
        }
        "localhost"
    } else {
        movie_url.host_str().unwrap_or_default()
    };

    let local_path = if let Some(local_path) = local_path {
        // Empty local path always fails.
        if local_path.is_empty() {
            return None;
        }

        // Remove leading/trailing slashes.
        let local_path = local_path.strip_suffix('/').unwrap_or(local_path);
        let local_path = local_path.strip_prefix('/').unwrap_or(local_path);

        // Verify that local_path is a prefix of the SWF path.
        if movie_path.starts_with(local_path)
            && (local_path.is_empty()
                || movie_path.len() == local_path.len()
                || movie_path[local_path.len()..].starts_with('/'))
        {
            local_path
        } else {
            tracing::warn!("SharedObject.get_local: localPath parameter does not match SWF path");
            return None;
        }
    } else {
        movie_path
    };

    // Final SO path: foo.com/folder/game.swf/SOName
    let path = format!("{movie_host}/{local_path}");

    // Avoid any paths with `..` to prevent SWFs from crawling the file system on desktop.
    if path.split('/').any(|s| matches!(s, "." | "..")) {
        tracing::error!("SharedObject.get_local: Invalid path with . or .. segments");
        return None;
    }

    let Some(storage_name) = storage_name(name) else {
        tracing::error!("SharedObject.get_local: Invalid name");
        return None;
    };

    Some(LocalSharedObjectKey {
        instance: format!("{path}/{}", instance_name(name)),
        storage: format!("{path}/{storage_name}"),
    })
}

/// Returns the name that the shared object `name` is stored under, or `None` when it's invalid.
///
/// Like in Flash Player, folders named `.` are skipped, and folders named `..` or empty ones
/// (except for a leading slash) are invalid. The last segment is the file name, which can be
/// anything, including `.` and `..`, as the storage appends an extension to it. A name with
/// folders is prefixed with `#`.
fn storage_name(name: &str) -> Option<String> {
    if name.is_empty() {
        return None;
    }

    let (folders, file_name) = name.rsplit_once('/').unwrap_or(("", name));
    let mut kept_folders = Vec::new();

    for (i, folder) in folders.split('/').enumerate() {
        match folder {
            "" if i == 0 => {}
            "." => {}
            "" | ".." => return None,
            folder => kept_folders.push(folder),
        }
    }

    if kept_folders.is_empty() {
        Some(file_name.to_owned())
    } else {
        Some(format!("#{}/{file_name}", kept_folders.join("/")))
    }
}

/// Flash Player returns the same object for names that only differ in leading `/` and `./`,
/// but not for other `.` folders, even though such names are stored under the same name.
fn instance_name(mut name: &str) -> &str {
    while let Some(rest) = name.strip_prefix('/').or_else(|| name.strip_prefix("./")) {
        name = rest;
    }
    name
}

#[cfg(test)]
mod tests {
    use super::{LocalSharedObjectKey, local_shared_object_key};

    fn key(movie_url: &str, name: &str, local_path: Option<&str>) -> Option<LocalSharedObjectKey> {
        local_shared_object_key(movie_url, name, local_path, false)
    }

    #[test]
    fn movie_in_hidden_directory() {
        let storage = |movie_url, local_path| {
            key(movie_url, "save", local_path).map(|k| k.storage().to_owned())
        };
        assert_eq!(
            storage("file:///home/user/.cache/game.swf", None).as_deref(),
            Some("localhost/home/user/.cache/game.swf/save")
        );
        assert_eq!(
            storage(
                "file:///home/user/.cache/game.swf",
                Some("/home/user/.cache")
            )
            .as_deref(),
            Some("localhost/home/user/.cache/save")
        );
        assert_eq!(
            storage("https://example.com/.games/game.swf", None).as_deref(),
            Some("example.com/.games/game.swf/save")
        );
    }

    #[test]
    fn name_segments() {
        let storage = |name| key("file:///game.swf", name, None).map(|k| k.storage().to_owned());
        assert_eq!(
            storage(".save").as_deref(),
            Some("localhost/game.swf/.save")
        );
        assert_eq!(storage("a/./b").as_deref(), Some("localhost/game.swf/#a/b"));
        assert_eq!(storage("./a/b").as_deref(), Some("localhost/game.swf/#a/b"));
        assert_eq!(storage("/save").as_deref(), Some("localhost/game.swf/save"));
        assert_eq!(
            storage("save/").as_deref(),
            Some("localhost/game.swf/#save/")
        );
        assert_eq!(
            storage("save/..").as_deref(),
            Some("localhost/game.swf/#save/..")
        );
        assert_eq!(storage(""), None);
        assert_eq!(storage("a//b"), None);
        assert_eq!(storage("../save"), None);
        assert_eq!(storage("a/./../b"), None);
    }

    #[test]
    fn instances() {
        let instance = |name| key("file:///game.swf", name, None).map(|k| k.instance().to_owned());
        assert_eq!(instance("././save"), instance("save"));
        assert_eq!(instance("/save"), instance("save"));
        assert_ne!(instance("a/./b"), instance("a/b"));
    }
}
