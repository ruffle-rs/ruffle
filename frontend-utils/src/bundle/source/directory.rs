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
        let root = self.join("content").canonicalize()?;
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
    use tempfile::{tempdir, NamedTempFile};

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
}
