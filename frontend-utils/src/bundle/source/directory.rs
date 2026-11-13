use crate::bundle::source::BundleSourceImpl;
use std::fs::File;
use std::io::{Error, ErrorKind};
use std::path::Path;

impl BundleSourceImpl for Path {
    type Read = File;

    fn read_file(&self, path: &str) -> Result<Self::Read, Error> {
        let potential_path = self
            .join(path.strip_prefix('/').unwrap_or(path))
            .canonicalize()?;
        if !potential_path.starts_with(self.canonicalize()?) {
            return Err(Error::from(ErrorKind::NotFound));
        }
        File::open(potential_path)
    }

    fn read_content(&self, path: &str) -> Result<Self::Read, Error> {
        let root = self.canonicalize()?.join("content");
        let potential_path = root
            .join(path.strip_prefix('/').unwrap_or(path))
            .canonicalize()?;
        if !potential_path.starts_with(root) {
            return Err(Error::from(ErrorKind::NotFound));
        }
        File::open(potential_path)
    }
}

#[cfg(test)]
mod tests {
    use crate::bundle::source::BundleSourceImpl;
    use std::io::{ErrorKind, Read, Write};
    use tempfile::{NamedTempFile, tempdir};

    /*
    [NA] Careful with panicking in these tests.
    tempdir() relies on Drop to clean up the directory,
    and since we're testing a real filesystem... it sucks if we leak.

    Construct the test, perform the test, drop the tmp_dir and *then* assert.
     */

    #[test]
    fn read_file_not_found() {
        let tmp_dir = tempdir().unwrap();
        let success = matches!(
            tmp_dir.path().read_file("some_file"),
            Err(e) if e.kind() == ErrorKind::NotFound
        );
        drop(tmp_dir);
        assert!(success)
    }

    #[test]
    fn read_file_invalid() {
        let tmp_dir = tempdir().unwrap();
        // [NA] Exact error depends on OS... just check it's an error, period.
        let success = tmp_dir.path().read_file("!?\\//").is_err();
        drop(tmp_dir);
        assert!(success)
    }

    #[test]
    fn read_file_works() {
        let tmp_dir = tempdir().unwrap();
        let _ = std::fs::write(tmp_dir.path().join("some_file.txt"), "Fancy!");
        let result = tmp_dir
            .path()
            .read_file("some_file.txt")
            .map_err(|e| e.to_string())
            .map(|mut f| {
                let mut result = String::new();
                let _ = f.read_to_string(&mut result);
                result
            });
        drop(tmp_dir);

        assert_eq!(result.as_deref(), Ok("Fancy!"))
    }

    #[test]
    fn read_file_outside_directory() {
        let tmp_dir = tempdir().unwrap();
        let mut tmp_file = NamedTempFile::new().unwrap();
        let _ = tmp_file.write(&[1, 2, 3, 4]);
        let success = matches!(
            tmp_dir
            .path()
            .read_file(&tmp_file.path().to_string_lossy()),
            Err(e) if e.kind() == ErrorKind::NotFound
        );
        drop(tmp_file);
        drop(tmp_dir);

        assert!(success)
    }

    #[test]
    fn read_content_not_found() {
        let tmp_dir = tempdir().unwrap();
        let success = matches!(
            tmp_dir.path().read_content("some_file"),
            Err(e) if e.kind() == ErrorKind::NotFound
        );
        drop(tmp_dir);
        assert!(success)
    }

    #[test]
    fn read_content_invalid() {
        let tmp_dir = tempdir().unwrap();

        // [NA] Exact error depends on OS... just check it's an error, period.
        let success = tmp_dir.path().read_content("!?\\//").is_err();
        drop(tmp_dir);
        assert!(success)
    }

    #[test]
    fn read_content_works() {
        let tmp_dir = tempdir().unwrap();
        let _ = std::fs::create_dir(tmp_dir.path().join("content"));
        let _ = std::fs::write(tmp_dir.path().join("content/some_file.txt"), "Fancy!");
        let result = tmp_dir
            .path()
            .read_content("some_file.txt")
            .map_err(|e| e.to_string())
            .map(|mut f| {
                let mut result = String::new();
                let _ = f.read_to_string(&mut result);
                result
            });
        drop(tmp_dir);

        assert_eq!(result.as_deref(), Ok("Fancy!"))
    }

    #[test]
    fn read_content_works_with_absolute_path() {
        let tmp_dir = tempdir().unwrap();
        let _ = std::fs::create_dir(tmp_dir.path().join("content"));
        let _ = std::fs::write(tmp_dir.path().join("content/some_file.txt"), "Fancy!");
        let result = tmp_dir
            .path()
            .read_content("/some_file.txt")
            .map_err(|e| e.to_string())
            .map(|mut f| {
                let mut result = String::new();
                let _ = f.read_to_string(&mut result);
                result
            });
        drop(tmp_dir);

        assert_eq!(result.as_deref(), Ok("Fancy!"))
    }

