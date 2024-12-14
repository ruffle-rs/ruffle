use crate::bundle::source::BundleSourceImpl;
use std::cell::RefCell;
use std::io::{Cursor, Error, ErrorKind, Read, Seek};
use zip::result::ZipError;
use zip::ZipArchive;

pub struct ZipSource<R: Read + Seek>(RefCell<ZipArchive<R>>);

impl<R: Read + Seek> ZipSource<R> {
    pub fn open(reader: R) -> Result<Self, ZipError> {
        Ok(Self(RefCell::new(ZipArchive::new(reader)?)))
    }
}

impl<R: Read + Seek> BundleSourceImpl for ZipSource<R> {
    type Read = Cursor<Vec<u8>>;

    fn read_file(&self, path: &str) -> Result<Self::Read, Error> {
        let mut self_ref = self.0.borrow_mut();
        let mut result = self_ref
            .by_name(path.strip_prefix('/').unwrap_or(path))
            .map_err(|e| match e {
                ZipError::Io(e) => e,
                ZipError::InvalidArchive(_) => e.into(),
                ZipError::UnsupportedArchive(_) => e.into(),
                ZipError::FileNotFound => Error::from(ErrorKind::NotFound),
                ZipError::InvalidPassword => Error::from(ErrorKind::PermissionDenied),
                _ => Error::from(ErrorKind::Other),
            })?;
        let mut buf = vec![];
        result.read_to_end(&mut buf)?;
        Ok(Cursor::new(buf))
    }

    fn read_content(&self, path: &str) -> Result<Self::Read, Error> {
        let path = path.strip_prefix('/').unwrap_or(path);
        self.read_file(&format!("content/{path}"))
    }
}

#[cfg(test)]
mod tests {
    use crate::bundle::source::zip::ZipSource;
    use crate::bundle::source::BundleSourceImpl;
    use std::io::{Cursor, ErrorKind, Read};
    use zip::result::ZipError;

    #[test]
    fn open_not_a_zip() {
        assert!(matches!(
            ZipSource::open(Cursor::new(&[0, 1, 2, 3])),
            Err(ZipError::InvalidArchive(_))
        ))
    }

    #[test]
    fn read_file_not_found() {
        let not_a_zip = include_bytes!("./test-assets/empty.zip");
        let source = ZipSource::open(Cursor::new(not_a_zip)).unwrap();
        assert!(matches!(
            source.read_file("some_file"),
            Err(e) if e.kind() == ErrorKind::NotFound
        ))
    }

    #[test]
    fn read_file_valid() {
        let not_a_zip = include_bytes!("./test-assets/just-bundle.zip");
        let source = ZipSource::open(Cursor::new(not_a_zip)).unwrap();
        let mut file = source.read_file("ruffle-bundle.toml").unwrap();
        let mut string = String::new();
        file.read_to_string(&mut string).unwrap();
        assert_eq!("[bundle]\nname = \"Ruffle Logo Animation\"\nurl = \"https://ruffle.rs/demo/logo-anim.swf\"", string);
    }

    #[test]
    fn read_content_not_found() {
        let not_a_zip = include_bytes!("./test-assets/empty.zip");
        let source = ZipSource::open(Cursor::new(not_a_zip)).unwrap();

        assert!(matches!(
            source.read_content("some_file"),
            Err(e) if e.kind() == ErrorKind::NotFound
        ))
    }

    #[test]
    fn read_content_not_in_content_dir() {
        let not_a_zip = include_bytes!("./test-assets/just-bundle.zip");
        let source = ZipSource::open(Cursor::new(not_a_zip)).unwrap();

        assert!(matches!(
            source.read_content("../ruffle-bundle.toml"),
            Err(e) if e.kind() == ErrorKind::NotFound
        ))
    }

    #[test]
    fn read_content_valid() {
        let not_a_zip = include_bytes!("./test-assets/bundle-and-content.xip");
        let source = ZipSource::open(Cursor::new(not_a_zip)).unwrap();
        let mut file = source.read_content("/foo.txt").unwrap();
        let mut string = String::new();
        file.read_to_string(&mut string).unwrap();
        assert_eq!("Hello world!\n", string);
    }
}
