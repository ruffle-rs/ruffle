//! Shared object logic common to AVM1 and AVM2.

/// Returns the storage key of the local shared object accessed by
/// `SharedObject.getLocal(name, local_path, secure)` from a movie at `movie_url`,
/// or `None` when the shared object can't be created.
pub fn local_shared_object_key(
    movie_url: &str,
    name: &str,
    local_path: Option<&str>,
    secure: bool,
) -> Option<String> {
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
        // Remove drive letter on Windows
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
    // SOName may be a path containing slashes. In this case, prefix with # to mimic Flash Player behavior.
    let prefix = if name.contains('/') { "#" } else { "" };
    let full_name = format!("{movie_host}/{local_path}/{prefix}{name}");

    // Avoid any paths with `..` to prevent SWFs from crawling the file system on desktop.
    // Flash will generally fail to save shared objects with a path component starting with `.`,
    // so let's disallow them altogether.
    if full_name.split('/').any(|s| s.starts_with('.')) {
        tracing::error!("SharedObject.get_local: Invalid path with .. segments");
        return None;
    }

    Some(full_name)
}
