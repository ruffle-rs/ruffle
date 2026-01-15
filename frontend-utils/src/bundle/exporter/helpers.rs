use std::path::{Path, PathBuf};

use thiserror::Error;
use url::Url;

use crate::bundle::exporter::{BundleExportError, BundleExporter};
use crate::bundle::info::BundleInformation;

#[derive(Debug, Error)]
pub enum FilesystemHelperError {
    #[error("No files to export")]
    NoFilesToExport,

    #[error("Some provided paths are not absolute")]
    NonAbsolutePaths,

    #[error("Provided files do not have a common directory")]
    NoCommonDirectory,

    #[error("Provided path is outside of the common directory: {0}")]
    PathOutOfCommonDirectory(PathBuf),

    #[error("Path not representable as URL: {0}")]
    PathNotRepresentableAsUrl(PathBuf),

    #[error("Error exporting bundle: {0}")]
    ErrorExportingBundle(#[from] BundleExportError),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub struct FilesystemHelper<P: AsRef<Path>> {
    root_dir: PathBuf,
    files_to_export: Vec<P>,
}

impl<P: AsRef<Path>> FilesystemHelper<P> {
    pub fn new(files_to_export: Vec<P>) -> Result<Self, FilesystemHelperError> {
        let root_dir = Self::calculate_common_prefix(&files_to_export)?;

        Ok(Self {
            root_dir,
            files_to_export,
        })
    }

    fn calculate_common_prefix(files: &[P]) -> Result<PathBuf, FilesystemHelperError> {
        let mut common_prefix = files
            .first()
            .ok_or(FilesystemHelperError::NoFilesToExport)?
            .as_ref()
            .parent()
            .ok_or(FilesystemHelperError::NoCommonDirectory)?
            .to_owned();

        for file in files {
            if !file.as_ref().is_absolute() {
                return Err(FilesystemHelperError::NonAbsolutePaths);
            }

            while !file.as_ref().starts_with(&common_prefix) {
                common_prefix = common_prefix
                    .parent()
                    .ok_or(FilesystemHelperError::NoCommonDirectory)?
                    .to_owned();
            }
        }

        Ok(common_prefix)
    }

    pub fn real_path_to_bundle_path<'b>(
        &self,
        real_path: &'b Path,
    ) -> Result<&'b Path, FilesystemHelperError> {
        real_path
            .strip_prefix(&self.root_dir)
            .map_err(|_| FilesystemHelperError::PathOutOfCommonDirectory(real_path.to_owned()))
    }

    pub fn real_path_to_bundle_url(&self, real_path: &Path) -> Result<Url, FilesystemHelperError> {
        let bundle_path = self.real_path_to_bundle_path(real_path)?;
        let bundle_path = PathBuf::from("/").join(bundle_path);

        #[cfg(unix)]
        return Url::from_file_path(&bundle_path)
            .map_err(|_| FilesystemHelperError::PathNotRepresentableAsUrl(real_path.to_owned()));

        // Non-unix paths are weird, just manually create a file:///a/b/c URL
        #[cfg(not(unix))]
        {
            fn path_to_url_non_unix(path: &Path) -> Result<Url, ()> {
                let mut url_path = String::new();
                for component in path.components() {
                    let component = component.as_os_str().to_str().ok_or(())?;
                    url_path.push('/');
                    url_path.push_str(component);
                }
                let mut url = Url::parse("file://").unwrap();
                url.set_path(&url_path);
                Ok(url)
            }

            return path_to_url_non_unix(&bundle_path).map_err(|_| {
                FilesystemHelperError::PathNotRepresentableAsUrl(real_path.to_owned())
            });
        }
    }

    pub fn write_files<W: std::io::Write + std::io::Seek>(
        &self,
        exporter: &mut BundleExporter<W>,
    ) -> Result<(), FilesystemHelperError> {
        for file in &self.files_to_export {
            let path = self.real_path_to_bundle_path(file.as_ref())?;
            exporter.write_content(path, &mut std::fs::File::open(file)?)?;
        }

        Ok(())
    }

    pub fn export_bundle(
        &self,
        mut info: BundleInformation,
        output: &Path,
    ) -> Result<(), FilesystemHelperError> {
        let file = std::fs::File::create(output)?;

        if let Ok(root_movie_path) = info.url.to_file_path() {
            info.url = self.real_path_to_bundle_url(&root_movie_path)?;
        }

        let mut exporter = BundleExporter::new(file, info);
        self.write_files(&mut exporter)?;
        exporter.finish()?;
        Ok(())
    }
}

#[cfg(test)]
mod fs_tests {
    use std::path::{Path, PathBuf};

    use url::Url;

    use crate::bundle::exporter::helpers::{FilesystemHelper, FilesystemHelperError};

    #[cfg(not(windows))]
    fn new_pathbuf(string_path: &str) -> PathBuf {
        PathBuf::from(string_path)
    }

    #[cfg(windows)]
    fn new_pathbuf(string_path: &str) -> PathBuf {
        PathBuf::from(format!("d:{}", string_path))
    }

    #[test]
    fn test_calculate_common_prefix_root_dir_basic() {
        let files_to_export = vec![
            //
            new_pathbuf("/a/b/c/file1"),
            new_pathbuf("/a/b/c/file2"),
        ];

        assert_eq!(
            FilesystemHelper::calculate_common_prefix(&files_to_export).unwrap(),
            new_pathbuf("/a/b/c")
        );
    }