    #[test]
    fn read_content_outside_content_directory() {
        let tmp_dir = tempdir().unwrap();
        let _ = std::fs::write(tmp_dir.path().join("some_file.txt"), "Fancy!");
        let success = matches!(
            tmp_dir
            .path()
            .read_content("../some_file.txt"),
            Err(e) if e.kind() == ErrorKind::NotFound
        );
        drop(tmp_dir);

        assert!(success)
    }

    #[test]
    fn read_content_outside_root_directory() {
        let tmp_dir = tempdir().unwrap();
        let mut tmp_file = NamedTempFile::new().unwrap();
        let _ = tmp_file.write(&[1, 2, 3, 4]);
        let success = matches!(
            tmp_dir
            .path()
            .read_content(&tmp_file.path().to_string_lossy()),
            Err(e) if e.kind() == ErrorKind::NotFound
        );
        drop(tmp_file);
        drop(tmp_dir);

        assert!(success)
    }

    #[cfg(unix)]
    fn read_to_string(result: std::io::Result<std::fs::File>) -> Result<String, ErrorKind> {
        result.map_err(|e| e.kind()).map(|mut f| {
            let mut result = String::new();
            let _ = f.read_to_string(&mut result);
            result
        })
    }

    #[test]
    #[cfg(unix)]
    fn read_file_symlinks() {
        use std::os::unix::fs::symlink;

        // tmp_dir/
        //     outside_file.txt
        //     bundle/
        //         some_file.txt
        //         link_absolute.txt -> /.../bundle/some_file.txt
        //         link_relative.txt -> some_file.txt
        //         outside_link_absolute.txt -> /.../outside_file.txt
        //         outside_link_relative.txt -> ../outside_file.txt
        //     bundle_link_absolute -> /.../bundle
        //     bundle_link_relative -> bundle

        let tmp_dir = tempdir().unwrap();
        let outside_file = tmp_dir.path().join("outside_file.txt");
        let bundle_dir = tmp_dir.path().join("bundle");
        let file = bundle_dir.join("some_file.txt");
        let bundle_link_absolute = tmp_dir.path().join("bundle_link_absolute");
        let bundle_link_relative = tmp_dir.path().join("bundle_link_relative");
        let _ = std::fs::create_dir(&bundle_dir);
        let _ = std::fs::write(&outside_file, "Outside!");
        let _ = std::fs::write(&file, "Fancy!");
        let _ = symlink(&file, bundle_dir.join("link_absolute.txt"));
        let _ = symlink("some_file.txt", bundle_dir.join("link_relative.txt"));
        let _ = symlink(&outside_file, bundle_dir.join("outside_link_absolute.txt"));
        let _ = symlink(
            "../outside_file.txt",
            bundle_dir.join("outside_link_relative.txt"),
        );
        let _ = symlink(&bundle_dir, &bundle_link_absolute);
        let _ = symlink("bundle", &bundle_link_relative);

        let results = vec![
            (
                "link_absolute.txt",
                read_to_string(bundle_dir.read_file("link_absolute.txt")),
            ),
            (
                "link_relative.txt",
                read_to_string(bundle_dir.read_file("link_relative.txt")),
            ),
            (
                "outside_link_absolute.txt",
                read_to_string(bundle_dir.read_file("outside_link_absolute.txt")),
            ),
            (
                "outside_link_relative.txt",
                read_to_string(bundle_dir.read_file("outside_link_relative.txt")),
            ),
            (
                "bundle_link_absolute",
                read_to_string(bundle_link_absolute.read_file("some_file.txt")),
            ),
            (
                "bundle_link_relative",
                read_to_string(bundle_link_relative.read_file("some_file.txt")),
            ),
        ];
        drop(tmp_dir);

        assert_eq!(
            results,
            vec![
                ("link_absolute.txt", Ok("Fancy!".to_string())),
                ("link_relative.txt", Ok("Fancy!".to_string())),
                ("outside_link_absolute.txt", Err(ErrorKind::NotFound)),
                ("outside_link_relative.txt", Err(ErrorKind::NotFound)),
                ("bundle_link_absolute", Ok("Fancy!".to_string())),
                ("bundle_link_relative", Ok("Fancy!".to_string())),
            ]
        )
    }

    #[test]
    #[cfg(unix)]
    fn read_content_symlinks() {
        use std::os::unix::fs::symlink;

        // tmp_dir/
        //     outside_file.txt
        //     outside_content/
        //         some_file.txt
        //     bundle/
        //         root_file.txt
        //         content/
        //             some_file.txt
        //             link_absolute.txt -> /.../bundle/content/some_file.txt
        //             link_relative.txt -> some_file.txt
        //             root_link_absolute.txt -> /.../bundle/root_file.txt
        //             root_link_relative.txt -> ../root_file.txt
        //             outside_link_absolute.txt -> /.../outside_file.txt
        //             outside_link_relative.txt -> ../../outside_file.txt
        //     linked_bundle_absolute/
        //         real_content/
        //             some_file.txt
        //         content -> /.../linked_bundle_absolute/real_content
        //     linked_bundle_relative/
        //         real_content/
        //             some_file.txt
        //         content -> real_content
        //     outside_linked_bundle_absolute/
        //         content -> /.../outside_content
        //     outside_linked_bundle_relative/
        //         content -> ../outside_content

        let tmp_dir = tempdir().unwrap();
        let outside_file = tmp_dir.path().join("outside_file.txt");
        let bundle_dir = tmp_dir.path().join("bundle");
        let root_file = bundle_dir.join("root_file.txt");
        let content_dir = bundle_dir.join("content");
        let file = content_dir.join("some_file.txt");
        let _ = std::fs::create_dir_all(&content_dir);
        let _ = std::fs::write(&outside_file, "Outside!");
        let _ = std::fs::write(&root_file, "Root!");
        let _ = std::fs::write(&file, "Fancy!");
        let _ = symlink(&file, content_dir.join("link_absolute.txt"));
        let _ = symlink("some_file.txt", content_dir.join("link_relative.txt"));
        let _ = symlink(&root_file, content_dir.join("root_link_absolute.txt"));
        let _ = symlink(
            "../root_file.txt",
            content_dir.join("root_link_relative.txt"),
        );
        let _ = symlink(&outside_file, content_dir.join("outside_link_absolute.txt"));
        let _ = symlink(
            "../../outside_file.txt",
            content_dir.join("outside_link_relative.txt"),
        );

        let linked_bundle_absolute = tmp_dir.path().join("linked_bundle_absolute");
        let linked_bundle_relative = tmp_dir.path().join("linked_bundle_relative");
        for linked_bundle in [&linked_bundle_absolute, &linked_bundle_relative] {
            let real_content_dir = linked_bundle.join("real_content");
            let _ = std::fs::create_dir_all(&real_content_dir);
            let _ = std::fs::write(real_content_dir.join("some_file.txt"), "Fancy!");
        }
        let _ = symlink(
            linked_bundle_absolute.join("real_content"),
            linked_bundle_absolute.join("content"),
        );
        let _ = symlink("real_content", linked_bundle_relative.join("content"));

        let outside_content_dir = tmp_dir.path().join("outside_content");
        let outside_linked_bundle_absolute = tmp_dir.path().join("outside_linked_bundle_absolute");
        let outside_linked_bundle_relative = tmp_dir.path().join("outside_linked_bundle_relative");
        let _ = std::fs::create_dir(&outside_content_dir);
        let _ = std::fs::write(outside_content_dir.join("some_file.txt"), "Outside!");
        let _ = std::fs::create_dir(&outside_linked_bundle_absolute);
        let _ = std::fs::create_dir(&outside_linked_bundle_relative);
        let _ = symlink(
            &outside_content_dir,
            outside_linked_bundle_absolute.join("content"),
        );
        let _ = symlink(
            "../outside_content",
            outside_linked_bundle_relative.join("content"),
        );

        let results = vec![
            (
                "link_absolute.txt",
                read_to_string(bundle_dir.read_content("link_absolute.txt")),
            ),
            (
                "link_relative.txt",
                read_to_string(bundle_dir.read_content("link_relative.txt")),
            ),
            (
                "root_link_absolute.txt",
                read_to_string(bundle_dir.read_content("root_link_absolute.txt")),
            ),
            (
                "root_link_relative.txt",
                read_to_string(bundle_dir.read_content("root_link_relative.txt")),
            ),
            (
                "outside_link_absolute.txt",
                read_to_string(bundle_dir.read_content("outside_link_absolute.txt")),
            ),
            (
                "outside_link_relative.txt",
                read_to_string(bundle_dir.read_content("outside_link_relative.txt")),
            ),
            (
                "linked_bundle_absolute",
                read_to_string(linked_bundle_absolute.read_content("some_file.txt")),
            ),
            (
                "linked_bundle_relative",
                read_to_string(linked_bundle_relative.read_content("some_file.txt")),
            ),
            (
                "outside_linked_bundle_absolute",
                read_to_string(outside_linked_bundle_absolute.read_content("some_file.txt")),
            ),
            (
                "outside_linked_bundle_relative",
                read_to_string(outside_linked_bundle_relative.read_content("some_file.txt")),
            ),
        ];
        drop(tmp_dir);

        assert_eq!(
            results,
            vec![
                ("link_absolute.txt", Ok("Fancy!".to_string())),
                ("link_relative.txt", Ok("Fancy!".to_string())),
                ("root_link_absolute.txt", Err(ErrorKind::NotFound)),
                ("root_link_relative.txt", Err(ErrorKind::NotFound)),
                ("outside_link_absolute.txt", Err(ErrorKind::NotFound)),
                ("outside_link_relative.txt", Err(ErrorKind::NotFound)),
                ("linked_bundle_absolute", Err(ErrorKind::NotFound)),
                ("linked_bundle_relative", Err(ErrorKind::NotFound)),
                ("outside_linked_bundle_absolute", Err(ErrorKind::NotFound)),
                ("outside_linked_bundle_relative", Err(ErrorKind::NotFound)),
            ]
        )
    }
}