    #[test]
    fn test_calculate_common_prefix_root_dir_advanced() {
        let files_to_export = vec![
            //
            new_pathbuf("/a/b/c/file1"),
            new_pathbuf("/a/b/c/file2"),
            new_pathbuf("/a/b/f/file3"),
            new_pathbuf("/a/b/g/file4"),
            new_pathbuf("/a/b/g/h/file5"),
        ];

        assert_eq!(
            FilesystemHelper::calculate_common_prefix(&files_to_export).unwrap(),
            new_pathbuf("/a/b")
        );
    }

    #[test]
    fn test_calculate_common_prefix_root_dir_root() {
        let files_to_export = vec![
            //
            new_pathbuf("/a/b/c/file1"),
            new_pathbuf("/a/b/c/file2"),
            new_pathbuf("/a/b/f/file3"),
            new_pathbuf("/a/b/g/file4"),
            new_pathbuf("/a/b/g/h/file5"),
            new_pathbuf("/u/b/g/h/file5"),
        ];

        assert_eq!(
            FilesystemHelper::calculate_common_prefix(&files_to_export).unwrap(),
            new_pathbuf("/")
        );
    }

    #[test]
    fn test_calculate_common_prefix_root_dir_single_file() {
        let files_to_export = vec![
            //
            new_pathbuf("/a/b/c/d/file1"),
        ];

        assert_eq!(
            FilesystemHelper::calculate_common_prefix(&files_to_export).unwrap(),
            new_pathbuf("/a/b/c/d")
        );
    }

    #[test]
    fn test_calculate_common_prefix_root_dir_relative() {
        let files_to_export = vec![
            //
            new_pathbuf("a/b/c/file1"),
            new_pathbuf("a/b/c/file2"),
        ];

        assert!(matches!(
            FilesystemHelper::calculate_common_prefix(&files_to_export),
            Err(FilesystemHelperError::NonAbsolutePaths)
        ));
    }

    #[test]
    fn test_calculate_common_prefix_root_dir_mixed1() {
        let files_to_export = vec![
            //
            new_pathbuf("/a/b/c/file1"),
            new_pathbuf("a/b/c/file2"),
        ];

        assert!(matches!(
            FilesystemHelper::calculate_common_prefix(&files_to_export),
            Err(FilesystemHelperError::NonAbsolutePaths)
        ));
    }

    #[test]
    fn test_calculate_common_prefix_root_dir_mixed2() {
        let files_to_export = vec![
            //
            new_pathbuf("a/b/c/file2"),
            new_pathbuf("/a/b/c/file1"),
        ];

        assert!(matches!(
            FilesystemHelper::calculate_common_prefix(&files_to_export),
            Err(FilesystemHelperError::NonAbsolutePaths)
        ));
    }

    #[test]
    fn test_real_path_to_bundle_path() {
        let files_to_export = vec![
            //
            new_pathbuf("/a/b/c/file1"),
            new_pathbuf("/a/b/c/file2"),
            new_pathbuf("/a/b/f/file3"),
            new_pathbuf("/a/b/g/file4"),
            new_pathbuf("/a/b/g/h/file5"),
        ];
        let helper = FilesystemHelper::new(files_to_export).unwrap();

        assert_eq!(
            helper
                .real_path_to_bundle_path(&new_pathbuf("/a/b/c/movie.swf"))
                .unwrap(),
            Path::new("c/movie.swf")
        );
        assert_eq!(
            helper
                .real_path_to_bundle_path(&new_pathbuf("/a/b/c/inner/movie.swf"))
                .unwrap(),
            Path::new("c/inner/movie.swf")
        );
        assert_eq!(
            helper
                .real_path_to_bundle_path(&new_pathbuf("/a/b/image.png"))
                .unwrap(),
            Path::new("image.png")
        );
        assert!(matches!(
            helper.real_path_to_bundle_path(&new_pathbuf("/a/bc/image.png")),
            Err(FilesystemHelperError::PathOutOfCommonDirectory(_))
        ));
        assert!(matches!(
            helper.real_path_to_bundle_path(&new_pathbuf("/a/g/image.png")),
            Err(FilesystemHelperError::PathOutOfCommonDirectory(_))
        ));
        assert!(matches!(
            helper.real_path_to_bundle_path(&new_pathbuf("/a/image.png")),
            Err(FilesystemHelperError::PathOutOfCommonDirectory(_))
        ));
    }

    #[test]
    fn test_real_path_to_bundle_url() {
        let files_to_export = vec![
            //
            new_pathbuf("/a/b/c/file1"),
            new_pathbuf("/a/b/c/file2"),
            new_pathbuf("/a/b/f/file3"),
            new_pathbuf("/a/b/g/file4"),
            new_pathbuf("/a/b/g/h/file5"),
        ];
        let helper = FilesystemHelper::new(files_to_export).unwrap();

        assert_eq!(
            helper
                .real_path_to_bundle_url(&new_pathbuf("/a/b/c/movie.swf"))
                .unwrap(),
            Url::parse("file:///c/movie.swf").unwrap()
        );
        assert_eq!(
            helper
                .real_path_to_bundle_url(&new_pathbuf("/a/b/movie.swf"))
                .unwrap(),
            Url::parse("file:///movie.swf").unwrap()
        );
        assert_eq!(
            helper
                .real_path_to_bundle_url(&new_pathbuf("/a/b/g/inner/movie.swf"))
                .unwrap(),
            Url::parse("file:///g/inner/movie.swf").unwrap()
        );
        assert!(matches!(
            helper.real_path_to_bundle_url(&new_pathbuf("/a/bg/root_movie.swf")),
            Err(FilesystemHelperError::PathOutOfCommonDirectory(_))
        ));
    }
}
